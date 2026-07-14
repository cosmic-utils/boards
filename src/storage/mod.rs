// SPDX-License-Identifier: GPL-3.0

pub mod store;

use crate::board::Board;
use uuid::Uuid;

pub trait DataStore {
    type Error: std::error::Error;

    fn load_board_index(&self) -> Result<Vec<BoardSummary>, Self::Error>;

    fn load_board(&self, id: Uuid) -> Result<Board, Self::Error>;

    fn save_board(&self, board: &Board) -> Result<(), Self::Error>;

    fn delete_board(&self, id: Uuid) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BoardSummary {
    pub id: Uuid,
    pub title: String,
    pub background: cosmic::iced::Color,
    #[serde(default = "crate::board::default_board_icon")]
    pub icon: String,
    pub column_count: usize,
    pub card_count: usize,
}
