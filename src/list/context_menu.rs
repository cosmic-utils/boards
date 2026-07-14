// SPDX-License-Identifier: GPL-3.0

use std::collections::HashMap;

use cosmic::widget::{self, menu};
use uuid::Uuid;

use crate::{app::Message, fl, list::message::ListMessage};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ListMenuAction {
    AddCard(Uuid),
    Rename(Uuid),
    SortByTag(Uuid),
    Delete(Uuid),
}

impl menu::Action for ListMenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match *self {
            ListMenuAction::AddCard(list_id) => {
                Message::List(ListMessage::OpenNewCardInput(widget::Id::unique(), list_id))
            }
            ListMenuAction::Rename(list_id) => Message::List(ListMessage::ToggleTitleEdit(list_id)),
            ListMenuAction::SortByTag(list_id) => Message::List(ListMessage::SortByTag(list_id)),
            ListMenuAction::Delete(list_id) => Message::List(ListMessage::Delete(list_id)),
        }
    }
}

pub fn list_context_menu(list_id: Uuid) -> Option<Vec<menu::Tree<Message>>> {
    Some(menu::items(
        &HashMap::new(),
        vec![
            menu::Item::Button(fl!("add-card"), None, ListMenuAction::AddCard(list_id)),
            menu::Item::Button(fl!("rename"), None, ListMenuAction::Rename(list_id)),
            menu::Item::Button(fl!("sort-by-tag"), None, ListMenuAction::SortByTag(list_id)),
            menu::Item::Divider,
            menu::Item::Button(fl!("delete"), None, ListMenuAction::Delete(list_id)),
        ],
    ))
}
