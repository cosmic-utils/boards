// SPDX-License-Identifier: GPL-3.0

pub mod dialog;
pub mod message;
pub mod widget;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub color: cosmic::iced::Color,
}

impl Tag {
    pub fn new(name: String, color: cosmic::iced::Color) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            color,
        }
    }
}

pub fn default_tag_color() -> cosmic::iced::Color {
    cosmic::iced::Color::from_rgb8(59, 130, 246)
}
