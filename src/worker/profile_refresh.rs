use std::{
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use chrono::prelude::*;
use diesel::prelude::*;
use futures::Stream;
use uuid::Uuid;

use crate::store::orm::Recipient;
use crate::store::Storage;

const REYIELD_DELAY: Duration = Duration::from_secs(5 * 60);

/// Stream that yields UUIDs of outdated profiles that require an update.
///
/// Only yields a UUID once every 5 minutes.
pub struct OutdatedProfileStream {
    ignore_set: Vec<(Instant, Uuid)>,
    storage: Storage,
    next_wake: Option<Pin<Box<tokio::time::Sleep>>>,
}

pub struct OutdatedProfile(pub Uuid);

impl OutdatedProfileStream {
    pub fn new(storage: Storage) -> Self {
        Self {
            ignore_set: Vec::new(),
            storage,
            next_wake: None,
        }
    }

    fn clean_ignore_set(&mut self) {
        self.ignore_set
            .retain(|(time, _uuid)| *time > Instant::now());
    }

    fn next_out_of_date_profile(&mut self) -> Option<OutdatedProfile> {
        use crate::schema::recipients::dsl::*;

        // https://github.com/signalapp/Signal-Android/blob/09b9349f6c0cf02688a79d8c2c9edeb8b32dd3cf/app/src/main/java/org/thoughtcrime/securesms/database/RecipientDatabase.kt#L3209
        let _last_interaction_threshold = Utc::now() - chrono::Duration::days(30);
        let last_fetch_threshold = Utc::now() - chrono::Duration::days(1);

        let db = self.storage.db.lock();
        let out_of_date_profiles: Vec<Recipient> = recipients
            .filter(
                profile_key.is_not_null().and(
                    uuid.is_not_null().and(
                        last_profile_fetch
                            .is_null()
                            .or(last_profile_fetch.le(last_fetch_threshold.naive_utc())),
                    ),
                ),
            )
            .order_by(last_profile_fetch.asc())
            .load(&*db)
            .expect("db");

        log::info!("Found {} out-of-date profiles.", out_of_date_profiles.len());

        for recipient in out_of_date_profiles {
            let recipient_uuid = recipient.uuid.as_ref().expect("database precondition");
            match self
                .ignore_set
                .binary_search_by(|(_time, other_uuid)| other_uuid.cmp(&recipient_uuid))
            {
                Ok(_present) => continue,
                Err(idx) => {
                    self.ignore_set
                        .insert(idx, (Instant::now() + REYIELD_DELAY, recipient_uuid));
                    return Some(OutdatedProfile(recipient_uuid));
                }
            }
        }

        None
    }

    fn compute_next_wake(&mut self) -> bool {
        // Either the next wake is because of the ignore set, or if that's empty, the next one in
        // the database.
        if let Some((time, _)) = self.ignore_set.iter().min_by_key(|(time, _)| time) {
            self.next_wake = Some(Box::pin(tokio::time::sleep_until(
                tokio::time::Instant::from_std(*time),
            )));
            return true;
        }

        // No immediate updates needed at this point,
        // so we look at the next recipient,
        // and schedule a wake.
        use crate::schema::recipients::dsl::*;

        let db = self.storage.db.lock();
        let next_wake: Option<Recipient> = recipients
            .filter(uuid.is_not_null())
            .order_by(last_profile_fetch.asc())
            .first(&*db)
            .optional()
            .expect("db");
        if let Some(recipient) = next_wake {
            let time = recipient
                .last_profile_fetch
                .expect("empty last_profile_fetch should be in ignore set");
            let time = chrono::offset::Utc.from_utc_datetime(&time);
            let delta = Utc::now() - time;
            self.next_wake = Some(Box::pin(tokio::time::sleep(
                delta.to_std().unwrap_or(REYIELD_DELAY),
            )));
            return true;
        }

        false
    }
}

impl Stream for OutdatedProfileStream {
    type Item = OutdatedProfile;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.clean_ignore_set();

        if let Some(out_of_date_profile) = self.next_out_of_date_profile() {
            return Poll::Ready(Some(out_of_date_profile));
        }

        self.compute_next_wake();
        let next_wake: Pin<&mut _> = self
            .next_wake
            .as_mut()
            .expect("next wake should have been set")
            .as_mut();

        futures::ready!(std::future::Future::poll(next_wake, cx));

        Poll::Pending
    }
}
