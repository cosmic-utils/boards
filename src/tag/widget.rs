// SPDX-License-Identifier: GPL-3.0

use cosmic::iced::widget::container::Style as ContainerStyle;
use cosmic::iced::{Background, Border, Color};
use cosmic::prelude::*;
use cosmic::widget;

use crate::app::Message;
use crate::tag::Tag;

pub fn tag_badge<'a>(tag: &Tag) -> Element<'a, Message> {
    let space_xxs = cosmic::theme::spacing().space_xxs;
    let space_xs = cosmic::theme::spacing().space_xs;
    let name = tag.name.clone();
    let color = tag.color;

    widget::container(widget::text::caption(name).class(Color::WHITE))
        .padding([space_xxs, space_xs])
        .class(cosmic::theme::Container::custom(move |_theme| {
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

pub fn tag_chip<'a>(tag: &Tag, active: bool, on_press: Message) -> Element<'a, Message> {
    let space_xxs = cosmic::theme::spacing().space_xxs;
    let space_xs = cosmic::theme::spacing().space_xs;
    let name = tag.name.clone();
    let color = tag.color;
    let border_width = if active { 2.0 } else { 0.0 };

    let style = move |_focused: bool, _theme: &cosmic::Theme| widget::button::Style {
        background: Some(Background::Color(color)),
        border_radius: 4.0.into(),
        border_width,
        border_color: Color::WHITE,
        ..Default::default()
    };

    widget::button::custom(widget::text::caption(name).class(Color::WHITE))
        .padding([space_xxs, space_xs])
        .class(cosmic::theme::Button::Custom {
            active: Box::new(style),
            disabled: Box::new(move |theme| style(false, theme)),
            hovered: Box::new(style),
            pressed: Box::new(style),
        })
        .on_press(on_press)
        .into()
}
