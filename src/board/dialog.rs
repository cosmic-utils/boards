use cosmic::iced::{Background, Color};
use cosmic::{Element, Task, iced::Length, widget};
use dialogs::{DialogHost, DialogSlot};
use uuid::Uuid;

use crate::board::Board;
use crate::fl;

const BOARD_ICON_CHOICES: &[&str] = &[
    "view-grid-symbolic",
    "folder-symbolic",
    "user-home-symbolic",
    "emblem-favorite-symbolic",
    "x-office-calendar-symbolic",
    "applications-engineering-symbolic",
    "mail-unread-symbolic",
    "system-users-symbolic",
    "task-due-symbolic",
    "applications-games-symbolic",
    "weather-clear-symbolic",
    "emblem-important-symbolic",
];

fn icon_choice_button(name: &'static str, active: bool) -> Element<'static, BoardSettingsMessage> {
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
}

#[derive(Debug, Clone)]
pub struct BoardSettingsState {
    pub board_id: Uuid,
    pub title: String,
    pub input_id: widget::Id,
    pub current_icon: String,
}

#[derive(Debug, Clone)]
pub enum BoardSettingsMessage {
    TitleChanged(String),
    Rename,
    SetIcon(String),
    Delete,
    Close,
}

impl BoardSettingsDialog {
    pub fn new(board: &Board) -> Self {
        Self {
            dialog: DialogSlot::new(BoardSettingsState {
                board_id: board.id,
                title: board.title.clone(),
                input_id: widget::Id::unique(),
                current_icon: board.icon.clone(),
            }),
        }
    }
}

impl DialogHost for BoardSettingsDialog {
    type Message = BoardSettingsMessage;

    fn dialog(&self) -> Option<Element<'_, Self::Message>> {
        self.dialog.get().map(|state| {
            let title_input = widget::text_input(fl!("board-name"), state.title.as_str())
                .id(state.input_id.clone())
                .on_input(BoardSettingsMessage::TitleChanged)
                .on_submit(|_| BoardSettingsMessage::Rename)
                .width(Length::Fill);

            let icon_buttons: Vec<Element<'_, Self::Message>> = BOARD_ICON_CHOICES
                .iter()
                .map(|name| icon_choice_button(name, state.current_icon == *name))
                .collect();
            let icon_section = widget::settings::section().add(
                cosmic::widget::flex_row(icon_buttons).spacing(cosmic::theme::spacing().space_xs),
            );

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
            BoardSettingsMessage::Rename
            | BoardSettingsMessage::Delete
            | BoardSettingsMessage::Close => {
                self.dialog.dismiss();
            }
        }
        Task::none()
    }
}
