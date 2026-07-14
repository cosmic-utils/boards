// SPDX-License-Identifier: GPL-3.0

pub mod context_menu;
pub mod dialog;
pub mod message;
pub mod widget;

use crate::card::Card;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub id: Uuid,
    pub title: String,
    pub cards: Vec<Card>,
    pub position: u32,
}

impl Column {
    pub fn new(title: String, position: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            cards: Vec::new(),
            position,
        }
    }
}
