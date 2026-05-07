// SPDX-License-Identifier: GPL-3.0

use cosmic::iced::widget::container::Style as ContainerStyle;
use cosmic::iced::{Background, Border, Color};
use cosmic::prelude::*;
use cosmic::widget;

use crate::app::Message;
use crate::models::label::Label;

/// Renders a small colored pill badge for a label.
pub fn label_badge<'a>(label: &'a Label) -> Element<'a, Message> {
    let color = label.color.as_color();
    let space_xxs = cosmic::theme::spacing().space_xxs;
    let space_xs = cosmic::theme::spacing().space_xs;

    widget::container(widget::text::caption(label.name.as_str()).class(Color::WHITE))
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
