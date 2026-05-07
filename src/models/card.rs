// SPDX-License-Identifier: GPL-3.0

use super::{checklist::ChecklistItem, label::Label};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: Uuid,
    pub title: String,
    /// Markdown-formatted description
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub labels: Vec<Label>,
    #[serde(default)]
    pub checklist: Vec<ChecklistItem>,
    /// Civil (timezone-free) date
    pub due_date: Option<jiff::civil::Date>,
    /// Sort position within its list
    pub position: u32,
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}
