// SPDX-License-Identifier: GPL-3.0

use std::collections::HashMap;

use cosmic::widget::menu;
use uuid::Uuid;

use crate::{
    app::{AppModel, Message},
    board::message::BoardMessage,
    fl,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum NavMenuAction {
    OpenSettings(Uuid),
    Delete(Uuid),
}

impl menu::Action for NavMenuAction {
    type Message = cosmic::Action<Message>;

    fn message(&self) -> Self::Message {
        cosmic::Action::App(match *self {
            NavMenuAction::OpenSettings(id) => Message::Board(BoardMessage::OpenSettings(id)),
            NavMenuAction::Delete(id) => Message::Board(BoardMessage::Delete(id)),
        })
    }
}

impl AppModel {
    pub fn board_nav_menu(&self) -> Vec<menu::Tree<cosmic::Action<Message>>> {
        menu::nav_context(
            &HashMap::new(),
            self.nav
                .iter()
                .map(|entity| {
                    let Some(&id) = self.nav.data::<Uuid>(entity) else {
                        return Vec::new();
                    };
                    vec![
                        menu::Item::Button(
                            fl!("board-settings"),
                            None,
                            NavMenuAction::OpenSettings(id),
                        ),
                        menu::Item::Divider,
                        menu::Item::Button(fl!("delete"), None, NavMenuAction::Delete(id)),
                    ]
                })
                .collect(),
        )
    }
}
