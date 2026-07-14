use cosmic::{Task, iced::Length, widget};
use dialogs::{DialogHost, DialogSlot};

use crate::fl;

#[derive(Debug, Clone)]
pub struct NewListDialog {
    pub dialog: DialogSlot<NewListState>,
}

impl NewListDialog {
    pub fn new() -> Self {
        Self {
            dialog: DialogSlot::new(NewListState::new()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NewListState {
    pub title: String,
    pub input_id: widget::Id,
}

#[derive(Debug, Clone)]
pub enum NewListMessage {
    NameChanged(String),
    Submit,
    Close,
}

impl NewListState {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            input_id: widget::Id::unique(),
        }
    }
}

impl DialogHost for NewListDialog {
    type Message = NewListMessage;

    fn dialog(&self) -> Option<cosmic::prelude::Element<'_, Self::Message>> {
        self.dialog.get().map(|state| {
            let title_input = widget::text_input(fl!("list-title"), state.title.as_str())
                .id(state.input_id.clone())
                .on_input(NewListMessage::NameChanged)
                .on_submit(|_| NewListMessage::Submit)
                .width(Length::Fill);

            let create_btn =
                widget::button::suggested(fl!("new-list")).on_press(NewListMessage::Submit);

            let close_btn = widget::button::standard(fl!("cancel")).on_press(NewListMessage::Close);

            widget::dialog()
                .title(fl!("new-list"))
                .body(fl!("list-title"))
                .control(title_input)
                .primary_action(create_btn)
                .secondary_action(close_btn)
                .into()
        })
    }

    fn update(&mut self, message: Self::Message) -> cosmic::prelude::Task<Self::Message> {
        match message {
            NewListMessage::NameChanged(title) => {
                if let Some(state) = self.dialog.get_mut() {
                    state.title = title;
                }
            }
            NewListMessage::Submit | NewListMessage::Close => {
                self.dialog.dismiss();
            }
        }
        Task::none()
    }
}
