use cosmic::Task;
use cosmic::iced::Color;
use uuid::Uuid;

use crate::{
    app::{AppModel, DialogPage, Message},
    card::{Card, dialog::CardDetailDialog},
    checklist::message::ChecklistMessage,
};

#[derive(Debug, Clone)]
pub enum CardMessage {
    Create(Uuid),
    UpdateTitle {
        card_id: Uuid,
        new_title: String,
    },
    UpdateDescription {
        card_id: Uuid,
        new_description: String,
    },
    Delete(Uuid),
    Open(Uuid),
    MoveUp(Uuid),
    MoveDown(Uuid),
    MoveToList {
        card_id: Uuid,
        target_list_id: Uuid,
    },
    ToggleTag {
        card_id: Uuid,
        tag_id: Uuid,
    },
    SetDueDate {
        card_id: Uuid,
        date: jiff::civil::Date,
    },
    ClearDueDate(Uuid),
    SetAccentColor {
        card_id: Uuid,
        color: Option<Color>,
    },
    Checklist(ChecklistMessage),
}

impl AppModel {
    pub fn update_card(&mut self, message: CardMessage) -> Task<cosmic::Action<Message>> {
        match message {
            CardMessage::Create(list_id) => {
                let title = match &self.new_card_input {
                    Some((_, lid, t)) if *lid == list_id => t.trim().to_string(),
                    _ => return Task::none(),
                };
                if title.is_empty() {
                    return Task::none();
                }
                if let Some(board) = self.active_board_mut() {
                    if let Some(list) = board.lists.iter_mut().find(|l| l.id == list_id) {
                        let position = list.cards.len() as u32;
                        list.cards.push(Card::new(title, position));
                    }
                    board.updated_at = jiff::Timestamp::now();
                }
                self.new_card_input = None;
                self.save_active_board()
            }

            CardMessage::UpdateTitle { card_id, new_title } => {
                if let Some(card) = self.active_card_mut(card_id) {
                    card.title = new_title;
                    card.updated_at = jiff::Timestamp::now();
                }
                self.touch_board();
                self.save_active_board()
            }

            CardMessage::UpdateDescription {
                card_id,
                new_description,
            } => {
                if let Some(card) = self.active_card_mut(card_id) {
                    card.description = new_description;
                    card.updated_at = jiff::Timestamp::now();
                }
                self.touch_board();
                self.save_active_board()
            }

            CardMessage::Delete(card_id) => {
                if let Some(board) = self.active_board_mut() {
                    for list in &mut board.lists {
                        if list.cards.iter().any(|c| c.id == card_id) {
                            list.cards.retain(|c| c.id != card_id);
                            for (i, card) in list.cards.iter_mut().enumerate() {
                                card.position = i as u32;
                            }
                            break;
                        }
                    }
                }
                self.touch_board();
                if self.is_card_detail_open(card_id) {
                    self.page = None;
                }
                self.save_active_board()
            }

            CardMessage::Open(card_id) => {
                self.page = Some(DialogPage::CardDetail(CardDetailDialog::new(card_id)));
                Task::none()
            }

            CardMessage::MoveUp(card_id) => {
                if let Some(board) = self.active_board_mut() {
                    for list in &mut board.lists {
                        if let Some(idx) = list.cards.iter().position(|c| c.id == card_id) {
                            if idx > 0 {
                                list.cards.swap(idx - 1, idx);
                                for (i, card) in list.cards.iter_mut().enumerate() {
                                    card.position = i as u32;
                                }
                            }
                            break;
                        }
                    }
                }
                self.touch_board();
                self.save_active_board()
            }

            CardMessage::MoveDown(card_id) => {
                if let Some(board) = self.active_board_mut() {
                    for list in &mut board.lists {
                        let len = list.cards.len();
                        if let Some(idx) = list.cards.iter().position(|c| c.id == card_id) {
                            if idx + 1 < len {
                                list.cards.swap(idx, idx + 1);
                                for (i, card) in list.cards.iter_mut().enumerate() {
                                    card.position = i as u32;
                                }
                            }
                            break;
                        }
                    }
                }
                self.touch_board();
                self.save_active_board()
            }

            CardMessage::MoveToList {
                card_id,
                target_list_id,
            } => {
                self.move_card(card_id, target_list_id, None);
                self.save_active_board()
            }

            CardMessage::ToggleTag { card_id, tag_id } => {
                if let Some(card) = self.active_card_mut(card_id) {
                    if let Some(pos) = card.tag_ids.iter().position(|id| *id == tag_id) {
                        card.tag_ids.remove(pos);
                    } else {
                        card.tag_ids.push(tag_id);
                    }
                    card.updated_at = jiff::Timestamp::now();
                }
                self.touch_board();
                self.save_active_board()
            }

            CardMessage::SetDueDate { card_id, date } => {
                if let Some(card) = self.active_card_mut(card_id) {
                    card.due_date = Some(date);
                    card.updated_at = jiff::Timestamp::now();
                }
                self.touch_board();
                self.save_active_board()
            }

            CardMessage::ClearDueDate(card_id) => {
                if let Some(card) = self.active_card_mut(card_id) {
                    card.due_date = None;
                    card.updated_at = jiff::Timestamp::now();
                }
                self.touch_board();
                self.save_active_board()
            }

            CardMessage::SetAccentColor { card_id, color } => {
                if let Some(card) = self.active_card_mut(card_id) {
                    card.accent_color = color;
                    card.updated_at = jiff::Timestamp::now();
                }
                self.touch_board();
                self.save_active_board()
            }

            CardMessage::Checklist(msg) => self.update_checklist(msg),
        }
    }

    pub fn move_card(&mut self, card_id: Uuid, target_list_id: Uuid, before_card_id: Option<Uuid>) {
        if let Some(board) = self.active_board_mut() {
            let now = jiff::Timestamp::now();
            let mut moved_card: Option<Card> = None;
            for list in &mut board.lists {
                if let Some(idx) = list.cards.iter().position(|c| c.id == card_id) {
                    moved_card = Some(list.cards.remove(idx));
                    for (i, c) in list.cards.iter_mut().enumerate() {
                        c.position = i as u32;
                    }
                    break;
                }
            }
            if let Some(mut card) = moved_card {
                if let Some(target) = board.lists.iter_mut().find(|l| l.id == target_list_id) {
                    let insert_at = before_card_id
                        .and_then(|bid| target.cards.iter().position(|c| c.id == bid))
                        .unwrap_or(target.cards.len());
                    card.updated_at = now;
                    target.cards.insert(insert_at, card);
                    for (i, c) in target.cards.iter_mut().enumerate() {
                        c.position = i as u32;
                    }
                }
            }
            board.updated_at = now;
        }
    }
}
