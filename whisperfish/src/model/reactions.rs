#![allow(non_snake_case)]

use crate::store::observer::{EventObserving, Interest};
use crate::store::{orm, Storage};
use crate::{model::*, schema};
use qmetaobject::{prelude::*, QJsonObject};

/// QML-constructable object that interacts with a single session.
#[derive(Default, QObject)]
pub struct ReactionsImpl {
    base: qt_base_class!(trait QObject),
    message_id: Option<i32>,

    reaction_list: QObjectBox<ReactionListModel>,
}

crate::observing_model! {
    pub struct Reactions(ReactionsImpl) {
        messageId: i32; READ get_message_id WRITE set_message_id,
        valid: bool; READ get_valid,
        reactions: QVariant; READ reactions,
        groupedReactions: QJsonObject; READ grouped_reactions,
        count: i32; READ reaction_count,
    }
}

impl EventObserving for ReactionsImpl {
    fn observe(&mut self, storage: Storage, event: crate::store::observer::Event) {
        if let Some(message_id) = self.message_id {
            self.reaction_list
                .pinned()
                .borrow_mut()
                .observe(storage, message_id, event);
        }
    }

    fn interests(&self) -> Vec<Interest> {
        self.message_id
            .into_iter()
            .map(|id| {
                Interest::whole_table_with_relation(
                    schema::reactions::table,
                    schema::messages::table,
                    id,
                )
            })
            .chain(
                self.reaction_list
                    .pinned()
                    .borrow()
                    .reactions
                    .iter()
                    .flat_map(|(reaction, recipient)| {
                        reaction.interests().chain(recipient.interests())
                    }),
            )
            .collect()
    }
}

define_model_roles! {
    pub(super) enum ReactionRoles for orm::Reaction [with offset 100] {
        Id(reaction_id): "id",
        MessageId(message_id): "messageId",
        Author(author): "authorRecipientId",
        Emoji(emoji via QString::from): "emoji",
        SentTime(sent_time via qdatetime_from_naive): "sentTime",
        ReceivedTime(received_time via qdatetime_from_naive): "receivedTime",
    }
}

impl ReactionsImpl {
    fn get_message_id(&self) -> i32 {
        self.message_id.unwrap_or(-1)
    }

    fn get_valid(&self) -> bool {
        self.message_id.is_some()
    }

    fn reaction_count(&self) -> i32 {
        self.reaction_list.pinned().borrow().row_count()
    }

    fn fetch(&mut self, storage: Storage, id: i32) {
        self.reaction_list
            .pinned()
            .borrow_mut()
            .load_all(storage, id);
    }

    fn set_message_id(&mut self, storage: Option<Storage>, id: i32) {
        self.message_id = Some(id);
        if let Some(storage) = storage {
            self.fetch(storage, id);
        }
    }

    fn init(&mut self, storage: Storage) {
        if let Some(id) = self.message_id {
            self.fetch(storage, id);
        }
    }

    fn reactions(&self) -> QVariant {
        self.reaction_list.pinned().into()
    }

    fn grouped_reactions(&self) -> QJsonObject {
        let mut map = std::collections::HashMap::new();

        for (reaction, _) in &self.reaction_list.pinned().borrow().reactions {
            *map.entry(reaction.emoji.clone()).or_insert(0) += 1;
        }
        let mut qmap: QJsonObject = QJsonObject::default();
        for (emoji, count) in map {
            qmap.insert(&emoji, QVariant::from(count).into());
        }
        qmap
    }
}

#[derive(QObject, Default)]
pub struct ReactionListModel {
    base: qt_base_class!(trait QAbstractListModel),
    reactions: Vec<(orm::Reaction, orm::Recipient)>,
}

impl ReactionListModel {
    fn load_all(&mut self, storage: Storage, message_id: i32) {
        self.begin_reset_model();
        self.reactions = storage.fetch_reactions_for_message(message_id);
        self.end_reset_model();
    }

    fn observe(
        &mut self,
        storage: Storage,
        message_id: i32,
        _event: crate::store::observer::Event,
    ) {
        self.load_all(storage, message_id);
    }
}

impl QAbstractListModel for ReactionListModel {
    fn row_count(&self) -> i32 {
        self.reactions.len() as i32
    }

    fn data(&self, index: QModelIndex, role: i32) -> QVariant {
        if role > 100 {
            let role = ReactionRoles::from(role);
            role.get(&self.reactions[index.row() as usize].0)
        } else {
            let role = RecipientRoles::from(role);
            role.get(&self.reactions[index.row() as usize].1)
        }
    }

    fn role_names(&self) -> std::collections::HashMap<i32, QByteArray> {
        ReactionRoles::role_names()
            .into_iter()
            .chain(RecipientRoles::role_names().into_iter())
            .collect()
    }
}
