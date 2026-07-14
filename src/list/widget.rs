// SPDX-License-Identifier: GPL-3.0

use cosmic::cosmic_theme::Spacing;
use cosmic::iced::{Alignment, Border, Color, Length};
use cosmic::prelude::*;
use cosmic::widget;
use uuid::Uuid;

use crate::app::Message;
use crate::board::context::BoardContext;
use crate::card::widget::view_card;
use crate::dnd::CardDragData;
use crate::dnd::message::DndMessage;
use crate::list::List;
use crate::list::context_menu::list_context_menu;
use crate::list::message::ListMessage;

const TAIL_HEIGHT: f32 = 48.0;
const EMPTY_HEIGHT: f32 = 100.0;

fn append_visual(active: bool, height: f32) -> Element<'static, Message> {
    if active {
        let line = widget::container(widget::Space::new())
            .width(Length::Fill)
            .height(Length::Fixed(2.0))
            .style(|t: &cosmic::Theme| {
                let accent = Color::from(t.cosmic().accent_color());
                cosmic::iced::widget::container::Style {
                    background: Some(accent.into()),
                    border: Border {
                        color: accent,
                        width: 0.0,
                        radius: 1.0.into(),
                    },
                    ..Default::default()
                }
            });

        widget::container(line)
            .padding([(height as u16 - 2) / 2, 4])
            .width(Length::Fill)
            .into()
    } else {
        widget::container(widget::Space::new())
            .width(Length::Fill)
            .height(Length::Fixed(height))
            .into()
    }
}

fn append_drop_zone(list_id: Uuid, active: bool, height: f32) -> Element<'static, Message> {
    let visual = append_visual(active, height);

    cosmic::widget::dnd_destination::dnd_destination_for_data::<CardDragData, Message>(
        visual,
        move |data, _action| match data {
            Some(d) => Message::Dnd(DndMessage::CardDropped {
                card_id: d.card_id,
                target_list_id: list_id,
                before_card_id: None,
            }),
            None => Message::Dnd(DndMessage::CardCancelled),
        },
    )
    .on_enter(move |_, _, _| {
        Message::Dnd(DndMessage::HoverChanged {
            list_id,
            before_card_id: None,
        })
    })
    .on_leave(|| Message::Dnd(DndMessage::LeftDropZone))
    .into()
}

fn new_card_input<'a>(input_id: &widget::Id, list_id: Uuid, text: &'a str) -> Element<'a, Message> {
    let space_xs = cosmic::theme::spacing().space_xs;

    let input = widget::text_input("Card title…", text)
        .id(input_id.clone())
        .on_input(|title| Message::List(ListMessage::NewCardInputChanged(title)))
        .on_submit(move |_| Message::List(ListMessage::ConfirmNewCard(list_id)))
        .width(Length::Fill);

    let confirm_btn = widget::button::suggested("Add")
        .on_press(Message::List(ListMessage::ConfirmNewCard(list_id)));

    let cancel_btn =
        widget::button::text("Cancel").on_press(Message::List(ListMessage::DismissNewInput));

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

fn build_header(list: &List, editing_title: bool) -> Element<'_, Message> {
    let Spacing {
        space_xs,
        space_xxs,
        ..
    } = cosmic::theme::spacing();

    let list_id = list.id;

    let title_input = widget::editable_input(
        "List title…",
        list.title.as_str(),
        editing_title,
        move |_| Message::List(ListMessage::ToggleTitleEdit(list_id)),
    )
    .on_input(move |new_title| Message::List(ListMessage::Rename { list_id, new_title }))
    .width(Length::Fill);

    let row = widget::row::with_capacity(2)
        .push(title_input)
        .push(
            widget::button::icon(widget::icon::from_name("list-add-symbolic")).on_press(
                Message::List(ListMessage::OpenNewCardInput(widget::Id::unique(), list_id)),
            ),
        )
        .align_y(Alignment::Center)
        .spacing(space_xs)
        .padding([space_xxs, space_xs]);

    cosmic::widget::context_menu(row, list_context_menu(list_id)).into()
}

fn build_cards_column<'a>(
    list: &'a List,
    ctx: &BoardContext<'_>,
    drag_hovered_card_id: Option<Uuid>,
    tail_active: bool,
) -> Element<'a, Message> {
    let space_xs = cosmic::theme::spacing().space_xs;
    let list_id = list.id;

    let visible: Vec<&crate::card::Card> = list
        .cards
        .iter()
        .filter(|c| ctx.matches(&c.title, &c.tag_ids))
        .collect();

    if visible.is_empty() {
        let zone = append_drop_zone(list_id, tail_active, EMPTY_HEIGHT);
        return widget::column::with_capacity(1)
            .push(zone)
            .padding([0, space_xs])
            .into();
    }

    let capacity = visible.len() + 1;
    let mut col = widget::column::with_capacity(capacity)
        .spacing(space_xs)
        .padding([0, space_xs]);

    for card in visible {
        let is_target = Some(card.id) == drag_hovered_card_id;
        col = col.push(view_card(card, is_target, list_id, ctx));
    }

    col = col.push(append_drop_zone(list_id, tail_active, TAIL_HEIGHT));

    col.into()
}

pub fn view_list<'a>(
    list: &'a List,
    editing_title: bool,
    ctx: &BoardContext<'_>,
    drag_hovered_card_id: Option<Uuid>,
    tail_active: bool,
) -> Element<'a, Message> {
    let header = build_header(list, editing_title);

    let cards_area = build_cards_column(list, ctx, drag_hovered_card_id, tail_active);
    let scrollable_body = widget::scrollable::vertical(cards_area).height(Length::Fill);

    let inner_col = widget::column::with_capacity(5)
        .push(header)
        .push(widget::divider::horizontal::default())
        .push(widget::space::vertical().height(Length::Fixed(8.)))
        .push(scrollable_body);

    widget::container(inner_col)
        .width(280)
        .height(Length::Fill)
        .class(cosmic::theme::Container::Secondary)
        .into()
}

pub fn view_list_with_input<'a>(
    list: &'a List,
    editing_title: bool,
    ctx: &BoardContext<'_>,
    input_text: &'a str,
    input_id: &widget::Id,
    drag_hovered_card_id: Option<Uuid>,
    tail_active: bool,
) -> Element<'a, Message> {
    let space_xs = cosmic::theme::spacing().space_xs;
    let space_s = cosmic::theme::spacing().space_s;

    let list_id = list.id;
    let header = build_header(list, editing_title);

    let cards_area = build_cards_column(list, ctx, drag_hovered_card_id, tail_active);
    let scrollable_body = widget::scrollable::vertical(cards_area).height(Length::Fill);

    let footer_content = new_card_input(input_id, list_id, input_text);
    let footer_container = widget::container(footer_content).padding([space_xs, space_s]);

    let inner_col = widget::column::with_capacity(5)
        .push(header)
        .push(widget::divider::horizontal::default())
        .push(widget::space::vertical().height(Length::Fixed(8.)))
        .push(scrollable_body)
        .push(widget::divider::horizontal::default())
        .push(footer_container);

    widget::container(inner_col)
        .width(280)
        .height(Length::Fill)
        .class(cosmic::theme::Container::Secondary)
        .into()
}
