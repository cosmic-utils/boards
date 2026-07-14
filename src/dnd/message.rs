use cosmic::Task;
use uuid::Uuid;

use crate::app::{AppModel, Message};

#[derive(Debug, Clone)]
pub enum DndMessage {
    CardStarted(Uuid),
    HoverChanged {
        list_id: Uuid,
        before_card_id: Option<Uuid>,
    },
    LeftDropZone,
    CardDropped {
        card_id: Uuid,
        target_list_id: Uuid,
        before_card_id: Option<Uuid>,
    },
    CardCancelled,
}

impl AppModel {
    pub fn update_dnd(&mut self, message: DndMessage) -> Task<cosmic::Action<Message>> {
        match message {
            DndMessage::CardStarted(_card_id) => Task::none(),

            DndMessage::HoverChanged {
                list_id,
                before_card_id,
            } => {
                self.drag_hover = Some((list_id, before_card_id));
                Task::none()
            }

            DndMessage::LeftDropZone | DndMessage::CardCancelled => {
                self.drag_hover = None;
                Task::none()
            }

            DndMessage::CardDropped {
                card_id,
                target_list_id,
                before_card_id,
            } => {
                self.drag_hover = None;
                if before_card_id == Some(card_id) {
                    return Task::none();
                }

                let actual_before_id = before_card_id.and_then(|target_card_id| {
                    let list = self
                        .active_board()?
                        .lists
                        .iter()
                        .find(|l| l.id == target_list_id)?;
                    let src_idx = list.cards.iter().position(|c| c.id == card_id);
                    let tgt_idx = list.cards.iter().position(|c| c.id == target_card_id);
                    match (src_idx, tgt_idx) {
                        (Some(s), Some(t)) if s < t => list.cards.get(t + 1).map(|c| c.id),
                        _ => Some(target_card_id),
                    }
                });

                self.move_card(card_id, target_list_id, actual_before_id);
                self.save_active_board()
            }
        }
    }
}
