// SPDX-License-Identifier: GPL-3.0

use crate::dialog::{DialogHost, DialogSlot};
use cosmic::{
    Element, Task,
    iced::Length,
    widget::{
        self,
        color_picker::{ColorPickerModel, ColorPickerUpdate},
    },
};
use uuid::Uuid;

use crate::{
    fl,
    tag::{Tag, default_tag_color},
};

pub struct TagDialog {
    pub dialog: DialogSlot<TagState>,
}

pub struct TagState {
    pub editing_tag_id: Option<Uuid>,
    pub name: String,
    pub input_id: widget::Id,
    pub color_picker: ColorPickerModel,
}

#[derive(Debug, Clone)]
pub enum TagDialogMessage {
    NameChanged(String),
    ColorPicker(ColorPickerUpdate),
    Submit,
    Delete,
    Close,
}

impl TagDialog {
    pub fn new_create() -> Self {
        Self {
            dialog: DialogSlot::new(TagState {
                editing_tag_id: None,
                name: String::new(),
                input_id: widget::Id::unique(),
                color_picker: ColorPickerModel::new("Hex", "RGB", None, Some(default_tag_color())),
            }),
        }
    }

    pub fn new_edit(tag: &Tag) -> Self {
        Self {
            dialog: DialogSlot::new(TagState {
                editing_tag_id: Some(tag.id),
                name: tag.name.clone(),
                input_id: widget::Id::unique(),
                color_picker: ColorPickerModel::new("Hex", "RGB", None, Some(tag.color)),
            }),
        }
    }
}

impl DialogHost for TagDialog {
    type Message = TagDialogMessage;

    fn dialog(&self) -> Option<Element<'_, Self::Message>> {
        self.dialog.get().map(|state| {
            let space_s = cosmic::theme::spacing().space_s;

            let title = if state.editing_tag_id.is_some() {
                fl!("edit-tag")
            } else {
                fl!("new-tag")
            };

            let name_input = widget::text_input(fl!("tag-name"), state.name.as_str())
                .id(state.input_id.clone())
                .on_input(TagDialogMessage::NameChanged)
                .on_submit(|_| TagDialogMessage::Submit)
                .width(Length::Fill);

            let picker = state
                .color_picker
                .builder(TagDialogMessage::ColorPicker)
                .build(fl!("recent-colors"), fl!("copy-color"), fl!("copied-color"));

            let save_btn =
                widget::button::suggested(fl!("save")).on_press(TagDialogMessage::Submit);
            let close_btn =
                widget::button::standard(fl!("cancel")).on_press(TagDialogMessage::Close);

            let mut col = widget::column::with_capacity(3)
                .push(name_input)
                .push(picker)
                .spacing(space_s);

            if state.editing_tag_id.is_some() {
                col = col.push(
                    widget::button::destructive(fl!("delete-tag"))
                        .on_press(TagDialogMessage::Delete),
                );
            }

            widget::dialog()
                .title(title)
                .control(col)
                .primary_action(save_btn)
                .secondary_action(close_btn)
                .into()
        })
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            TagDialogMessage::NameChanged(name) => {
                if let Some(state) = self.dialog.get_mut() {
                    state.name = name;
                }
                Task::none()
            }
            TagDialogMessage::ColorPicker(update) => {
                if let Some(state) = self.dialog.get_mut() {
                    return state.color_picker.update(update);
                }
                Task::none()
            }
            TagDialogMessage::Submit | TagDialogMessage::Close | TagDialogMessage::Delete => {
                self.dialog.dismiss();
                Task::none()
            }
        }
    }
}
