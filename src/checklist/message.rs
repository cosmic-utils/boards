use cosmic::Task;
use uuid::Uuid;

use crate::{
    app::{AppModel, Message},
    checklist::ChecklistItem,
};

#[derive(Debug, Clone)]
pub enum ChecklistMessage {
    Add { card_id: Uuid, text: String },
    Toggle { card_id: Uuid, item_id: Uuid },
    Delete { card_id: Uuid, item_id: Uuid },
}

impl AppModel {
    pub fn update_checklist(&mut self, message: ChecklistMessage) -> Task<cosmic::Action<Message>> {
        match message {
            ChecklistMessage::Add { card_id, text } => {
                if let Some(card) = self.active_card_mut(card_id) {
                    let position = card.checklist.len() as u32;
                    card.checklist.push(ChecklistItem::new(text, position));
                    card.updated_at = jiff::Timestamp::now();
                }
                self.touch_board();
                self.save_active_board()
            }

            ChecklistMessage::Toggle { card_id, item_id } => {
                if let Some(card) = self.active_card_mut(card_id) {
                    if let Some(item) = card.checklist.iter_mut().find(|i| i.id == item_id) {
                        item.completed = !item.completed;
                        card.updated_at = jiff::Timestamp::now();
                    }
                }
                self.touch_board();
                self.save_active_board()
            }

            ChecklistMessage::Delete { card_id, item_id } => {
                if let Some(card) = self.active_card_mut(card_id) {
                    card.checklist.retain(|i| i.id != item_id);
                    for (idx, item) in card.checklist.iter_mut().enumerate() {
                        item.position = idx as u32;
                    }
                    card.updated_at = jiff::Timestamp::now();
                }
                self.touch_board();
                self.save_active_board()
            }
        }
    }
}
