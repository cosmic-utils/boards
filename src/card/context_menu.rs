// SPDX-License-Identifier: GPL-3.0

use std::collections::HashMap;

use cosmic::widget::menu;
use uuid::Uuid;

use crate::{
    app::Message, board::context::BoardContext, card::Card, card::message::CardMessage, fl,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CardMenuAction {
    Open(Uuid),
    MoveUp(Uuid),
    MoveDown(Uuid),
    MoveToColumn {
        card_id: Uuid,
        target_column_id: Uuid,
    },
    ToggleTag {
        card_id: Uuid,
        tag_id: Uuid,
    },
    ClearDueDate(Uuid),
    Delete(Uuid),
}

impl menu::Action for CardMenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        let card_message = match *self {
            CardMenuAction::Open(card_id) => CardMessage::Open(card_id),
            CardMenuAction::MoveUp(card_id) => CardMessage::MoveUp(card_id),
            CardMenuAction::MoveDown(card_id) => CardMessage::MoveDown(card_id),
            CardMenuAction::MoveToColumn {
                card_id,
                target_column_id,
            } => CardMessage::MoveToColumn {
                card_id,
                target_column_id,
            },
            CardMenuAction::ToggleTag { card_id, tag_id } => {
                CardMessage::ToggleTag { card_id, tag_id }
            }
            CardMenuAction::ClearDueDate(card_id) => CardMessage::ClearDueDate(card_id),
            CardMenuAction::Delete(card_id) => CardMessage::Delete(card_id),
        };
        Message::CardMenuAction(card_message)
    }
}

pub fn card_context_menu(card: &Card, ctx: &BoardContext) -> Option<Vec<menu::Tree<Message>>> {
    let card_id = card.id;

    let mut items = vec![
        menu::Item::Button(fl!("open"), None, CardMenuAction::Open(card_id)),
        menu::Item::Divider,
        menu::Item::Button(fl!("move-up"), None, CardMenuAction::MoveUp(card_id)),
        menu::Item::Button(fl!("move-down"), None, CardMenuAction::MoveDown(card_id)),
    ];

    if !ctx.other_columns.is_empty() {
        let move_to_column = ctx
            .other_columns
            .iter()
            .map(|(target_column_id, title)| {
                menu::Item::Button(
                    title.clone(),
                    None,
                    CardMenuAction::MoveToColumn {
                        card_id,
                        target_column_id: *target_column_id,
                    },
                )
            })
            .collect();
        items.push(menu::Item::Folder(fl!("move-to-column"), move_to_column));
    }

    if !ctx.tags.is_empty() {
        let tags_menu = ctx
            .tags
            .iter()
            .map(|tag| {
                let active = card.tag_ids.contains(&tag.id);
                menu::Item::CheckBox(
                    tag.name.clone(),
                    None,
                    active,
                    CardMenuAction::ToggleTag {
                        card_id,
                        tag_id: tag.id,
                    },
                )
            })
            .collect();
        items.push(menu::Item::Divider);
        items.push(menu::Item::Folder(fl!("tags"), tags_menu));
    }

    if card.due_date.is_some() {
        items.push(menu::Item::Button(
            fl!("clear-due-date"),
            None,
            CardMenuAction::ClearDueDate(card_id),
        ));
    }

    items.push(menu::Item::Divider);
    items.push(menu::Item::Button(
        fl!("delete-card"),
        None,
        CardMenuAction::Delete(card_id),
    ));

    Some(menu::items(&HashMap::new(), items))
}
