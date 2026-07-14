// SPDX-License-Identifier: GPL-3.0

pub mod context_menu;
pub mod dialog;
pub mod message;
pub mod widget;

use crate::checklist::ChecklistItem;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: Uuid,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tag_ids: Vec<Uuid>,
    #[serde(default)]
    pub checklist: Vec<ChecklistItem>,
    pub due_date: Option<jiff::civil::Date>,
    #[serde(default)]
    pub accent_color: Option<cosmic::iced::Color>,
    pub position: u32,
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}

impl Card {
    pub fn new(title: String, position: u32) -> Self {
        let now = jiff::Timestamp::now();
        Self {
            id: Uuid::new_v4(),
            title,
            description: String::new(),
            tag_ids: Vec::new(),
            checklist: Vec::new(),
            due_date: None,
            accent_color: None,
            position,
            created_at: now,
            updated_at: now,
        }
    }
}
