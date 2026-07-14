// SPDX-License-Identifier: GPL-3.0

use cosmic::iced::{Alignment, Color};
use cosmic::{
    Element, Task,
    cosmic_theme::Spacing,
    iced::Length,
    widget::{
        self,
        calendar::CalendarModel,
        color_picker::{ColorPickerModel, ColorPickerUpdate, color_button},
    },
};
use uuid::Uuid;

use crate::{
    app::{AppModel, DialogPage, Message},
    card::message::CardMessage,
    checklist::message::ChecklistMessage,
    fl,
    tag::widget::tag_chip,
};

#[derive(Debug, Clone)]
pub struct CardDetailDialog {
    pub card_id: Uuid,
    pub checklist_input: String,
    pub calendar_visible: bool,
    pub calendar_model: CalendarModel,
}

impl CardDetailDialog {
    pub fn new(card_id: Uuid) -> Self {
        Self {
            card_id,
            checklist_input: String::new(),
            calendar_visible: false,
            calendar_model: CalendarModel::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum CardDetailsMessage {
    Card(CardMessage),
    ChecklistInputChanged(String),
    ConfirmChecklistItem,
    ToggleCalendar,
    CalendarPrevMonth,
    CalendarNextMonth,
    CalendarYearChanged(i32),
    CalendarDateSelected(jiff::civil::Date),
    Close,
}

impl AppModel {
    pub fn is_card_detail_open(&self, card_id: Uuid) -> bool {
        matches!(&self.page, Some(DialogPage::CardDetail(d)) if d.card_id == card_id)
    }

    pub fn update_card_detail(
        &mut self,
        message: CardDetailsMessage,
    ) -> Task<cosmic::Action<Message>> {
        let Some(DialogPage::CardDetail(dialog)) = &mut self.page else {
            return Task::none();
        };

        match message {
            CardDetailsMessage::Card(card_msg) => self.update_card(card_msg),

            CardDetailsMessage::ChecklistInputChanged(text) => {
                dialog.checklist_input = text;
                Task::none()
            }

            CardDetailsMessage::ConfirmChecklistItem => {
                let text = dialog.checklist_input.trim().to_string();
                if text.is_empty() {
                    return Task::none();
                }
                let card_id = dialog.card_id;
                dialog.checklist_input.clear();
                self.update_checklist(ChecklistMessage::Add { card_id, text })
            }

            CardDetailsMessage::ToggleCalendar => {
                dialog.calendar_visible = !dialog.calendar_visible;
                Task::none()
            }

            CardDetailsMessage::CalendarPrevMonth => {
                dialog.calendar_model.show_prev_month();
                Task::none()
            }

            CardDetailsMessage::CalendarNextMonth => {
                dialog.calendar_model.show_next_month();
                Task::none()
            }

            CardDetailsMessage::CalendarYearChanged(year) => {
                let vis = dialog.calendar_model.visible;
                if let Ok(new_vis) = jiff::civil::Date::new(year as i16, vis.month(), vis.day()) {
                    dialog.calendar_model.visible = new_vis;
                } else if let Ok(new_vis) = jiff::civil::Date::new(year as i16, vis.month(), 28) {
                    dialog.calendar_model.visible = new_vis;
                }
                let sel = dialog.calendar_model.selected;
                if let Ok(new_sel) = jiff::civil::Date::new(year as i16, sel.month(), sel.day()) {
                    dialog.calendar_model.selected = new_sel;
                } else if let Ok(new_sel) = jiff::civil::Date::new(year as i16, sel.month(), 28) {
                    dialog.calendar_model.selected = new_sel;
                }
                Task::none()
            }

            CardDetailsMessage::CalendarDateSelected(date) => {
                dialog.calendar_model.set_selected_visible(date);
                let card_id = dialog.card_id;
                self.update_card(CardMessage::SetDueDate { card_id, date })
            }

            CardDetailsMessage::Close => {
                self.page = None;
                Task::none()
            }
        }
    }

    pub fn view_card_detail<'a>(&'a self, dialog: &'a CardDetailDialog) -> Element<'a, Message> {
        let Spacing {
            space_xxs,
            space_xs,
            space_s,
            space_m,
            ..
        } = cosmic::theme::spacing();

        let Some(board) = self.active_board() else {
            return widget::column![].into();
        };
        let Some((list, card)) = board.lists.iter().find_map(|l| {
            l.cards
                .iter()
                .find(|c| c.id == dialog.card_id)
                .map(|c| (l, c))
        }) else {
            return widget::column![].into();
        };

        let card_id = card.id;
        let wrap = |m: CardDetailsMessage| Message::CardDetailsDialog(m);
        let wrap_card = |m: CardMessage| Message::CardDetailsDialog(CardDetailsMessage::Card(m));

        let title_input = widget::text_input("Card title…", card.title.as_str())
            .on_input(move |new_title| wrap_card(CardMessage::UpdateTitle { card_id, new_title }))
            .width(Length::Fill);

        let in_list = widget::text::caption(format!("{}: {}", fl!("in-list"), list.title));

        let due_date_text = match card.due_date {
            Some(d) => format!("📅 {d}"),
            None => fl!("no-due-date"),
        };
        let mut due_row = widget::row::with_capacity(3)
            .spacing(space_xs)
            .align_y(Alignment::Center);
        due_row = due_row.push(widget::text::body(due_date_text));
        if card.due_date.is_some() {
            due_row = due_row.push(
                widget::button::text(fl!("clear-date"))
                    .on_press(wrap_card(CardMessage::ClearDueDate(card_id))),
            );
        }
        let toggle_label = if dialog.calendar_visible {
            fl!("hide-date-picker")
        } else {
            fl!("pick-date")
        };
        due_row = due_row.push(
            widget::button::text(toggle_label).on_press(wrap(CardDetailsMessage::ToggleCalendar)),
        );

        let calendar_section: Option<Element<'_, Message>> = if dialog.calendar_visible {
            let year = dialog.calendar_model.visible.year() as i32;
            let year_spin = widget::spin_button(
                year.to_string(),
                "Year",
                year,
                1i32,
                1900i32,
                2200i32,
                move |y| wrap(CardDetailsMessage::CalendarYearChanged(y)),
            );
            let year_row = widget::row::with_capacity(2)
                .push(widget::text::body("Year:"))
                .push(year_spin)
                .spacing(space_s)
                .align_y(Alignment::Center);

            let cal = widget::calendar::calendar(
                &dialog.calendar_model,
                move |date| wrap(CardDetailsMessage::CalendarDateSelected(date)),
                move || wrap(CardDetailsMessage::CalendarPrevMonth),
                move || wrap(CardDetailsMessage::CalendarNextMonth),
                jiff::civil::Weekday::Monday,
            );

            Some(
                widget::column::with_capacity(2)
                    .push(year_row)
                    .push(cal)
                    .spacing(space_s)
                    .into(),
            )
        } else {
            None
        };

        let mut due_section =
            widget::settings::section().add(widget::settings::item::item(fl!("due-date"), due_row));
        if let Some(cal) = calendar_section {
            due_section = due_section.add(cal);
        }

        let accent_swatch = color_button(
            Some(Message::OpenAccentColorDialog(card_id)),
            card.accent_color,
            Length::Fixed(32.0),
        );
        let accent_section = widget::settings::section().add(widget::settings::item::item(
            fl!("accent-color"),
            accent_swatch,
        ));

        let mut tag_items: Vec<Element<'_, Message>> = Vec::with_capacity(board.tags.len());
        for tag in &board.tags {
            let tag_id = tag.id;
            let active = card.tag_ids.contains(&tag_id);
            let chip = tag_chip(
                tag,
                active,
                wrap_card(CardMessage::ToggleTag { card_id, tag_id }),
            );
            let edit_btn = widget::button::icon(widget::icon::from_name("edit-symbolic"))
                .icon_size(14)
                .on_press(Message::OpenTagDialog(Some(tag_id)));
            tag_items.push(
                widget::row::with_capacity(2)
                    .push(chip)
                    .push(edit_btn)
                    .align_y(Alignment::Center)
                    .spacing(space_xxs)
                    .into(),
            );
        }
        let new_tag_btn = widget::button::icon(widget::icon::from_name("list-add-symbolic"))
            .on_press(Message::OpenTagDialog(None));

        let tags_section = widget::settings::section()
            .add(widget::settings::item::item(fl!("tags"), new_tag_btn))
            .add(cosmic::widget::flex_row(tag_items).spacing(space_xs));

        let desc_input = widget::text_input("Add a description…", card.description.as_str())
            .on_input(move |new_description| {
                wrap_card(CardMessage::UpdateDescription {
                    card_id,
                    new_description,
                })
            })
            .width(Length::Fill);
        let description_section = widget::settings::section()
            .add(widget::settings::item::item(fl!("description"), desc_input));

        let total = card.checklist.len();
        let done = card.checklist.iter().filter(|i| i.completed).count();
        let checklist_title = if total > 0 {
            format!("{} ({done}/{total})", fl!("checklist"))
        } else {
            fl!("checklist")
        };

        let mut checklist_section = widget::settings::section().title(checklist_title);
        for item in &card.checklist {
            let item_id = item.id;
            let check = widget::checkbox(item.completed)
                .label(item.text.as_str())
                .on_toggle(move |_: bool| {
                    wrap_card(CardMessage::Checklist(ChecklistMessage::Toggle {
                        card_id,
                        item_id,
                    }))
                })
                .width(Length::Fill);
            let del_btn = widget::button::icon(widget::icon::from_name("user-trash-symbolic"))
                .on_press(wrap_card(CardMessage::Checklist(
                    ChecklistMessage::Delete { card_id, item_id },
                )));
            checklist_section = checklist_section.add(widget::settings::item::item_row(vec![
                check.into(),
                del_btn.into(),
            ]));
        }
        let add_item_input = widget::text_input("New item…", &dialog.checklist_input)
            .on_input(move |text| wrap(CardDetailsMessage::ChecklistInputChanged(text)))
            .on_submit(move |_| wrap(CardDetailsMessage::ConfirmChecklistItem))
            .width(Length::Fill);
        let add_item_btn = widget::button::suggested(fl!("add-item"))
            .on_press(wrap(CardDetailsMessage::ConfirmChecklistItem));
        checklist_section = checklist_section.add(widget::settings::item::item_row(vec![
            add_item_input.into(),
            add_item_btn.into(),
        ]));

        let card_idx = list.cards.iter().position(|c| c.id == card.id);
        let list_len = list.cards.len();

        let mut order_items: Vec<Element<'_, Message>> = Vec::new();
        if card_idx.map(|i| i > 0).unwrap_or(false) {
            order_items.push(
                widget::button::text(fl!("move-up"))
                    .on_press(wrap_card(CardMessage::MoveUp(card_id)))
                    .into(),
            );
        }
        if card_idx.map(|i| i + 1 < list_len).unwrap_or(false) {
            order_items.push(
                widget::button::text(fl!("move-down"))
                    .on_press(wrap_card(CardMessage::MoveDown(card_id)))
                    .into(),
            );
        }

        let other_lists: Vec<_> = board.lists.iter().filter(|l| l.id != list.id).collect();

        let move_section: Option<Element<'_, Message>> = if !other_lists.is_empty()
            || !order_items.is_empty()
        {
            let mut section = widget::settings::section().title(fl!("move-card"));

            if !other_lists.is_empty() {
                let mut move_row = widget::row::with_capacity(other_lists.len()).spacing(space_xs);
                for target_list in &other_lists {
                    let target_list_id = target_list.id;
                    move_row =
                        move_row.push(widget::button::text(target_list.title.as_str()).on_press(
                            wrap_card(CardMessage::MoveToList {
                                card_id,
                                target_list_id,
                            }),
                        ));
                }
                section = section.add(move_row);
            }

            if !order_items.is_empty() {
                section = section.add(widget::row::with_children(order_items).spacing(space_xs));
            }

            Some(section.into())
        } else {
            None
        };

        let delete_btn = widget::button::destructive(fl!("delete-card"))
            .on_press(wrap_card(CardMessage::Delete(card_id)));

        let mut sections: Vec<Element<'_, Message>> = vec![
            due_section.into(),
            accent_section.into(),
            tags_section.into(),
            description_section.into(),
            checklist_section.into(),
        ];
        if let Some(mv) = move_section {
            sections.push(mv);
        }

        let content = widget::settings::view_column(sections);

        let col = widget::column::with_capacity(4)
            .push(title_input)
            .push(in_list)
            .push(content)
            .push(delete_btn)
            .spacing(space_s)
            .padding(space_m);

        let close_btn =
            widget::button::suggested(fl!("done")).on_press(wrap(CardDetailsMessage::Close));

        widget::dialog()
            .title(card.title.as_str())
            .control(widget::scrollable::vertical(col).height(Length::Fixed(480.0)))
            .primary_action(close_btn)
            .into()
    }
}

pub struct AccentColorDialog {
    pub card_id: Uuid,
    pub color_picker: ColorPickerModel,
}

#[derive(Debug, Clone)]
pub enum AccentColorDialogMessage {
    ColorPicker(ColorPickerUpdate),
    Clear,
    Close,
}

impl AccentColorDialog {
    pub fn new(card_id: Uuid, current: Option<Color>) -> Self {
        Self {
            card_id,
            color_picker: ColorPickerModel::new("Hex", "RGB", None, current),
        }
    }

    pub fn view(&self) -> Element<'_, AccentColorDialogMessage> {
        let picker = self
            .color_picker
            .builder(AccentColorDialogMessage::ColorPicker)
            .build(fl!("recent-colors"), fl!("copy-color"), fl!("copied-color"));

        let clear_btn = widget::button::destructive(fl!("remove-color"))
            .on_press(AccentColorDialogMessage::Clear);
        let done_btn =
            widget::button::suggested(fl!("done")).on_press(AccentColorDialogMessage::Close);

        let col = widget::column::with_capacity(2)
            .push(picker)
            .push(clear_btn)
            .spacing(cosmic::theme::spacing().space_s);

        widget::dialog()
            .title(fl!("accent-color"))
            .control(col)
            .primary_action(done_btn)
            .into()
    }
}

impl AppModel {
    pub fn update_accent_color_dialog(
        &mut self,
        message: AccentColorDialogMessage,
    ) -> Task<cosmic::Action<Message>> {
        match message {
            AccentColorDialogMessage::ColorPicker(update) => {
                let Some(dialog) = &mut self.accent_color_dialog else {
                    return Task::none();
                };
                let card_id = dialog.card_id;
                let before = dialog.color_picker.get_applied_color();
                let picker_task = dialog.color_picker.update(update);
                let after = dialog.color_picker.get_applied_color();
                if after != before
                    && let Some(color) = after
                {
                    let apply_task = self.update_card(CardMessage::SetAccentColor {
                        card_id,
                        color: Some(color),
                    });
                    return Task::batch([picker_task, apply_task]);
                }
                picker_task
            }

            AccentColorDialogMessage::Clear => {
                if let Some(dialog) = self.accent_color_dialog.take() {
                    return self.update_card(CardMessage::SetAccentColor {
                        card_id: dialog.card_id,
                        color: None,
                    });
                }
                Task::none()
            }

            AccentColorDialogMessage::Close => {
                self.accent_color_dialog = None;
                Task::none()
            }
        }
    }
}
