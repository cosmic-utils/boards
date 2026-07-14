use crate::dialog::{DialogHost, DialogSlot};
use cosmic::{Task, iced::Length, widget};

use crate::fl;

#[derive(Debug, Clone)]
pub struct NewColumnDialog {
    pub dialog: DialogSlot<NewColumnState>,
}

impl NewColumnDialog {
    pub fn new() -> Self {
        Self {
            dialog: DialogSlot::new(NewColumnState::new()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NewColumnState {
    pub title: String,
    pub input_id: widget::Id,
}

#[derive(Debug, Clone)]
pub enum NewColumnMessage {
    NameChanged(String),
    Submit,
    Close,
}

impl NewColumnState {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            input_id: widget::Id::unique(),
        }
    }
}

impl DialogHost for NewColumnDialog {
    type Message = NewColumnMessage;

    fn dialog(&self) -> Option<cosmic::prelude::Element<'_, Self::Message>> {
        self.dialog.get().map(|state| {
            let title_input = widget::text_input(fl!("column-title"), state.title.as_str())
                .id(state.input_id.clone())
                .on_input(NewColumnMessage::NameChanged)
                .on_submit(|_| NewColumnMessage::Submit)
                .width(Length::Fill);

            let create_btn =
                widget::button::suggested(fl!("new-column")).on_press(NewColumnMessage::Submit);

            let close_btn =
                widget::button::standard(fl!("cancel")).on_press(NewColumnMessage::Close);

            widget::dialog()
                .title(fl!("new-column"))
                .body(fl!("column-title"))
                .control(title_input)
                .primary_action(create_btn)
                .secondary_action(close_btn)
                .into()
        })
    }

    fn update(&mut self, message: Self::Message) -> cosmic::prelude::Task<Self::Message> {
        match message {
            NewColumnMessage::NameChanged(title) => {
                if let Some(state) = self.dialog.get_mut() {
                    state.title = title;
                }
            }
            NewColumnMessage::Submit | NewColumnMessage::Close => {
                self.dialog.dismiss();
            }
        }
        Task::none()
    }
}
