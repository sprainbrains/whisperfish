#![allow(non_snake_case)]

use std::collections::HashMap;

use crate::model::*;
use crate::store::observer::{EventObserving, Interest};
use crate::store::{orm, schema, Storage};
use qmetaobject::prelude::*;

/// QML-constructable object that interacts with a single message.
#[derive(Default, QObject)]
pub struct GroupedReactionsImpl {
    base: qt_base_class!(trait QObject),
    message_id: Option<i32>,

    grouped_reaction_list: QObjectBox<GroupedReactionListModel>,
}

crate::observing_model! {
    pub struct GroupedReactions(GroupedReactionsImpl) {
        messageId: i32;                READ get_message_id    WRITE set_message_id NOTIFY message_id_changed,
        valid: bool;                   READ get_valid                              NOTIFY valid_changed,
        groupedReactions: QVariant;    READ reactions                              NOTIFY reactions_changed,
        count: i32;                    READ reaction_count                         NOTIFY count_changed,
    }
}

impl EventObserving for GroupedReactionsImpl {
    type Context = ModelContext<Self>;

    fn observe(&mut self, ctx: Self::Context, event: crate::store::observer::Event) {
        if let Some(message_id) = self.message_id {
            self.grouped_reaction_list
                .pinned()
                .borrow_mut()
                .observe(ctx, message_id, event);
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
            .collect()
    }
}

define_model_roles! {
    pub(super) enum GroupedReactionRoles for orm::GroupedReaction {
        Reaction(emoji via QString::from): "reaction",
        Count(count): "count",
    }
}

impl GroupedReactionsImpl {
    fn get_message_id(&self) -> i32 {
        self.message_id.unwrap_or(-1)
    }

    fn get_valid(&self) -> bool {
        self.message_id.is_some()
    }

    fn reaction_count(&self) -> i32 {
        self.grouped_reaction_list.pinned().borrow().row_count()
    }

    fn fetch(&mut self, storage: Storage, id: i32) {
        self.grouped_reaction_list
            .pinned()
            .borrow_mut()
            .load_all(storage, id);
    }

    fn set_message_id(&mut self, ctx: Option<ModelContext<Self>>, id: i32) {
        self.message_id = Some(id);
        if let Some(ctx) = ctx {
            self.fetch(ctx.storage(), id);
        }
    }

    fn init(&mut self, ctx: ModelContext<Self>) {
        if let Some(id) = self.message_id {
            self.fetch(ctx.storage(), id);
        }
    }

    fn reactions(&self) -> QVariant {
        self.grouped_reaction_list.pinned().into()
    }
}

#[derive(QObject, Default)]
pub struct GroupedReactionListModel {
    base: qt_base_class!(trait QAbstractListModel),
    grouped_reactions: Vec<orm::GroupedReaction>,
}

impl GroupedReactionListModel {
    fn load_all(&mut self, storage: Storage, message_id: i32) {
        self.begin_reset_model();
        self.grouped_reactions = storage.fetch_grouped_reactions_for_message(message_id);
        self.end_reset_model();
    }

    fn observe(
        &mut self,
        ctx: ModelContext<GroupedReactionsImpl>,
        message_id: i32,
        _event: crate::store::observer::Event,
    ) {
        self.load_all(ctx.storage(), message_id);
    }
}

impl QAbstractListModel for GroupedReactionListModel {
    fn row_count(&self) -> i32 {
        self.grouped_reactions.len() as i32
    }

    fn data(&self, index: QModelIndex, role: i32) -> QVariant {
        let role = GroupedReactionRoles::from(role);
        role.get(&self.grouped_reactions[index.row() as usize])
    }

    fn role_names(&self) -> HashMap<i32, QByteArray> {
        GroupedReactionRoles::role_names()
    }
}
