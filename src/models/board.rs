// SPDX-License-Identifier: GPL-3.0

use super::list::List;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    pub id: Uuid,
    pub title: String,
    #[serde(default)]
    pub description: String,
    /// Board header background colour
    #[serde(default = "default_background")]
    pub background: cosmic::iced::Color,
    /// Ordered list of columns
    pub lists: Vec<List>,
    /// RFC 3339 UTC timestamp
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}

fn default_background() -> cosmic::iced::Color {
    cosmic::iced::Color::from_rgb8(54, 95, 168)
}
