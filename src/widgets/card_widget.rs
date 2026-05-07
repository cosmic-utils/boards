// SPDX-License-Identifier: GPL-3.0

use cosmic::iced::{Border, Length};
use cosmic::prelude::*;
use cosmic::widget;
use uuid::Uuid;

use crate::app::Message;
use crate::models::card::Card;
use crate::widgets::dnd::CardDragData;
use crate::widgets::label_badge::label_badge;

/// Renders a single card as a clickable, draggable, and droppable container.
///
/// * `card`           – card data to render
/// * `is_drop_target` – when `true` the whole card is highlighted with the
///                      accent colour to signal it is the active DnD destination
/// * `list_id`        – the list that owns this card (needed for DnD messages)
pub fn view_card<'a>(card: &'a Card, is_drop_target: bool, list_id: Uuid) -> Element<'a, Message> {
    let space_xxs = cosmic::theme::spacing().space_xxs;
    let space_s = cosmic::theme::spacing().space_s;

    // Labels row (show up to 3 badges)
    let mut labels_row =
        widget::row::with_capacity(card.labels.len().min(3) + 1).spacing(space_xxs);
    for label in card.labels.iter().take(3) {
        labels_row = labels_row.push(label_badge(label));
    }
    if card.labels.len() > 3 {
        let extra = card.labels.len() - 3;
        labels_row = labels_row.push(widget::text::caption(format!("+{extra}")));
    }

    // Checklist progress
    let total = card.checklist.len();
    let done = card.checklist.iter().filter(|i| i.completed).count();
    let checklist_text = if total > 0 {
        Some(format!("☐ {done}/{total}"))
    } else {
        None
    };

    // Due date text
    let due_text = card.due_date.map(|d| format!("📅 {}", d));

    let mut body = widget::column::with_capacity(5)
        .spacing(space_xxs)
        .padding([space_s, space_s]);

    if !card.labels.is_empty() {
        body = body.push(labels_row);
    }
    body = body.push(widget::text::body(card.title.as_str()));
    if let Some(due) = due_text {
        body = body.push(widget::text::caption(due));
    }
    if let Some(progress) = checklist_text {
        body = body.push(widget::text::caption(progress));
    }

    let card_id = card.id;

    // The card container switches to an accent-colour background when it is
    // the active DnD drop target (matching the cosmic-files folder-hover style).
    let card_container: Element<'a, Message> = if is_drop_target {
        widget::container(body)
            .width(Length::Fill)
            .style(|t: &cosmic::Theme| {
                let accent = cosmic::iced::Color::from(t.cosmic().accent_color());
                let tint = cosmic::iced::Color { a: 0.15, ..accent };
                cosmic::iced::widget::container::Style {
                    background: Some(tint.into()),
                    border: Border {
                        color: accent,
                        width: 2.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                }
            })
            .into()
    } else {
        widget::container(body)
            .class(cosmic::theme::Container::Card)
            .width(Length::Fill)
            .into()
    };

    // mouse_area: clicking the card opens the detail pane (on_release so it
    // does not fire when the user begins a drag).
    let clickable = widget::mouse_area(card_container).on_release(Message::OpenCardDetail(card_id));

    // dnd_source: initiates the system drag gesture.
    let source = widget::dnd_source(clickable)
        .drag_content(move || CardDragData { card_id })
        .on_start(Some(Message::DragCardStarted(card_id)))
        .on_cancel(Some(Message::DragCardCancelled))
        .on_finish(Some(Message::DragCardCancelled));

    // dnd_destination_for_data: makes the whole card a drop target.
    // Dropping here inserts the dragged card *before* this card.
    cosmic::widget::dnd_destination::dnd_destination_for_data::<CardDragData, Message>(
        source,
        move |data, _action| match data {
            Some(d) => Message::DragCardDropped {
                card_id: d.card_id,
                target_list_id: list_id,
                before_card_id: Some(card_id),
            },
            None => Message::DragCardCancelled,
        },
    )
    .on_enter(move |_, _, _| Message::DragHoverChanged {
        list_id,
        before_card_id: Some(card_id),
    })
    .on_leave(|| Message::DragLeftDropZone)
    .into()
}
