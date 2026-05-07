// SPDX-License-Identifier: GPL-3.0

pub mod store;

use crate::models::board::Board;
use uuid::Uuid;

pub trait DataStore {
    type Error: std::error::Error;

    /// Load all board summaries (id + title + color only)
    fn load_board_index(&self) -> Result<Vec<BoardSummary>, Self::Error>;

    /// Load a full board with all its lists and cards
    fn load_board(&self, id: Uuid) -> Result<Board, Self::Error>;

    /// Persist a full board
    fn save_board(&self, board: &Board) -> Result<(), Self::Error>;

    /// Delete a board and its data file
    fn delete_board(&self, id: Uuid) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BoardSummary {
    pub id: Uuid,
    pub title: String,
    pub background: cosmic::iced::Color,
    pub list_count: usize,
    pub card_count: usize,
}
