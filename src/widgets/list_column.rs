// SPDX-License-Identifier: GPL-3.0

use cosmic::iced::{Alignment, Border, Color, Length};
use cosmic::prelude::*;
use cosmic::widget;
use uuid::Uuid;

use crate::app::Message;
use crate::models::list::List;
use crate::widgets::card_widget::view_card;
use crate::widgets::dnd::CardDragData;
use crate::widgets::new_card_input::new_card_input;

// ── Append / empty-list drop zone ────────────────────────────────────────────

/// Height of the trailing append zone rendered after the last card.
const TAIL_HEIGHT: f32 = 48.0;

/// Height of the drop zone shown when a list is completely empty.
const EMPTY_HEIGHT: f32 = 100.0;

/// Visual element for the append / empty-list drop zone.
///
/// When `active` renders a 2 px accent-coloured horizontal line to signal
/// "drop here to append", inspired by cosmic-files' DnD indicator style.
/// When inactive it is a transparent spacer.
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

/// A `dnd_destination_for_data` wrapping the append / empty-list visual.
/// Dropping here appends the dragged card to `list_id` (before_card_id = None).
fn append_drop_zone(list_id: Uuid, active: bool, height: f32) -> Element<'static, Message> {
    let visual = append_visual(active, height);

    cosmic::widget::dnd_destination::dnd_destination_for_data::<CardDragData, Message>(
        visual,
        move |data, _action| match data {
            Some(d) => Message::DragCardDropped {
                card_id: d.card_id,
                target_list_id: list_id,
                before_card_id: None,
            },
            None => Message::DragCardCancelled,
        },
    )
    .on_enter(move |_, _, _| Message::DragHoverChanged {
        list_id,
        before_card_id: None,
    })
    .on_leave(|| Message::DragLeftDropZone)
    .into()
}

// ── Column builders ───────────────────────────────────────────────────────────

/// Builds the scrollable card column.
///
/// * `drag_hovered_card_id` – the card in this list currently being hovered
///   during a drag (the highlighted drop target); `None` if no card is hovered.
/// * `tail_active`          – whether the append zone at the bottom is the
///   active drop target.
fn build_cards_column(
    list: &List,
    drag_hovered_card_id: Option<Uuid>,
    tail_active: bool,
) -> Element<'_, Message> {
    let space_xs = cosmic::theme::spacing().space_xs;
    let list_id = list.id;

    if list.cards.is_empty() {
        // Full-height append zone so the user can drop into an empty list.
        let zone = append_drop_zone(list_id, tail_active, EMPTY_HEIGHT);
        return widget::column::with_capacity(1)
            .push(zone)
            .padding([0, space_xs])
            .into();
    }

    // N cards + spacing + tail append zone.
    let capacity = list.cards.len() + 1;
    let mut col = widget::column::with_capacity(capacity)
        .spacing(space_xs)
        .padding([0, space_xs]);

    for card in &list.cards {
        let is_target = Some(card.id) == drag_hovered_card_id;
        col = col.push(view_card(card, is_target, list_id));
    }

    // Trailing append zone — taller so it is easy to hit after the last card.
    col = col.push(append_drop_zone(list_id, tail_active, TAIL_HEIGHT));

    col.into()
}

// ── Public view functions ─────────────────────────────────────────────────────

/// Renders a single Kanban column with a pinned header, scrollable card area,
/// and a pinned "Add a card" footer.
///
/// * `drag_hovered_card_id` – which card in this list the cursor is hovering
///   during a drag (`None` when no card in this list is hovered).
/// * `tail_active`          – whether the trailing append zone is the active
///   drop target.
pub fn view_list<'a>(
    list: &'a List,
    drag_hovered_card_id: Option<Uuid>,
    tail_active: bool,
) -> Element<'a, Message> {
    let space_xs = cosmic::theme::spacing().space_xs;
    let space_s = cosmic::theme::spacing().space_s;

    let list_id = list.id;

    // ── Pinned header ──
    let header = widget::row::with_capacity(2)
        .push(widget::text::title4(list.title.as_str()).width(Length::Fill))
        .push(
            widget::button::icon(widget::icon::from_name("list-add-symbolic"))
                .on_press(Message::OpenNewCardInput(list_id)),
        )
        .align_y(Alignment::Center)
        .spacing(space_xs)
        .padding([space_xs, space_s]);

    let cards_area = build_cards_column(list, drag_hovered_card_id, tail_active);
    let scrollable_body = widget::scrollable::vertical(cards_area).height(Length::Fill);

    let footer = widget::button::text("+ Add a card")
        .width(Length::Fill)
        .on_press(Message::OpenNewCardInput(list_id));
    let footer_container = widget::container(footer).padding([space_xs, space_s]);

    let inner_col = widget::column::with_capacity(5)
        .push(header)
        .push(widget::divider::horizontal::default())
        .push(scrollable_body)
        .push(widget::divider::horizontal::default())
        .push(footer_container);

    widget::container(inner_col)
        .width(280)
        .height(Length::Fill)
        .class(cosmic::theme::Container::Secondary)
        .into()
}

/// Renders a list column where the new-card inline input is open.
///
/// * `drag_hovered_card_id` – which card in this list the cursor is hovering
///   during a drag (`None` when no card in this list is hovered).
/// * `tail_active`          – whether the trailing append zone is the active
///   drop target.
pub fn view_list_with_input<'a>(
    list: &'a List,
    input_text: &'a str,
    drag_hovered_card_id: Option<Uuid>,
    tail_active: bool,
) -> Element<'a, Message> {
    let space_xs = cosmic::theme::spacing().space_xs;
    let space_s = cosmic::theme::spacing().space_s;

    let list_id = list.id;

    let header = widget::row::with_capacity(2)
        .push(widget::text::title4(list.title.as_str()).width(Length::Fill))
        .push(
            widget::button::icon(widget::icon::from_name("list-add-symbolic"))
                .on_press(Message::OpenNewCardInput(list_id)),
        )
        .align_y(Alignment::Center)
        .spacing(space_xs)
        .padding([space_xs, space_s]);

    let cards_area = build_cards_column(list, drag_hovered_card_id, tail_active);
    let scrollable_body = widget::scrollable::vertical(cards_area).height(Length::Fill);

    // Footer shows the new-card input instead of the add button.
    let footer_content = new_card_input(list_id, input_text);
    let footer_container = widget::container(footer_content).padding([space_xs, space_s]);

    let inner_col = widget::column::with_capacity(5)
        .push(header)
        .push(widget::divider::horizontal::default())
        .push(scrollable_body)
        .push(widget::divider::horizontal::default())
        .push(footer_container);

    widget::container(inner_col)
        .width(280)
        .height(Length::Fill)
        .class(cosmic::theme::Container::Secondary)
        .into()
}
