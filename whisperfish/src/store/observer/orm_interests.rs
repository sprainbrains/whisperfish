use crate::{schema, store::orm};

use super::Interest;

impl orm::Session {
    pub fn interests(&self) -> impl Iterator<Item = Interest> + '_ {
        std::iter::once(Interest::row(schema::sessions::table, self.id))
        // TODO:
        // - If group, watch the group
        // - If 1:1, watch the recipient
    }
}

impl orm::AugmentedSession {
    pub fn interests(&self) -> impl Iterator<Item = Interest> + '_ {
        self.inner
            .interests()
            .chain(
                self.last_message
                    .iter()
                    .flat_map(orm::AugmentedMessage::interests),
            )
            .chain(
                // XXX what about new group members?
                self.group_members
                    .iter()
                    .flat_map(orm::Recipient::interests),
            )
            // XXX this should be only newly inserted messages related to this session, but alas
            .chain(std::iter::once(Interest::whole_table(
                schema::messages::table,
            )))
    }
}

impl orm::Recipient {
    pub fn interests(&self) -> impl Iterator<Item = Interest> + '_ {
        std::iter::once(Interest::row(schema::recipients::table, self.id))
    }
}

impl orm::AugmentedMessage {
    pub fn interests(&self) -> impl Iterator<Item = Interest> + '_ {
        self.inner
            .interests()
            .chain(self.sender.iter().flat_map(orm::Recipient::interests))
            .chain(self.attachments.iter().flat_map(orm::Attachment::interests))
            // XXX this should be filtered for the current relation
            .chain(std::iter::once(Interest::whole_table(
                schema::attachments::table,
            )))
            .chain(
                self.reactions
                    .iter()
                    .flat_map(|(reaction, sender)| reaction.interests().chain(sender.interests())),
            )
            // XXX this should be filtered for the current relation
            .chain(std::iter::once(Interest::whole_table(
                schema::reactions::table,
            )))
            .chain(
                self.receipts
                    .iter()
                    .flat_map(|(receipt, sender)| receipt.interests().chain(sender.interests())),
            )
            // XXX this should be filtered for the current relation
            .chain(std::iter::once(Interest::whole_table(
                schema::receipts::table,
            )))
            .chain(
                // This box is necessary because of the recursion, which otherwise builds an
                // infinitely big type or a non-fixed type, and then Rust throws a very ugly
                // diagnostic to your head.
                // https://github.com/rust-lang/rust/issues/97686
                Box::new(self.quoted_message.iter().flat_map(|m| m.interests()))
                    as Box<dyn Iterator<Item = Interest>>,
            )
    }
}

impl orm::Message {
    pub fn interests(&self) -> impl Iterator<Item = Interest> + '_ {
        std::iter::once(Interest::row(schema::messages::table, self.id))
    }
}

impl orm::Attachment {
    pub fn interests(&self) -> impl Iterator<Item = Interest> + '_ {
        std::iter::once(Interest::row(schema::attachments::table, self.id))
    }
}

impl orm::Reaction {
    pub fn interests(&self) -> impl Iterator<Item = Interest> + '_ {
        // XXX This is a composite primary key
        // std::iter::once(Interest::row(schema::reactions::table, self.id))
        std::iter::once(Interest::whole_table(schema::reactions::table))
    }
}

impl orm::Receipt {
    pub fn interests(&self) -> impl Iterator<Item = Interest> + '_ {
        // XXX This is a composite primary key
        // std::iter::once(Interest::row(schema::receipts::table, self.id))
        std::iter::once(Interest::whole_table(schema::receipts::table))
    }
}

impl orm::GroupV1 {
    pub fn interests(&self) -> impl Iterator<Item = Interest> + '_ {
        std::iter::once(Interest::row(schema::group_v1s::table, self.id.clone()))
    }
}

impl orm::GroupV2 {
    pub fn interests(&self) -> impl Iterator<Item = Interest> + '_ {
        std::iter::once(Interest::row(schema::group_v2s::table, self.id.clone()))
    }
}
