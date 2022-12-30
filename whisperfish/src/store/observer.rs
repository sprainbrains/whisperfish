//! Storage observer subsystem

use super::orm;
use actix::prelude::*;

pub enum Interest {
    All,
}

#[derive(Clone, Message)]
#[rtype(result = "()")]
pub enum Event {
    Any,
}

impl Event {
    pub fn new_reaction(_msg: &orm::Message, _author: &orm::Recipient, _reaction: &str) -> Self {
        Self::Any
    }
}

impl Interest {
    pub fn is_interesting(&self, _ev: &Event) -> bool {
        true
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

    pub(super) fn distribute_event(&mut self, event: Event) {
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
                    Some(subscriber) => match subscriber.do_send(event.clone()) {
                        Ok(()) => (),
                        Err(SendError::Full(_)) => {
                            log::warn!(
                                "Dropping an event for a subscriber because of a full mailbox."
                            );
                        }
                        Err(SendError::Closed(_)) => {
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
    fn observe(&mut self, event: Event);
    fn interests() -> Vec<Interest>;
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

    pub(super) fn distribute_event(&mut self, event: Event) {
        let observatory = self.observatory.clone();
        actix::spawn(async move {
            let mut observatory = observatory.write().await;
            observatory.distribute_event(event);
        });
    }
}
