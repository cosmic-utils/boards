use cosmic::Task;
use cosmic::widget::icon;
use uuid::Uuid;

use crate::{
    app::{AppModel, DialogPage, Message},
    board::Board,
    board::dialog::BoardSettingsDialog,
    storage::{DataStore, store::Store},
};

#[derive(Debug, Clone)]
pub enum BoardMessage {
    Create(String),
    Rename { id: Uuid, new_title: String },
    SetIcon { id: Uuid, icon: String },
    Delete(Uuid),
    Activate(Uuid),
    OpenSettings(Uuid),
}

impl AppModel {
    pub fn update_board(&mut self, message: BoardMessage) -> Task<cosmic::Action<Message>> {
        match message {
            BoardMessage::Create(title) => {
                let board = Board::new(title);
                let id = board.id;

                self.nav
                    .insert()
                    .text(board.title.clone())
                    .icon(icon::from_name(board.icon.as_str()))
                    .data(id);

                let ids: Vec<_> = self.nav.iter().collect();
                if let Some(nav_id) = ids
                    .into_iter()
                    .find(|&nid| self.nav.data::<Uuid>(nid) == Some(&id))
                {
                    self.nav.activate(nav_id);
                }
                self.core.nav_bar_set_toggled(true);

                let save_task = self.save_board_task(board.clone());
                self.boards.insert(id, board);

                Task::batch([save_task, self.update_title()])
            }

            BoardMessage::Rename { id, new_title } => {
                if let Some(board) = self.boards.get_mut(&id) {
                    board.title = new_title.clone();
                    board.updated_at = jiff::Timestamp::now();
                }
                let ids: Vec<_> = self.nav.iter().collect();
                if let Some(nav_id) = ids
                    .into_iter()
                    .find(|&nid| self.nav.data::<Uuid>(nid) == Some(&id))
                {
                    self.nav.text_set(nav_id, new_title);
                }
                self.save_active_board()
            }

            BoardMessage::SetIcon { id, icon: new_icon } => {
                if let Some(board) = self.boards.get_mut(&id) {
                    board.icon = new_icon.clone();
                    board.updated_at = jiff::Timestamp::now();
                }
                let ids: Vec<_> = self.nav.iter().collect();
                if let Some(nav_id) = ids
                    .into_iter()
                    .find(|&nid| self.nav.data::<Uuid>(nid) == Some(&id))
                {
                    self.nav
                        .icon_set(nav_id, icon::from_name(new_icon.as_str()).into());
                }
                self.save_active_board()
            }

            BoardMessage::Delete(id) => {
                let ids: Vec<_> = self.nav.iter().collect();
                if let Some(nid) = ids
                    .into_iter()
                    .find(|&nid| self.nav.data::<Uuid>(nid) == Some(&id))
                {
                    self.nav.remove(nid);
                }
                self.boards.remove(&id);

                let first_id = self.nav.iter().next();
                if let Some(first) = first_id {
                    self.nav.activate(first);
                }
                self.core.window.show_context = false;

                let task = Task::perform(async move { Store::new().delete_board(id) }, |result| {
                    match result {
                        Ok(_) => cosmic::Action::App(Message::BoardSaved),
                        Err(e) => cosmic::Action::App(Message::LoadError(e.to_string())),
                    }
                });
                Task::batch([task, self.update_title()])
            }

            BoardMessage::Activate(id) => self.activate_board(id),

            BoardMessage::OpenSettings(id) => {
                let activate_task = self.activate_board(id);
                let Some(board) = self.boards.get(&id) else {
                    return activate_task;
                };
                let dialog = BoardSettingsDialog::new(board);
                let focus = dialog
                    .dialog
                    .get()
                    .map(|s| cosmic::widget::text_input::focus(s.input_id.clone()))
                    .unwrap_or_else(Task::none);
                self.page = Some(DialogPage::BoardSettings(dialog));
                Task::batch([activate_task, focus])
            }
        }
    }
}
