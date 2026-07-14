use cosmic::Task;
use cosmic::iced::Color;
use uuid::Uuid;

use crate::{
    app::{AppModel, Message},
    tag::Tag,
};

#[derive(Debug, Clone)]
pub enum TagMessage {
    Create {
        name: String,
        color: Color,
    },
    Update {
        id: Uuid,
        name: String,
        color: Color,
    },
    Delete(Uuid),
}

impl AppModel {
    pub fn update_tag(&mut self, message: TagMessage) -> Task<cosmic::Action<Message>> {
        match message {
            TagMessage::Create { name, color } => {
                if let Some(board) = self.active_board_mut() {
                    board.tags.push(Tag::new(name, color));
                    board.updated_at = jiff::Timestamp::now();
                }
                self.save_active_board()
            }

            TagMessage::Update { id, name, color } => {
                if let Some(board) = self.active_board_mut() {
                    if let Some(tag) = board.tags.iter_mut().find(|t| t.id == id) {
                        tag.name = name;
                        tag.color = color;
                    }
                    board.updated_at = jiff::Timestamp::now();
                }
                self.save_active_board()
            }

            TagMessage::Delete(id) => {
                if let Some(board) = self.active_board_mut() {
                    board.tags.retain(|t| t.id != id);
                    for column in &mut board.columns {
                        for card in &mut column.cards {
                            card.tag_ids.retain(|tag_id| *tag_id != id);
                        }
                    }
                    board.updated_at = jiff::Timestamp::now();
                }
                self.save_active_board()
            }
        }
    }
}
