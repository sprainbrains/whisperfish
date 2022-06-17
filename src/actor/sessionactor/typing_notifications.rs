use super::*;
use libsignal_service::proto::TypingMessage;

const TYPING_EXIPIRY_DELAY: std::time::Duration = std::time::Duration::from_secs(5);

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct TypingNotification(pub TypingMessage);

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct UpdateTypingNotifications;

pub(super) struct TypingQueueItem {
    inner: TypingMessage,
    expire: std::time::Instant,
}

impl Handler<TypingNotification> for SessionActor {
    type Result = ();

    fn handle(
        &mut self,
        TypingNotification(typing): TypingNotification,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        let expire = std::time::Instant::now() + TYPING_EXIPIRY_DELAY;
        self.typing_queue.push_back(TypingQueueItem {
            inner: typing,
            expire,
        });

        ctx.notify(UpdateTypingNotifications);
    }
}

impl Handler<UpdateTypingNotifications> for SessionActor {
    type Result = ();

    fn handle(&mut self, _: UpdateTypingNotifications, ctx: &mut Self::Context) -> Self::Result {
        if self.typing_queue.is_empty() {
            return;
        }

        // Remove all expired typing notifications.
        while let Some(item) = self.typing_queue.front() {
            if item.expire <= std::time::Instant::now() {
                self.typing_queue.pop_front();
            } else {
                break;
            }
        }

        if let Some(item) = self.typing_queue.front() {
            let next_delay = std::time::Instant::now() - item.expire;
            ctx.notify_later(UpdateTypingNotifications, next_delay);
        }

        // XXX update model
    }
}
