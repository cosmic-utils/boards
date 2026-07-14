// SPDX-License-Identifier: GPL-3.0

pub mod context;
pub mod context_menu;
pub mod dialog;
pub mod message;
pub mod view;

use crate::column::Column;
use crate::tag::Tag;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    pub id: Uuid,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_background")]
    pub background: cosmic::iced::Color,
    pub columns: Vec<Column>,
    #[serde(default)]
    pub tags: Vec<Tag>,
    #[serde(default = "default_board_icon")]
    pub icon: String,
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}

impl Board {
    pub fn new(title: String) -> Self {
        let now = jiff::Timestamp::now();
        Self {
            id: Uuid::new_v4(),
            title,
            description: String::new(),
            background: default_background(),
            columns: Vec::new(),
            tags: Vec::new(),
            icon: default_board_icon(),
            created_at: now,
            updated_at: now,
        }
    }
}

fn default_background() -> cosmic::iced::Color {
    cosmic::iced::Color::from_rgb8(54, 95, 168)
}

pub fn default_board_icon() -> String {
    "view-grid-symbolic".to_string()
}
