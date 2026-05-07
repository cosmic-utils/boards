// SPDX-License-Identifier: GPL-3.0

use cosmic::iced::Length;
use cosmic::prelude::*;
use cosmic::widget;
use uuid::Uuid;

use crate::app::Message;

/// Renders an inline text input for creating a new card in a list.
pub fn new_card_input<'a>(list_id: Uuid, text: &'a str) -> Element<'a, Message> {
    let space_xs = cosmic::theme::spacing().space_xs;

    let input = widget::text_input("Card title…", text)
        .on_input(Message::NewCardInputChanged)
        .on_submit(move |_| Message::ConfirmNewCard(list_id))
        .width(Length::Fill);

    let confirm_btn = widget::button::suggested("Add").on_press(Message::ConfirmNewCard(list_id));

    let cancel_btn = widget::button::text("Cancel").on_press(Message::DismissNewInput);

    let buttons = widget::row::with_capacity(2)
        .push(confirm_btn)
        .push(cancel_btn)
        .spacing(space_xs);

    widget::column::with_capacity(2)
        .push(input)
        .push(buttons)
        .spacing(space_xs)
        .into()
}

/// Renders an inline text input for creating a new list.
pub fn new_list_input<'a>(text: &'a str) -> Element<'a, Message> {
    let space_xs = cosmic::theme::spacing().space_xs;

    let input = widget::text_input("List name…", text)
        .on_input(Message::NewListInputChanged)
        .on_submit(|_| Message::ConfirmNewList)
        .width(Length::Fill);

    let confirm_btn = widget::button::suggested("Add").on_press(Message::ConfirmNewList);

    let cancel_btn = widget::button::text("Cancel").on_press(Message::DismissNewInput);

    let buttons = widget::row::with_capacity(2)
        .push(confirm_btn)
        .push(cancel_btn)
        .spacing(space_xs);

    widget::column::with_capacity(2)
        .push(input)
        .push(buttons)
        .spacing(space_xs)
        .padding(space_xs)
        .into()
}
