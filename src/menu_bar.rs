// SPDX-License-Identifier: GPL-3.0

//! roots exposing the board-level actions that don't already live in a
//! right-click context menu.

use std::collections::HashMap;

use cosmic::iced::keyboard::Key;
use cosmic::iced::keyboard::key::Named;
use cosmic::prelude::*;
use cosmic::widget::menu::{self, key_bind::KeyBind, key_bind::Modifier};
use cosmic::widget::menu::{ItemHeight, ItemWidth};
use uuid::Uuid;

use crate::{
    app::{AppModel, ContextPage, DialogPage, Message},
    board::{dialog::NewBoardDialog, message::BoardMessage},
    column::dialog::NewColumnDialog,
    fl,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    NewBoard,
    NewColumn,
    BoardSettings(Uuid),
    DeleteBoard(Uuid),
    ToggleSidebar,
    Quit,
    About,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match *self {
            MenuAction::NewBoard => {
                Message::OpenDialogPage(DialogPage::NewBoard(NewBoardDialog::new()))
            }
            MenuAction::NewColumn => {
                Message::OpenDialogPage(DialogPage::NewColumn(NewColumnDialog::new()))
            }
            MenuAction::BoardSettings(id) => Message::Board(BoardMessage::OpenSettings(id)),
            MenuAction::DeleteBoard(id) => Message::Board(BoardMessage::Delete(id)),
            MenuAction::ToggleSidebar => Message::ToggleNavBar,
            MenuAction::Quit => Message::Quit,
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
        }
    }
}

fn item(label: String, enabled: bool, action: MenuAction) -> menu::Item<MenuAction, String> {
    if enabled {
        menu::Item::Button(label, None, action)
    } else {
        menu::Item::ButtonDisabled(label, None, action)
    }
}

pub fn menu_bar(app: &AppModel) -> Element<'_, Message> {
    let has_board = app.active_board().is_some();
    let active_board_id = app.active_board_id().unwrap_or_default();

    let file_items = vec![
        menu::Item::Button(fl!("new-board"), None, MenuAction::NewBoard),
        item(fl!("new-column"), has_board, MenuAction::NewColumn),
        menu::Item::Divider,
        menu::Item::Button(fl!("quit"), None, MenuAction::Quit),
    ];

    let edit_items = vec![
        item(
            fl!("board-settings"),
            has_board,
            MenuAction::BoardSettings(active_board_id),
        ),
        item(
            fl!("delete-board"),
            has_board,
            MenuAction::DeleteBoard(active_board_id),
        ),
    ];

    let view_items = vec![menu::Item::Button(
        fl!("toggle-sidebar"),
        None,
        MenuAction::ToggleSidebar,
    )];

    let help_items = vec![menu::Item::Button(fl!("about"), None, MenuAction::About)];

    menu::bar(vec![
        menu::Tree::with_children(
            menu::root(fl!("file")).apply(Element::from),
            menu::items(&app.key_binds, file_items),
        ),
        menu::Tree::with_children(
            menu::root(fl!("edit")).apply(Element::from),
            menu::items(&app.key_binds, edit_items),
        ),
        menu::Tree::with_children(
            menu::root(fl!("view")).apply(Element::from),
            menu::items(&app.key_binds, view_items),
        ),
        menu::Tree::with_children(
            menu::root(fl!("help")).apply(Element::from),
            menu::items(&app.key_binds, help_items),
        ),
    ])
    .item_height(ItemHeight::Dynamic(40))
    .item_width(ItemWidth::Uniform(260))
    .spacing(4.0)
    .into()
}

pub fn key_binds() -> HashMap<KeyBind, MenuAction> {
    let mut binds = HashMap::new();

    macro_rules! bind {
        ([$($modifier:ident),* $(,)?], $key:expr, $action:expr) => {
            binds.insert(
                KeyBind {
                    modifiers: vec![$(Modifier::$modifier),*],
                    key: $key,
                },
                $action,
            );
        };
    }

    if cfg!(target_os = "macos") {
        bind!([Super], Key::Character("n".into()), MenuAction::NewBoard);
        bind!(
            [Super, Shift],
            Key::Character("n".into()),
            MenuAction::NewColumn
        );
        bind!(
            [Super],
            Key::Character(",".into()),
            MenuAction::BoardSettings(Uuid::nil())
        );
        bind!(
            [Super],
            Key::Character("b".into()),
            MenuAction::ToggleSidebar
        );
        bind!([Super], Key::Character("q".into()), MenuAction::Quit);
        bind!([], Key::Named(Named::F1), MenuAction::About);
    } else {
        bind!([Ctrl], Key::Character("n".into()), MenuAction::NewBoard);
        bind!(
            [Ctrl, Shift],
            Key::Character("n".into()),
            MenuAction::NewColumn
        );
        bind!(
            [Ctrl],
            Key::Character(",".into()),
            MenuAction::BoardSettings(Uuid::nil())
        );
        bind!(
            [Ctrl],
            Key::Character("b".into()),
            MenuAction::ToggleSidebar
        );
        bind!([Ctrl], Key::Character("q".into()), MenuAction::Quit);
        bind!([], Key::Named(Named::F1), MenuAction::About);
    }

    binds
}
