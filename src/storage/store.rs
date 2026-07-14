// SPDX-License-Identifier: GPL-3.0

use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use uuid::Uuid;

use super::{BoardSummary, DataStore};
use crate::board::Board;

pub struct Store {
    data_dir: PathBuf,
}

#[derive(Debug)]
pub enum StoreError {
    Io(std::io::Error),
    TomlSer(toml::ser::Error),
    TomlDe(toml::de::Error),
    Persist(tempfile::PersistError),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::Io(e) => write!(f, "IO error: {e}"),
            StoreError::TomlSer(e) => write!(f, "TOML serialize error: {e}"),
            StoreError::TomlDe(e) => write!(f, "TOML deserialize error: {e}"),
            StoreError::Persist(e) => write!(f, "Persist error: {e}"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<std::io::Error> for StoreError {
    fn from(e: std::io::Error) -> Self {
        StoreError::Io(e)
    }
}
impl From<toml::ser::Error> for StoreError {
    fn from(e: toml::ser::Error) -> Self {
        StoreError::TomlSer(e)
    }
}
impl From<toml::de::Error> for StoreError {
    fn from(e: toml::de::Error) -> Self {
        StoreError::TomlDe(e)
    }
}
impl From<tempfile::PersistError> for StoreError {
    fn from(e: tempfile::PersistError) -> Self {
        StoreError::Persist(e)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct IndexFile {
    #[serde(default)]
    boards: Vec<BoardSummary>,
}

impl Store {
    pub fn new() -> Self {
        let data_dir = directories::ProjectDirs::from("dev", "edfloreshz", "Boards")
            .expect("could not determine project directories")
            .data_dir()
            .to_path_buf();
        std::fs::create_dir_all(&data_dir).ok();
        std::fs::create_dir_all(data_dir.join("boards")).ok();
        Self { data_dir }
    }

    fn board_path(&self, id: Uuid) -> PathBuf {
        self.data_dir.join("boards").join(format!("{id}.toml"))
    }

    fn index_path(&self) -> PathBuf {
        self.data_dir.join("boards.index.toml")
    }

    fn write_atomic(&self, path: &Path, content: &str) -> Result<(), StoreError> {
        let dir = path.parent().expect("path has no parent");
        let mut tmp = NamedTempFile::new_in(dir)?;
        tmp.write_all(content.as_bytes())?;
        tmp.flush()?;
        tmp.as_file().sync_all()?;
        tmp.persist(path)?;
        Ok(())
    }

    pub fn rebuild_index(&self) -> Result<(), StoreError> {
        let boards_dir = self.data_dir.join("boards");
        let mut summaries = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&boards_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(board) = toml::from_str::<Board>(&content) {
                            let card_count: usize = board.lists.iter().map(|l| l.cards.len()).sum();
                            summaries.push(BoardSummary {
                                id: board.id,
                                title: board.title.clone(),
                                background: board.background,
                                icon: board.icon.clone(),
                                list_count: board.lists.len(),
                                card_count,
                            });
                        }
                    }
                }
            }
        }

        summaries.sort_by_key(|s| s.id);

        let index = IndexFile { boards: summaries };
        let content = toml::to_string_pretty(&index)?;
        self.write_atomic(&self.index_path(), &content)
    }
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

impl DataStore for Store {
    type Error = StoreError;

    fn load_board_index(&self) -> Result<Vec<BoardSummary>, Self::Error> {
        let path = self.index_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(path)?;
        let index: IndexFile = toml::from_str(&content)?;
        Ok(index.boards)
    }

    fn load_board(&self, id: Uuid) -> Result<Board, Self::Error> {
        let content = std::fs::read_to_string(self.board_path(id))?;
        Ok(toml::from_str(&content)?)
    }

    fn save_board(&self, board: &Board) -> Result<(), Self::Error> {
        let content = toml::to_string_pretty(board)?;
        self.write_atomic(&self.board_path(board.id), &content)?;
        self.rebuild_index()
    }

    fn delete_board(&self, id: Uuid) -> Result<(), Self::Error> {
        let path = self.board_path(id);
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        self.rebuild_index()
    }
}
