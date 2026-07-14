// SPDX-License-Identifier: GPL-3.0

use std::collections::HashMap;

use cosmic::widget::{self, menu};
use uuid::Uuid;

use crate::{app::Message, column::message::ColumnMessage, fl};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ColumnMenuAction {
    AddCard(Uuid),
    Rename(Uuid),
    SortByTag(Uuid),
    Delete(Uuid),
}

impl menu::Action for ColumnMenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match *self {
            ColumnMenuAction::AddCard(column_id) => Message::Column(
                ColumnMessage::OpenNewCardInput(widget::Id::unique(), column_id),
            ),
            ColumnMenuAction::Rename(column_id) => {
                Message::Column(ColumnMessage::ToggleTitleEdit(column_id))
            }
            ColumnMenuAction::SortByTag(column_id) => {
                Message::Column(ColumnMessage::SortByTag(column_id))
            }
            ColumnMenuAction::Delete(column_id) => {
                Message::Column(ColumnMessage::Delete(column_id))
            }
        }
    }
}

pub fn column_context_menu(column_id: Uuid) -> Option<Vec<menu::Tree<Message>>> {
    Some(menu::items(
        &HashMap::new(),
        vec![
            menu::Item::Button(fl!("add-card"), None, ColumnMenuAction::AddCard(column_id)),
            menu::Item::Button(fl!("rename"), None, ColumnMenuAction::Rename(column_id)),
            menu::Item::Button(
                fl!("sort-by-tag"),
                None,
                ColumnMenuAction::SortByTag(column_id),
            ),
            menu::Item::Divider,
            menu::Item::Button(fl!("delete"), None, ColumnMenuAction::Delete(column_id)),
        ],
    ))
}
