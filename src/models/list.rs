// SPDX-License-Identifier: GPL-3.0

use super::card::Card;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct List {
    pub id: Uuid,
    pub title: String,
    /// Ordered list of cards
    pub cards: Vec<Card>,
    /// Sort position (for ordering columns)
    pub position: u32,
}
