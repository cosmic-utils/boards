use cosmic::{Task, widget};
use uuid::Uuid;

use crate::{
    app::{AppModel, Message},
    card::message::CardMessage,
    column::Column,
};

#[derive(Debug, Clone)]
pub enum ColumnMessage {
    Create(String),
    Rename { column_id: Uuid, new_title: String },
    Delete(Uuid),
    ToggleTitleEdit(Uuid),
    SortByTag(Uuid),
    OpenNewCardInput(widget::Id, Uuid),
    NewCardInputChanged(String),
    ConfirmNewCard(Uuid),
    DismissNewInput,
}

impl AppModel {
    pub fn update_column(&mut self, message: ColumnMessage) -> Task<cosmic::Action<Message>> {
        match message {
            ColumnMessage::Create(title) => {
                if let Some(board) = self.active_board_mut() {
                    let position = board.columns.len() as u32;
                    board.columns.push(Column::new(title, position));
                    board.updated_at = jiff::Timestamp::now();
                }
                self.save_active_board()
            }

            ColumnMessage::Rename {
                column_id,
                new_title,
            } => {
                if let Some(board) = self.active_board_mut() {
                    if let Some(column) = board.columns.iter_mut().find(|l| l.id == column_id) {
                        column.title = new_title;
                        self.touch_board();
                    }
                }
                self.save_active_board()
            }

            ColumnMessage::Delete(column_id) => {
                if let Some(board) = self.active_board_mut() {
                    board.columns.retain(|l| l.id != column_id);
                    for (i, column) in board.columns.iter_mut().enumerate() {
                        column.position = i as u32;
                    }
                    self.touch_board();
                }
                if self.editing_column_title == Some(column_id) {
                    self.editing_column_title = None;
                }
                self.save_active_board()
            }

            ColumnMessage::ToggleTitleEdit(column_id) => {
                self.editing_column_title = if self.editing_column_title == Some(column_id) {
                    None
                } else {
                    Some(column_id)
                };
                Task::none()
            }

            ColumnMessage::SortByTag(column_id) => {
                if let Some(board) = self.active_board_mut() {
                    let tags = board.tags.clone();
                    if let Some(column) = board.columns.iter_mut().find(|l| l.id == column_id) {
                        column.cards.sort_by_key(|card| {
                            let name = card
                                .tag_ids
                                .iter()
                                .filter_map(|id| tags.iter().find(|t| t.id == *id))
                                .map(|t| t.name.to_lowercase())
                                .min();
                            (name.is_none(), name.unwrap_or_default())
                        });
                        for (i, card) in column.cards.iter_mut().enumerate() {
                            card.position = i as u32;
                        }
                    }
                    self.touch_board();
                }
                self.save_active_board()
            }

            ColumnMessage::OpenNewCardInput(input_id, column_id) => {
                self.new_card_input = Some((input_id.clone(), column_id, String::new()));
                cosmic::widget::text_input::focus(input_id)
            }

            ColumnMessage::NewCardInputChanged(text) => {
                if let Some((_, _, ref mut t)) = self.new_card_input {
                    *t = text;
                }
                Task::none()
            }

            ColumnMessage::ConfirmNewCard(column_id) => {
                self.update_card(CardMessage::Create(column_id))
            }

            ColumnMessage::DismissNewInput => {
                self.new_card_input = None;
                Task::none()
            }
        }
    }
}
