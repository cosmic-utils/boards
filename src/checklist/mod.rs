// SPDX-License-Identifier: GPL-3.0

pub mod message;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub id: Uuid,
    pub text: String,
    pub completed: bool,
    pub position: u32,
}

impl ChecklistItem {
    pub fn new(text: String, position: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            text,
            completed: false,
            position,
        }
    }
}
