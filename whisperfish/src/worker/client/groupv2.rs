use actix::prelude::*;
use diesel::prelude::*;
use qmeta_async::with_executor;

use libsignal_service::groups_v2::*;

use crate::{
    actor::FetchSession,
    store::{GroupV2, TrustLevel},
};

use super::*;

#[derive(Message)]
#[rtype(result = "()")]
/// Request group v2 metadata from server by session id
pub struct RequestGroupV2InfoBySessionId(pub i32);

#[derive(Message)]
#[rtype(result = "()")]
/// Request group v2 metadata from server
pub struct RequestGroupV2Info(pub GroupV2);

impl ClientWorker {
    #[with_executor]
    pub fn refresh_group_v2(&self, session_id: usize) {
        log::trace!("Request to refresh group v2 by session id = {}", session_id);

        let client = self.actor.clone().unwrap();
        let session = self.session_actor.clone().unwrap();
        actix::spawn(async move {
            client
                .send(RequestGroupV2InfoBySessionId(session_id as _))
                .await
                .unwrap();
            session
                .send(FetchSession {
                    id: session_id as _,
                    mark_read: false,
                })
                .await
                .unwrap();
        });
    }
}

impl Handler<RequestGroupV2Info> for ClientActor {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(
        &mut self,
        RequestGroupV2Info(request): RequestGroupV2Info,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let storage = self.storage.clone().unwrap();
        let uuid = self.uuid().expect("whoami");

        let authenticated_service = self.authenticated_service();
        let zk_params = self.service_cfg().zkgroup_server_public_params;
        let group_id = request.secret.get_group_identifier();
        let group_id_hex = hex::encode(group_id);

        Box::pin(
            async move {
                let mut credential_cache = storage.credential_cache_mut().await;
                let mut gm =
                    GroupsManager::new(authenticated_service, &mut *credential_cache, zk_params);
                let credentials = gm.get_authorization_for_today(uuid, request.secret).await?;
                let group = gm.get_group(request.secret, credentials).await?;
                // We now know the group's name and properties
                // XXX this is an assumption that we might want to check.
                let acl = group
                    .access_control
                    .as_ref()
                    .expect("access control present in DecryptedGroup");
                {
                    // XXX if the group does not exist, consider inserting here.
                    let db = storage.db.lock();
                    use crate::schema::group_v2s::dsl::*;
                    diesel::update(group_v2s)
                        .set((
                            name.eq(&group.title),
                            // TODO: maybe rename the SQLite column to version
                            revision.eq(group.version as i32),
                            invite_link_password.eq(&group.invite_link_password),
                            access_required_for_attributes.eq(acl.attributes),
                            access_required_for_members.eq(acl.members),
                            access_required_for_add_from_invite_link.eq(acl.add_from_invite_link),
                        ))
                        .filter(id.eq(&group_id_hex))
                        .execute(&*db)
                        .expect("update groupv2 name");
                }
                {
                    let timeout = group
                        .disappearing_messages_timer
                        .as_ref()
                        .map(|d| d.duration as i32);
                    let db = storage.db.lock();
                    use crate::schema::sessions::dsl::*;
                    diesel::update(sessions)
                        .set((expiring_message_timeout.eq(timeout),))
                        .filter(group_v2_id.eq(&group_id_hex))
                        .execute(&*db)
                        .expect("update session disappearing_messages_timer");
                }

                // We know the group's members.
                // First assert their existence in the database.
                // We can assert existence for members, pending members, and requesting members.
                let members_to_assert = group
                    .members
                    .iter()
                    .map(|member| (&member.uuid, Some(&member.profile_key)))
                    .chain(
                        group
                            .pending_members
                            .iter()
                            .map(|member| (&member.uuid, None)),
                    )
                    .chain(
                        group
                            .requesting_members
                            .iter()
                            .map(|member| (&member.uuid, Some(&member.profile_key))),
                    );

                // We need all the profile keys and UUIDs in the database.
                for (uuid, profile_key) in members_to_assert {
                    let recipient = storage.fetch_or_insert_recipient_by_uuid(&uuid.to_string());
                    if let Some(profile_key) = profile_key {
                        let recipient = storage.update_profile_key(recipient.e164.as_deref(), recipient.uuid.as_deref(), &profile_key.get_bytes(), TrustLevel::Uncertain);
                        match recipient.profile_key {
                            Some(key) if key == profile_key.get_bytes() => {
                                log::trace!("Profile key matches server-stored profile key");
                            }
                            Some(_key) => {
                                // XXX trigger a profile key update message
                                log::warn!("Profile key does not match server-stored profile key.");
                            }
                            None => {
                                log::error!("Profile key None but tried to set.  This will probably crash a bit later.");
                            },
                        }
                    }
                }

                // Now the members are stored as recipient in the database.
                // Let's link them with the group in two steps (in one migration):
                // 1. Delete all existing memberships.
                // 2. Insert all memberships from the DecryptedGroup.
                let uuids = group.members.iter().map(|member| {
                    member.uuid.to_string()
                });
                let db = storage.db.lock();
                db.transaction(|| -> Result<(), diesel::result::Error> {
                    use crate::schema::{group_v2_members, recipients};
                    let stale_members: Vec<i32> = group_v2_members::table
                        .select(group_v2_members::recipient_id)
                        .inner_join(recipients::table)
                        .filter(
                            recipients::uuid
                                .ne_all(uuids)
                                .and(group_v2_members::group_v2_id.eq(&group_id_hex)),
                        )
                        .load(&*db)?;
                    log::trace!("Have {} stale members", stale_members.len());
                    let dropped = diesel::delete(group_v2_members::table)
                        .filter(
                            group_v2_members::group_v2_id
                                .eq(&group_id_hex)
                                .and(group_v2_members::recipient_id.eq_any(&stale_members)),
                        )
                        .execute(&*db)?;
                    assert_eq!(
                        stale_members.len(),
                        dropped,
                        "didn't drop all stale members"
                    );

                    for member in &group.members {
                        // XXX there's a bit of duplicate work going on here.
                        let recipient =
                            storage.fetch_or_insert_recipient_by_uuid(&member.uuid.to_string());
                        log::trace!(
                            "Asserting {} as a member of the group",
                            recipient.e164_or_uuid()
                        );

                        // Upsert in Diesel 2.0... Manually for now.
                        let membership: Option<orm::GroupV2Member> = group_v2_members::table
                            .filter(
                                group_v2_members::recipient_id
                                    .eq(recipient.id)
                                    .and(group_v2_members::group_v2_id.eq(&group_id_hex)),
                            )
                            .first(&*db)
                            .optional()?;
                        if let Some(membership) = membership {
                            log::trace!(
                                "  Member {} already in db. Updating membership.",
                                recipient.e164_or_uuid()
                            );
                            log::info!("Existing membership {:?}; updating", membership);
                            diesel::update(group_v2_members::table)
                                .set((group_v2_members::role.eq(member.role as i32),))
                                .filter(
                                    group_v2_members::recipient_id
                                        .eq(recipient.id)
                                        .and(group_v2_members::group_v2_id.eq(&group_id_hex)),
                                )
                                .execute(&*db)?;
                        } else {
                            log::info!("  Member is new, inserting.");
                            diesel::insert_into(group_v2_members::table)
                                .values((
                                    group_v2_members::group_v2_id.eq(&group_id_hex.clone()),
                                    group_v2_members::recipient_id.eq(recipient.id),
                                    group_v2_members::joined_at_revision
                                        .eq(member.joined_at_revision as i32),
                                    group_v2_members::role.eq(member.role as i32),
                                ))
                                .execute(&*db)?;
                        }
                    }
                    Ok(())
                })
                .expect("updated members");

                // XXX there's more stuff to store from the DecryptedGroup.

                Ok::<_, anyhow::Error>(group)
            }
            .into_actor(self)
            .map(|result, _act, _ctx| {
                let _group = match result {
                    Ok(g) => g,
                    Err(e) => {
                        log::error!("Could not update group: {}", e);
                        return;
                    }
                };
                // XXX send notification of group update to UI for refresh.
            }),
        )
    }
}

impl Handler<RequestGroupV2InfoBySessionId> for ClientActor {
    type Result = ();

    fn handle(
        &mut self,
        RequestGroupV2InfoBySessionId(sid): RequestGroupV2InfoBySessionId,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        match self
            .storage
            .as_ref()
            .unwrap()
            .fetch_session_by_id(sid)
            .map(|s| s.r#type)
        {
            Some(orm::SessionType::GroupV2(group_v2)) => {
                let mut key_stack = [0u8; zkgroup::GROUP_MASTER_KEY_LEN];
                key_stack.clone_from_slice(&hex::decode(group_v2.master_key).expect("hex in db"));
                let key = GroupMasterKey::new(key_stack);
                let secret = GroupSecretParams::derive_from_master_key(key);

                let store_v2 = crate::store::GroupV2 {
                    secret,
                    revision: group_v2.revision as _,
                };
                ctx.notify(RequestGroupV2Info(store_v2));
            }
            _ => {
                log::warn!("No group_v2 with session id {}", sid);
            }
        }
    }
}
