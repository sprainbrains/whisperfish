//! Storage observer subsystem

use crate::schema;

mod orm_interests;

use actix::prelude::*;

#[derive(Debug, Clone)]
pub enum Interest {
    All,
    Row { table: Table, key: PrimaryKey },
    Table { table: Table },
}

impl Interest {
    pub fn whole_table<T: diesel::Table + 'static>(_table: T) -> Self {
        let table = Table::from_diesel::<T>();
        Interest::Table { table }
    }

    pub fn row<T: diesel::Table + 'static>(_table: T, key: impl Into<PrimaryKey>) -> Self {
        let table = Table::from_diesel::<T>();
        Interest::Row {
            table,
            key: key.into(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Table {
    Attachments,
    GroupV1Members,
    GroupV1s,
    GroupV2Members,
    GroupV2s,
    IdentityRecords,
    Messages,
    Prekeys,
    Reactions,
    Receipts,
    Recipients,
    SenderKeyRecords,
    SessionRecords,
    Sessions,
    SignedPrekeys,
    Stickers,
}

#[derive(Clone, Message, Debug)]
#[rtype(result = "Vec<Interest>")]
pub enum Event {
    Any,
    Insert { table: Table, key: PrimaryKey },
    Update { table: Table, key: PrimaryKey },
    Delete { table: Table, key: PrimaryKey },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrimaryKey {
    Unknown,
    RowId(i32),
    StringRowId(String),
}

impl From<i32> for PrimaryKey {
    fn from(x: i32) -> Self {
        Self::RowId(x)
    }
}

impl From<String> for PrimaryKey {
    fn from(x: String) -> Self {
        Self::StringRowId(x)
    }
}

impl Event {}

impl Interest {
    pub fn is_interesting(&self, ev: &Event) -> bool {
        match (self, ev) {
            (_, Event::Any) | (Interest::All, _) => true,

            // Interested in a whole table, and an event on the table is triggered
            (Interest::Table { table: ti }, Event::Insert { table: te, key: _ }) => ti == te,
            (Interest::Table { table: ti }, Event::Update { table: te, key: _ }) => ti == te,
            (Interest::Table { table: ti }, Event::Delete { table: te, key: _ }) => ti == te,

            // Interested in a particular row, and an event is triggered on some unknown row
            (
                Interest::Row { table: ti, key: _ },
                Event::Insert {
                    table: te,
                    key: PrimaryKey::Unknown,
                },
            ) => ti == te,
            (
                Interest::Row { table: ti, key: _ },
                Event::Update {
                    table: te,
                    key: PrimaryKey::Unknown,
                },
            ) => ti == te,
            (
                Interest::Row { table: ti, key: _ },
                Event::Delete {
                    table: te,
                    key: PrimaryKey::Unknown,
                },
            ) => ti == te,

            // Interested in a particular row, and an event is triggered on that specific row
            (Interest::Row { table: ti, key: ki }, Event::Insert { table: te, key: ke }) => {
                ti == te && ki == ke
            }
            (Interest::Row { table: ti, key: ki }, Event::Update { table: te, key: ke }) => {
                ti == te && ki == ke
            }
            (Interest::Row { table: ti, key: ki }, Event::Delete { table: te, key: ke }) => {
                ti == te && ki == ke
            }
            #[allow(unreachable_patterns)] // XXX should one of the enums be non-exhaustive instead?
            _ => {
                log::debug!(
                    "Unhandled event-interest pair; assuming interesting. {:?} {:?}",
                    ev,
                    self
                );
                true
            }
        }
    }
}

impl Table {
    fn from_diesel<T: diesel::Table + 'static>() -> Self {
        macro_rules! diesel_to_observation_table {
            ($($table:ident => $variant:ident,)* $(,)?) => {
                $(if std::any::TypeId::of::<T>() == std::any::TypeId::of::<schema::$table::table>() {
                    return Self::$variant
                })*
            }
        }
        diesel_to_observation_table!(
            attachments => Attachments,
            group_v1_members => GroupV1Members,
            group_v1s => GroupV1s,
            group_v2_members => GroupV2Members,
            group_v2s => GroupV2s,
            identity_records => IdentityRecords,
            messages => Messages,
            prekeys => Prekeys,
            reactions => Reactions,
            receipts => Receipts,
            recipients => Recipients,
            sender_key_records => SenderKeyRecords,
            session_records => SessionRecords,
            sessions => Sessions,
            signed_prekeys => SignedPrekeys,
            stickers => Stickers,
        );

        unimplemented!("Unknown diesel table")
    }
}

pub struct Subscription {
    interests: Vec<Interest>,
    subscriber: actix::WeakRecipient<Event>,
}

/// The Observatory watches the database for changes, and dispatches it to interested [Observer]s.
#[derive(Default)]
pub struct Observatory {
    subscriptions: Vec<Subscription>,
}

impl Observatory {
    pub fn register(&mut self, interests: Vec<Interest>, subscriber: actix::WeakRecipient<Event>) {
        self.subscriptions.push(Subscription {
            interests,
            subscriber,
        });
    }

    pub async fn distribute_event(&mut self, event: Event) {
        // Remove stale subscriptions
        self.subscriptions
            .retain(|x| x.subscriber.upgrade().is_some());

        for subscription in &mut self.subscriptions {
            if subscription
                .interests
                .iter()
                .any(|x| x.is_interesting(&event))
            {
                match subscription.subscriber.upgrade() {
                    Some(subscriber) => match subscriber.send(event.clone()).await {
                        Ok(interests) => {
                            subscription.interests = interests;
                        }
                        Err(MailboxError::Timeout) => {
                            log::warn!("Dropping an event for a subscriber because of a timeout.");
                        }
                        Err(MailboxError::Closed) => {
                            log::warn!("Mailbox has closed meanwhile.  Dropping with next event.");
                        }
                    },
                    None => {
                        log::warn!("Subscriber got dropped while processing.");
                    }
                }
            }
        }
    }
}

pub trait EventObserving {
    fn observe(&mut self, storage: super::Storage, event: Event);
    fn interests(&self) -> Vec<Interest>;
}

impl super::Storage {
    pub fn register_observer(
        &mut self,
        interests: Vec<Interest>,
        subscriber: actix::WeakRecipient<Event>,
    ) {
        let observatory = self.observatory.clone();
        actix::spawn(async move {
            let mut observatory = observatory.write().await;
            observatory.register(interests, subscriber);
        });
    }

    pub(super) fn distribute_event(&self, event: Event) {
        let observatory = self.observatory.clone();
        actix::spawn(async move {
            let mut observatory = observatory.write().await;
            observatory.distribute_event(event).await;
        });
    }

    pub(super) fn observe_insert<T: diesel::Table + 'static>(
        &self,
        _table: T,
        key: impl Into<PrimaryKey>,
    ) {
        let table = Table::from_diesel::<T>();

        self.distribute_event(Event::Insert {
            table,
            key: key.into(),
        });
    }

    pub(super) fn observe_update<T: diesel::Table + 'static>(
        &self,
        _table: T,
        key: impl Into<PrimaryKey>,
    ) {
        let table = Table::from_diesel::<T>();

        self.distribute_event(Event::Update {
            table,
            key: key.into(),
        });
    }

    pub(super) fn observe_delete<T: diesel::Table + 'static>(
        &self,
        _table: T,
        key: impl Into<PrimaryKey>,
    ) {
        let table = Table::from_diesel::<T>();

        self.distribute_event(Event::Delete {
            table,
            key: key.into(),
        });
    }
}
