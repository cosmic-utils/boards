use cosmic::{Task, widget};
use uuid::Uuid;

use crate::{
    app::{AppModel, Message},
    card::message::CardMessage,
    list::List,
};

#[derive(Debug, Clone)]
pub enum ListMessage {
    Create(String),
    Rename { list_id: Uuid, new_title: String },
    Delete(Uuid),
    ToggleTitleEdit(Uuid),
    SortByTag(Uuid),
    OpenNewCardInput(widget::Id, Uuid),
    NewCardInputChanged(String),
    ConfirmNewCard(Uuid),
    DismissNewInput,
}

impl AppModel {
    pub fn update_list(&mut self, message: ListMessage) -> Task<cosmic::Action<Message>> {
        match message {
            ListMessage::Create(title) => {
                if let Some(board) = self.active_board_mut() {
                    let position = board.lists.len() as u32;
                    board.lists.push(List::new(title, position));
                    board.updated_at = jiff::Timestamp::now();
                }
                self.save_active_board()
            }

            ListMessage::Rename { list_id, new_title } => {
                if let Some(board) = self.active_board_mut() {
                    if let Some(list) = board.lists.iter_mut().find(|l| l.id == list_id) {
                        list.title = new_title;
                        self.touch_board();
                    }
                }
                self.save_active_board()
            }

            ListMessage::Delete(list_id) => {
                if let Some(board) = self.active_board_mut() {
                    board.lists.retain(|l| l.id != list_id);
                    for (i, list) in board.lists.iter_mut().enumerate() {
                        list.position = i as u32;
                    }
                    self.touch_board();
                }
                if self.editing_list_title == Some(list_id) {
                    self.editing_list_title = None;
                }
                self.save_active_board()
            }

            ListMessage::ToggleTitleEdit(list_id) => {
                self.editing_list_title = if self.editing_list_title == Some(list_id) {
                    None
                } else {
                    Some(list_id)
                };
                Task::none()
            }

            ListMessage::SortByTag(list_id) => {
                if let Some(board) = self.active_board_mut() {
                    let tags = board.tags.clone();
                    if let Some(list) = board.lists.iter_mut().find(|l| l.id == list_id) {
                        list.cards.sort_by_key(|card| {
                            let name = card
                                .tag_ids
                                .iter()
                                .filter_map(|id| tags.iter().find(|t| t.id == *id))
                                .map(|t| t.name.to_lowercase())
                                .min();
                            (name.is_none(), name.unwrap_or_default())
                        });
                        for (i, card) in list.cards.iter_mut().enumerate() {
                            card.position = i as u32;
                        }
                    }
                    self.touch_board();
                }
                self.save_active_board()
            }

            ListMessage::OpenNewCardInput(input_id, list_id) => {
                self.new_card_input = Some((input_id.clone(), list_id, String::new()));
                cosmic::widget::text_input::focus(input_id)
            }

            ListMessage::NewCardInputChanged(text) => {
                if let Some((_, _, ref mut t)) = self.new_card_input {
                    *t = text;
                }
                Task::none()
            }

            ListMessage::ConfirmNewCard(list_id) => self.update_card(CardMessage::Create(list_id)),

            ListMessage::DismissNewInput => {
                self.new_card_input = None;
                Task::none()
            }
        }
    }
}
