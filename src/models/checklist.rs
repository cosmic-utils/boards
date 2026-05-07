// SPDX-License-Identifier: GPL-3.0

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub id: Uuid,
    pub text: String,
    pub completed: bool,
    pub position: u32,
}
