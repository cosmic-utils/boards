// SPDX-License-Identifier: GPL-3.0

use cosmic::iced::mouse::Interaction;
use cosmic::iced::widget::container::Style as ContainerStyle;
use cosmic::iced::{Alignment, Background, Border, Color, Length};
use cosmic::prelude::*;
use cosmic::widget;
use jiff::ToSpan;
use uuid::Uuid;

use crate::app::Message;
use crate::board::context::BoardContext;
use crate::card::Card;
use crate::card::context_menu::card_context_menu;
use crate::dnd::CardDragData;
use crate::dnd::message::DndMessage;
use crate::tag::widget::tag_badge;

#[derive(Clone, Copy)]
enum BadgeKind {
    Danger,
    Warning,
    Success,
}

fn semantic_badge<'a>(text: String, kind: BadgeKind) -> Element<'a, Message> {
    let space_xxs = cosmic::theme::spacing().space_xxs;

    widget::container(widget::text::caption(text).class(Color::WHITE))
        .padding([0, space_xxs])
        .class(cosmic::theme::Container::custom(move |theme| {
            let palette = theme.cosmic();
            let color = match kind {
                BadgeKind::Danger => Color::from(palette.destructive_color()),
                BadgeKind::Warning => Color::from(palette.warning_color()),
                BadgeKind::Success => Color::from(palette.success_color()),
            };
            ContainerStyle {
                background: Some(Background::Color(color)),
                border: Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }))
        .into()
}

pub fn view_card<'a>(
    card: &'a Card,
    is_drop_target: bool,
    list_id: Uuid,
    ctx: &BoardContext<'_>,
) -> Element<'a, Message> {
    let space_xxs = cosmic::theme::spacing().space_xxs;
    let space_s = cosmic::theme::spacing().space_s;

    let tags: Vec<&crate::tag::Tag> = card
        .tag_ids
        .iter()
        .filter_map(|id| ctx.tags.iter().find(|t| t.id == *id))
        .collect();
    let mut tags_row = widget::row::with_capacity(tags.len().min(3) + 1).spacing(space_xxs);
    for tag in tags.iter().take(3) {
        tags_row = tags_row.push(tag_badge(tag));
    }
    if tags.len() > 3 {
        let extra = tags.len() - 3;
        tags_row = tags_row.push(widget::text::caption(format!("+{extra}")));
    }

    let today = jiff::Zoned::now().date();
    let due_meta = card.due_date.map(|due| {
        let text = format!("📅 {due}");
        if due < today {
            semantic_badge(text, BadgeKind::Danger)
        } else if due <= today.saturating_add(2.days()) {
            semantic_badge(text, BadgeKind::Warning)
        } else {
            widget::text::caption(text).into()
        }
    });

    let total = card.checklist.len();
    let done = card.checklist.iter().filter(|i| i.completed).count();
    let checklist_meta = (total > 0).then(|| {
        let text = format!("☐ {done}/{total}");
        if done == total {
            semantic_badge(text, BadgeKind::Success)
        } else {
            widget::text::caption(text).into()
        }
    });

    let mut meta_row = widget::row::with_capacity(2)
        .spacing(space_xxs)
        .align_y(Alignment::Center);
    let mut has_meta = false;
    if let Some(due) = due_meta {
        meta_row = meta_row.push(due);
        has_meta = true;
    }
    if let Some(checklist) = checklist_meta {
        meta_row = meta_row.push(checklist);
        has_meta = true;
    }

    let description_preview = (!card.description.is_empty()).then(|| {
        card.description
            .lines()
            .take(3)
            .collect::<Vec<_>>()
            .join("\n")
    });

    let mut body = widget::column::with_capacity(4)
        .spacing(space_xxs)
        .padding([space_s, space_s]);

    body = body.push(widget::text::heading(card.title.as_str()));
    if !tags.is_empty() {
        body = body.push(tags_row);
    }
    if let Some(preview) = description_preview {
        body = body.push(widget::text::caption(preview));
    }
    if has_meta {
        body = body.push(meta_row);
    }

    let content: Element<'a, Message> = if let Some(accent) = card.accent_color {
        let stripe = widget::container(widget::Space::new())
            .width(Length::Fixed(4.0))
            .height(Length::Fill)
            .class(cosmic::theme::Container::custom(move |_theme| {
                ContainerStyle {
                    background: Some(Background::Color(accent)),
                    ..Default::default()
                }
            }));
        widget::row::with_capacity(2).push(stripe).push(body).into()
    } else {
        body.into()
    };

    let card_id = card.id;

    let card_container: Element<'a, Message> = if is_drop_target {
        widget::container(content)
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
        widget::container(content)
            .class(cosmic::theme::Container::Card)
            .width(Length::Fill)
            .into()
    };

    let clickable = widget::mouse_area(card_container)
        .interaction(Interaction::Grab)
        .on_release(Message::CardClickReleased(card_id));

    let source = widget::dnd_source(clickable)
        .drag_content(move || CardDragData { card_id })
        .on_start(Some(Message::Dnd(DndMessage::CardStarted(card_id))))
        .on_cancel(Some(Message::Dnd(DndMessage::CardCancelled)))
        .on_finish(Some(Message::Dnd(DndMessage::CardCancelled)));

    let draggable: Element<'a, Message> =
        cosmic::widget::dnd_destination::dnd_destination_for_data::<CardDragData, Message>(
            source,
            move |data, _action| match data {
                Some(d) => Message::Dnd(DndMessage::CardDropped {
                    card_id: d.card_id,
                    target_list_id: list_id,
                    before_card_id: Some(card_id),
                }),
                None => Message::Dnd(DndMessage::CardCancelled),
            },
        )
        .on_enter(move |_, _, _| {
            Message::Dnd(DndMessage::HoverChanged {
                list_id,
                before_card_id: Some(card_id),
            })
        })
        .on_leave(|| Message::Dnd(DndMessage::LeftDropZone))
        .into();

    cosmic::widget::context_menu(draggable, card_context_menu(card, ctx)).into()
}
