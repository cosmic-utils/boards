use crate::dialog::{DialogHost, DialogSlot};
use cosmic::iced::{Background, Color};
use cosmic::{Element, Task, iced::Length, widget};
use icon_index::IconIndex;
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

use crate::board::Board;
use crate::fl;

/// Caps how many icon buttons a search can lay out per render, since even
/// a narrow query can otherwise match into the thousands of icons in a
/// full system icon index.
const MAX_ICON_RESULTS: usize = 10;

/// Shown by default, before the user searches for something else.
const CURATED_ICONS: &[&str] = &[
    "view-grid-symbolic",
    "view-list-symbolic",
    "view-continuous-symbolic",
    "view-app-grid-symbolic",
    "go-home-symbolic",
    "bookmark-new-symbolic",
    "edit-find-symbolic",
    "mail-send-symbolic",
    "folder-symbolic",
    "folder-documents-symbolic",
    "folder-download-symbolic",
    "folder-music-symbolic",
    "folder-pictures-symbolic",
    "folder-videos-symbolic",
    "folder-publicshare-symbolic",
    "user-home-symbolic",
    "starred-symbolic",
    "emblem-favorite-symbolic",
    "emblem-important-symbolic",
    "emblem-ok-symbolic",
    "emblem-photos-symbolic",
    "emblem-shared-symbolic",
    "x-office-calendar-symbolic",
    "x-office-address-book-symbolic",
    "x-office-document-symbolic",
    "x-office-spreadsheet-symbolic",
    "x-office-presentation-symbolic",
    "x-office-drawing-symbolic",
    "applications-office-symbolic",
    "accessories-text-editor-symbolic",
    "accessories-calculator-symbolic",
    "accessories-dictionary-symbolic",
    "applications-development-symbolic",
    "applications-graphics-symbolic",
    "applications-engineering-symbolic",
    "applications-science-symbolic",
    "applications-system-symbolic",
    "utilities-terminal-symbolic",
    "preferences-system-symbolic",
    "preferences-desktop-symbolic",
    "system-users-symbolic",
    "computer-symbolic",
    "network-workgroup-symbolic",
    "network-wireless-symbolic",
    "network-server-symbolic",
    "drive-harddisk-symbolic",
    "printer-symbolic",
    "mail-unread-symbolic",
    "user-available-symbolic",
    "contact-new-symbolic",
    "applications-games-symbolic",
    "applications-multimedia-symbolic",
    "camera-photo-symbolic",
    "audio-headphones-symbolic",
    "multimedia-player-symbolic",
    "applications-internet-symbolic",
    "weather-clear-symbolic",
    "alarm-symbolic",
    "battery-symbolic",
];

fn icon_choice_button<'a>(name: &'a str, active: bool) -> Element<'a, BoardSettingsMessage> {
    let border_width = if active { 2.0 } else { 0.0 };
    let style = move |_focused: bool, theme: &cosmic::Theme| {
        let accent = Color::from(theme.cosmic().accent_color());
        widget::button::Style {
            background: active.then_some(Background::Color(Color { a: 0.15, ..accent })),
            border_radius: 8.0.into(),
            border_width,
            border_color: accent,
            ..Default::default()
        }
    };

    widget::button::icon(widget::icon::from_name(name))
        .class(cosmic::theme::Button::Custom {
            active: Box::new(style),
            disabled: Box::new(move |theme| style(false, theme)),
            hovered: Box::new(style),
            pressed: Box::new(style),
        })
        .on_press(BoardSettingsMessage::SetIcon(name.to_string()))
        .into()
}

#[derive(Debug, Clone)]
pub struct NewBoardDialog {
    pub dialog: DialogSlot<NewBoardState>,
}

#[derive(Debug, Clone)]
pub struct NewBoardState {
    pub title: String,
    pub input_id: widget::Id,
}

#[derive(Debug, Clone)]
pub enum NewBoardMessage {
    NameChanged(String),
    Submit,
    Close,
}

impl NewBoardDialog {
    pub fn new() -> Self {
        Self {
            dialog: DialogSlot::new(NewBoardState {
                title: String::new(),
                input_id: widget::Id::unique(),
            }),
        }
    }
}

impl DialogHost for NewBoardDialog {
    type Message = NewBoardMessage;

    fn dialog(&self) -> Option<Element<'_, Self::Message>> {
        self.dialog.get().map(|state| {
            let title_input = widget::text_input(fl!("board-title"), state.title.as_str())
                .id(state.input_id.clone())
                .on_input(NewBoardMessage::NameChanged)
                .on_submit(|_| NewBoardMessage::Submit)
                .width(Length::Fill);

            let create_btn =
                widget::button::suggested(fl!("new-board")).on_press(NewBoardMessage::Submit);

            let close_btn =
                widget::button::standard(fl!("cancel")).on_press(NewBoardMessage::Close);

            widget::dialog()
                .title(fl!("new-board"))
                .body(fl!("board-title"))
                .control(title_input)
                .primary_action(create_btn)
                .secondary_action(close_btn)
                .into()
        })
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            NewBoardMessage::NameChanged(title) => {
                if let Some(state) = self.dialog.get_mut() {
                    state.title = title;
                }
            }
            NewBoardMessage::Submit | NewBoardMessage::Close => {
                self.dialog.dismiss();
            }
        }
        Task::none()
    }
}

#[derive(Debug, Clone)]
pub struct BoardSettingsDialog {
    pub dialog: DialogSlot<BoardSettingsState>,
    pub icons: Option<Arc<IconIndex>>,
}

#[derive(Debug, Clone)]
pub struct BoardSettingsState {
    pub board_id: Uuid,
    pub title: String,
    pub input_id: widget::Id,
    pub current_icon: String,
    pub icon_query: String,
}

#[derive(Debug, Clone)]
pub enum BoardSettingsMessage {
    TitleChanged(String),
    Rename,
    SetIcon(String),
    IconQueryChanged(String),
    Delete,
    Close,
}

impl BoardSettingsDialog {
    pub fn new(board: &Board, icons: Option<Arc<IconIndex>>) -> Self {
        Self {
            dialog: DialogSlot::new(BoardSettingsState {
                board_id: board.id,
                title: board.title.clone(),
                input_id: widget::Id::unique(),
                current_icon: board.icon.clone(),
                icon_query: String::new(),
            }),
            icons,
        }
    }
}

impl DialogHost for BoardSettingsDialog {
    type Message = BoardSettingsMessage;

    fn dialog<'a>(&'a self) -> Option<Element<'a, Self::Message>> {
        self.dialog.get().map(|state| {
            let title_input = widget::text_input(fl!("board-name"), state.title.as_str())
                .id(state.input_id.clone())
                .on_input(BoardSettingsMessage::TitleChanged)
                .on_submit(|_| BoardSettingsMessage::Rename)
                .width(Length::Fill);

            let query = state.icon_query.trim();
            let icon_buttons: Vec<Element<'_, Self::Message>> = if query.is_empty() {
                CURATED_ICONS
                    .iter()
                    .map(|name| icon_choice_button(name, state.current_icon == *name))
                    .collect()
            } else {
                self.icons
                    .as_ref()
                    .map(|icons| {
                        let mut seen = HashSet::new();
                        icons
                            .search(query)
                            .filter(|icon| seen.insert(icon.name.as_str()))
                            .take(MAX_ICON_RESULTS)
                            .map(|icon| {
                                icon_choice_button(&icon.name, state.current_icon == *icon.name)
                            })
                            .collect()
                    })
                    .unwrap_or_default()
            };

            let icon_search = widget::text_input(fl!("search-icons"), state.icon_query.as_str())
                .on_input(BoardSettingsMessage::IconQueryChanged)
                .width(Length::Fill);

            let icon_section =
                widget::settings::section()
                    .add(icon_search)
                    .add(widget::scrollable(
                        widget::flex_row(icon_buttons).spacing(cosmic::theme::spacing().space_xs),
                    ));

            let content = widget::settings::view_column(vec![
                widget::settings::section()
                    .add(widget::settings::item::item(fl!("board-name"), title_input))
                    .into(),
                icon_section.into(),
            ]);

            let mut col = widget::column::with_capacity(2)
                .push(content)
                .spacing(cosmic::theme::spacing().space_s);

            col = col.push(
                widget::button::destructive(fl!("delete-board"))
                    .on_press(BoardSettingsMessage::Delete),
            );

            let save_btn =
                widget::button::suggested(fl!("save")).on_press(BoardSettingsMessage::Rename);
            let close_btn =
                widget::button::standard(fl!("cancel")).on_press(BoardSettingsMessage::Close);

            widget::dialog()
                .title(fl!("board-settings"))
                .control(col)
                .primary_action(save_btn)
                .secondary_action(close_btn)
                .into()
        })
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            BoardSettingsMessage::TitleChanged(title) => {
                if let Some(state) = self.dialog.get_mut() {
                    state.title = title;
                }
            }
            BoardSettingsMessage::SetIcon(icon) => {
                if let Some(state) = self.dialog.get_mut() {
                    state.current_icon = icon;
                }
            }
            BoardSettingsMessage::IconQueryChanged(query) => {
                if let Some(state) = self.dialog.get_mut() {
                    state.icon_query = query;
                }
            }
            BoardSettingsMessage::Rename
            | BoardSettingsMessage::Delete
            | BoardSettingsMessage::Close => {
                self.dialog.dismiss();
            }
        }
        Task::none()
    }
}
