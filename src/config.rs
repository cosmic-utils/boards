// SPDX-License-Identifier: GPL-3.0

use cosmic::cosmic_config::{self, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry};

use uuid::Uuid;

#[derive(Debug, Default, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 1]
pub struct Config {
    /// The ID of the board that was active when the app was last closed.
    pub last_board_id: Option<Uuid>,
}
