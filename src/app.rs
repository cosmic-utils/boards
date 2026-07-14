// SPDX-License-Identifier: GPL-3.0

use crate::dialog::DialogHost;
use cosmic::{
    Application,
    app::context_drawer,
    cosmic_config::{self, CosmicConfigEntry},
    iced::Subscription,
    prelude::*,
    widget::{self, about::About, icon, menu, nav_bar},
};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    board::{
        Board,
        dialog::{BoardSettingsDialog, BoardSettingsMessage, NewBoardDialog, NewBoardMessage},
        message::BoardMessage,
    },
    card::{
        Card,
        dialog::{
            AccentColorDialog, AccentColorDialogMessage, CardDetailDialog, CardDetailsMessage,
        },
        message::CardMessage,
    },
    column::{dialog::NewColumnDialog, dialog::NewColumnMessage, message::ColumnMessage},
    config::Config,
    dnd::message::DndMessage,
    fl,
    menu_bar::{self, MenuAction},
    storage::{DataStore, store::Store},
    tag::{
        dialog::{TagDialog, TagDialogMessage},
        message::TagMessage,
    },
};

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const APP_ICON: &[u8] = include_bytes!("../resources/icons/hicolor/scalable/apps/icon.svg");

pub struct AppModel {
    pub core: cosmic::Core,
    pub nav: nav_bar::Model,
    pub boards: HashMap<Uuid, Board>,
    pub new_card_input: Option<(widget::Id, Uuid, String)>,
    pub context_page: ContextPage,
    pub about: About,
    pub key_binds: HashMap<menu::KeyBind, MenuAction>,
    pub config: Config,
    pub page: Option<DialogPage>,
    pub tag_dialog: Option<TagDialog>,
    pub accent_color_dialog: Option<AccentColorDialog>,
    pub drag_hover: Option<(Uuid, Option<Uuid>)>,
    pub editing_column_title: Option<Uuid>,
    pub search_query: String,
    pub suppress_next_card_open: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    NewBoardDialog(NewBoardMessage),
    NewColumnDialog(NewColumnMessage),
    BoardSettingsDialog(BoardSettingsMessage),
    TagDialog(TagDialogMessage),
    OpenTagDialog(Option<Uuid>),
    AccentColorDialog(AccentColorDialogMessage),
    OpenAccentColorDialog(Uuid),
    CardDetailsDialog(CardDetailsMessage),

    Board(BoardMessage),
    Column(ColumnMessage),
    CardMenuAction(CardMessage),
    CardClickReleased(Uuid),
    Dnd(DndMessage),

    BoardLoaded(Board),
    BoardSaved,
    LoadError(String),
    LaunchUrl(String),
    UpdateConfig(Config),
    ToggleContextPage(ContextPage),
    OpenDialogPage(DialogPage),
    ToggleNavBar,
    Quit,
    KeyPressed(cosmic::iced::keyboard::Event),
    SearchChanged(String),
}

#[derive(Debug, Clone)]
pub enum DialogPage {
    NewBoard(NewBoardDialog),
    NewColumn(NewColumnDialog),
    BoardSettings(BoardSettingsDialog),
    CardDetail(CardDetailDialog),
}

impl cosmic::Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "dev.edfloreshz.Boards";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(core: cosmic::Core, _flags: ()) -> (Self, Task<cosmic::Action<Self::Message>>) {
        let store = Store::new();
        let summaries = store.load_board_index().unwrap_or_default();

        let mut nav = nav_bar::Model::default();
        for summary in &summaries {
            nav.insert()
                .text(summary.title.clone())
                .icon(icon::from_name(summary.icon.as_str()))
                .data(summary.id);
        }

        let config = cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
            .map(|context| match Config::get_entry(&context) {
                Ok(config) => config,
                Err((_errors, config)) => config,
            })
            .unwrap_or_default();

        {
            let ids: Vec<_> = nav.iter().collect();
            let to_activate = if let Some(last_id) = config.last_board_id {
                ids.iter()
                    .copied()
                    .find(|&nid| nav.data::<Uuid>(nid) == Some(&last_id))
                    .or_else(|| ids.first().copied())
            } else {
                ids.first().copied()
            };
            if let Some(id) = to_activate {
                nav.activate(id);
            }
        }

        let about = About::default()
            .name(fl!("app-title"))
            .icon(widget::icon::from_svg_bytes(APP_ICON))
            .version(env!("CARGO_PKG_VERSION"))
            .links([(fl!("repository"), REPOSITORY)])
            .license(env!("CARGO_PKG_LICENSE"));

        let mut app = AppModel {
            core,
            nav,
            boards: HashMap::new(),
            new_card_input: None,
            context_page: ContextPage::default(),
            about,
            key_binds: menu_bar::key_binds(),
            config,
            page: None,
            tag_dialog: None,
            accent_color_dialog: None,
            drag_hover: None,
            editing_column_title: None,
            search_query: String::new(),
            suppress_next_card_open: false,
        };

        let load_task = if let Some(board_id) = app.active_board_id() {
            Task::perform(
                async move { Store::new().load_board(board_id) },
                |result| match result {
                    Ok(board) => cosmic::Action::App(Message::BoardLoaded(board)),
                    Err(e) => cosmic::Action::App(Message::LoadError(e.to_string())),
                },
            )
        } else {
            app.core.nav_bar_set_toggled(false);
            Task::none()
        };

        let title_task = app.update_title();

        (app, Task::batch([load_task, title_task]))
    }

    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        vec![menu_bar::menu_bar(self)]
    }

    fn header_center(&self) -> Vec<Element<'_, Self::Message>> {
        if self.active_board().is_some() {
            vec![
                widget::search_input(fl!("search"), self.search_query.as_str())
                    .on_input(Message::SearchChanged)
                    .on_clear(Message::SearchChanged(String::new()))
                    .width(cosmic::iced::Length::Fixed(280.0))
                    .into(),
            ]
        } else {
            vec![]
        }
    }

    fn header_end(&self) -> Vec<Element<'_, Self::Message>> {
        if let Some(board) = self.active_board() {
            vec![
                widget::button::icon(widget::icon::from_name("list-add-symbolic"))
                    .on_press(Message::OpenDialogPage(DialogPage::NewColumn(
                        NewColumnDialog::new(),
                    )))
                    .into(),
                widget::button::icon(widget::icon::from_name("emblem-system-symbolic"))
                    .on_press(Message::Board(BoardMessage::OpenSettings(board.id)))
                    .into(),
            ]
        } else {
            vec![]
        }
    }

    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    fn nav_context_menu(&self) -> Option<Vec<menu::Tree<cosmic::Action<Self::Message>>>> {
        Some(self.board_nav_menu())
    }

    fn dialog(&self) -> Option<Element<'_, Self::Message>> {
        if let Some(accent_dialog) = &self.accent_color_dialog {
            return Some(accent_dialog.view().map(Message::AccentColorDialog));
        }
        if let Some(tag_dialog) = &self.tag_dialog {
            return tag_dialog.dialog().map(|el| el.map(Message::TagDialog));
        }

        let page = self.page.as_ref()?;

        match page {
            DialogPage::NewBoard(p) => p.dialog().map(|el| el.map(Message::NewBoardDialog)),
            DialogPage::NewColumn(p) => p.dialog().map(|el| el.map(Message::NewColumnDialog)),
            DialogPage::BoardSettings(p) => {
                p.dialog().map(|el| el.map(Message::BoardSettingsDialog))
            }
            DialogPage::CardDetail(p) => Some(self.view_card_detail(p)),
        }
    }

    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<'_, Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match &self.context_page {
            ContextPage::About => context_drawer::about(
                &self.about,
                |url| Message::LaunchUrl(url.to_string()),
                Message::ToggleContextPage(ContextPage::About),
            ),
        })
    }

    fn view(&self) -> Element<'_, Self::Message> {
        if self.nav.iter().next().is_none() {
            return self.view_empty_state();
        }

        match self.active_board() {
            None => self.view_empty_state(),
            Some(board) => self.view_board(board),
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let subscriptions = vec![
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| Message::UpdateConfig(update.config)),
            cosmic::iced::keyboard::listen().map(Message::KeyPressed),
        ];
        Subscription::batch(subscriptions)
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::Board(msg) => return self.update_board(msg),
            Message::Column(msg) => return self.update_column(msg),
            Message::Dnd(msg) => return self.update_dnd(msg),

            Message::CardMenuAction(msg) => {
                self.suppress_next_card_open = true;
                return self.update_card(msg);
            }

            Message::CardClickReleased(card_id) => {
                let suppressed = std::mem::take(&mut self.suppress_next_card_open);
                if !suppressed {
                    return self.update_card(CardMessage::Open(card_id));
                }
            }

            Message::BoardLoaded(board) => {
                self.boards.insert(board.id, board);
            }

            Message::ToggleContextPage(page) => {
                if self.context_page == page {
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    self.context_page = page;
                    self.core.window.show_context = true;
                }
            }

            Message::OpenDialogPage(page) => {
                self.page = Some(page.clone());
                let focus = match &page {
                    DialogPage::NewBoard(dialog) => dialog
                        .dialog
                        .get()
                        .map(|s| cosmic::widget::text_input::focus(s.input_id.clone()))
                        .unwrap_or_else(Task::none),
                    DialogPage::NewColumn(dialog) => dialog
                        .dialog
                        .get()
                        .map(|s| cosmic::widget::text_input::focus(s.input_id.clone()))
                        .unwrap_or_else(Task::none),
                    DialogPage::BoardSettings(dialog) => dialog
                        .dialog
                        .get()
                        .map(|s| cosmic::widget::text_input::focus(s.input_id.clone()))
                        .unwrap_or_else(Task::none),
                    DialogPage::CardDetail(_) => Task::none(),
                };
                return Task::batch(vec![focus]);
            }

            Message::BoardSaved => {}
            Message::LoadError(err) => eprintln!("Boards error: {err}"),

            Message::LaunchUrl(url) => {
                if let Err(e) = open::that_detached(&url) {
                    eprintln!("failed to open {url:?}: {e}");
                }
            }

            Message::UpdateConfig(config) => {
                self.config = config;
            }

            Message::SearchChanged(query) => {
                self.search_query = query;
            }

            Message::ToggleNavBar => {
                self.core.nav_bar_toggle();
            }

            Message::Quit => {
                if let Some(id) = self.core.main_window_id() {
                    return cosmic::iced::window::close(id);
                }
            }

            Message::KeyPressed(event) => {
                let resolved = if let cosmic::iced::keyboard::Event::KeyPressed {
                    key,
                    modifiers,
                    physical_key,
                    repeat: false,
                    ..
                } = event
                {
                    self.key_binds
                        .iter()
                        .find(|(bind, _)| bind.matches(modifiers, &key, Some(&physical_key)))
                        .map(|(_, action)| *action)
                } else {
                    None
                };
                let resolved = resolved.map(|action| match action {
                    MenuAction::BoardSettings(_) => {
                        MenuAction::BoardSettings(self.active_board_id().unwrap_or_default())
                    }
                    other => other,
                });
                let resolved = resolved.map(|action| menu::action::MenuAction::message(&action));
                if let Some(msg) = resolved {
                    return self.update(msg);
                }
            }

            Message::NewBoardDialog(message) => {
                if matches!(message, NewBoardMessage::Submit) {
                    if let Some(DialogPage::NewBoard(dialog)) = &self.page {
                        let title = dialog
                            .dialog
                            .get()
                            .map(|s| s.title.trim().to_string())
                            .unwrap_or_default();
                        self.page = None;
                        if !title.is_empty() {
                            return self.update_board(BoardMessage::Create(title));
                        }
                    }
                    return Task::none();
                }
                if let Some(DialogPage::NewBoard(dialog)) = &mut self.page {
                    let task = dialog
                        .update(message)
                        .map(Message::NewBoardDialog)
                        .map(cosmic::action::app);
                    if dialog.dialog.get().is_none() {
                        self.page = None;
                    }
                    return task;
                }
            }
            Message::NewColumnDialog(message) => {
                if matches!(message, NewColumnMessage::Submit) {
                    if let Some(DialogPage::NewColumn(dialog)) = &self.page {
                        let title = dialog
                            .dialog
                            .get()
                            .map(|s| s.title.trim().to_string())
                            .unwrap_or_default();
                        self.page = None;
                        if !title.is_empty() {
                            return self.update_column(ColumnMessage::Create(title));
                        }
                    }
                    return Task::none();
                }
                if let Some(DialogPage::NewColumn(dialog)) = &mut self.page {
                    let task = dialog
                        .update(message)
                        .map(Message::NewColumnDialog)
                        .map(cosmic::action::app);
                    if dialog.dialog.get().is_none() {
                        self.page = None;
                    }
                    return task;
                }
            }
            Message::BoardSettingsDialog(message) => {
                if matches!(message, BoardSettingsMessage::Rename) {
                    if let Some(DialogPage::BoardSettings(dialog)) = &self.page
                        && let Some(state) = dialog.dialog.get()
                    {
                        let new_title = state.title.trim().to_string();
                        let board_id = state.board_id;
                        self.page = None;
                        if !new_title.is_empty() {
                            return self.update_board(BoardMessage::Rename {
                                id: board_id,
                                new_title,
                            });
                        }
                    }
                    return Task::none();
                }
                if matches!(message, BoardSettingsMessage::Delete) {
                    if let Some(DialogPage::BoardSettings(dialog)) = &self.page
                        && let Some(state) = dialog.dialog.get()
                    {
                        let board_id = state.board_id;
                        self.page = None;
                        return self.update_board(BoardMessage::Delete(board_id));
                    }
                    return Task::none();
                }
                if let BoardSettingsMessage::SetIcon(new_icon) = &message {
                    if let Some(DialogPage::BoardSettings(dialog)) = &mut self.page
                        && let Some(state) = dialog.dialog.get_mut()
                    {
                        let board_id = state.board_id;
                        state.current_icon = new_icon.clone();
                        return self.update_board(BoardMessage::SetIcon {
                            id: board_id,
                            icon: new_icon.clone(),
                        });
                    }
                    return Task::none();
                }
                if let Some(DialogPage::BoardSettings(dialog)) = &mut self.page {
                    let task = dialog
                        .update(message)
                        .map(Message::BoardSettingsDialog)
                        .map(cosmic::action::app);
                    if dialog.dialog.get().is_none() {
                        self.page = None;
                    }
                    return task;
                }
            }
            Message::OpenTagDialog(editing_id) => {
                let dialog = match editing_id {
                    Some(id) => self
                        .active_board()
                        .and_then(|board| board.tags.iter().find(|t| t.id == id))
                        .map(TagDialog::new_edit)
                        .unwrap_or_else(TagDialog::new_create),
                    None => TagDialog::new_create(),
                };
                let focus = dialog
                    .dialog
                    .get()
                    .map(|s| cosmic::widget::text_input::focus(s.input_id.clone()))
                    .unwrap_or_else(Task::none);
                self.tag_dialog = Some(dialog);
                return focus;
            }

            Message::OpenAccentColorDialog(card_id) => {
                let current = self.active_card(card_id).and_then(|c| c.accent_color);
                self.accent_color_dialog = Some(AccentColorDialog::new(card_id, current));
            }

            Message::AccentColorDialog(message) => {
                return self.update_accent_color_dialog(message);
            }

            Message::TagDialog(message) => {
                if matches!(message, TagDialogMessage::Submit) {
                    if let Some(dialog) = &self.tag_dialog
                        && let Some(state) = dialog.dialog.get()
                    {
                        let name = state.name.trim().to_string();
                        let color = state
                            .color_picker
                            .get_applied_color()
                            .unwrap_or_else(crate::tag::default_tag_color);
                        let editing_id = state.editing_tag_id;
                        self.tag_dialog = None;
                        if !name.is_empty() {
                            return self.update_tag(match editing_id {
                                Some(id) => TagMessage::Update { id, name, color },
                                None => TagMessage::Create { name, color },
                            });
                        }
                    }
                    return Task::none();
                }
                if matches!(message, TagDialogMessage::Delete) {
                    if let Some(dialog) = &self.tag_dialog
                        && let Some(id) = dialog.dialog.get().and_then(|s| s.editing_tag_id)
                    {
                        self.tag_dialog = None;
                        return self.update_tag(TagMessage::Delete(id));
                    }
                    return Task::none();
                }
                if let Some(dialog) = &mut self.tag_dialog {
                    let task = dialog
                        .update(message)
                        .map(Message::TagDialog)
                        .map(cosmic::action::app);
                    if dialog.dialog.get().is_none() {
                        self.tag_dialog = None;
                    }
                    return task;
                }
            }
            Message::CardDetailsDialog(message) => return self.update_card_detail(message),
        }

        Task::none()
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<cosmic::Action<Self::Message>> {
        self.nav.activate(id);
        self.new_card_input = None;
        self.editing_column_title = None;
        self.core.window.show_context = false;

        let load_task = if let Some(board_id) = self.active_board_id() {
            if !self.boards.contains_key(&board_id) {
                Task::perform(async move { Store::new().load_board(board_id) }, |result| {
                    match result {
                        Ok(board) => cosmic::Action::App(Message::BoardLoaded(board)),
                        Err(e) => cosmic::Action::App(Message::LoadError(e.to_string())),
                    }
                })
            } else {
                Task::none()
            }
        } else {
            Task::none()
        };

        Task::batch([load_task, self.update_title()])
    }
}

impl AppModel {
    pub fn active_board_id(&self) -> Option<Uuid> {
        self.nav.active_data::<Uuid>().copied()
    }

    pub fn active_board(&self) -> Option<&Board> {
        self.active_board_id().and_then(|id| self.boards.get(&id))
    }

    pub fn active_board_mut(&mut self) -> Option<&mut Board> {
        let id = self.active_board_id()?;
        self.boards.get_mut(&id)
    }

    pub fn active_card(&self, card_id: Uuid) -> Option<&Card> {
        self.active_board()?
            .columns
            .iter()
            .find_map(|l| l.cards.iter().find(|c| c.id == card_id))
    }

    pub fn active_card_mut(&mut self, card_id: Uuid) -> Option<&mut Card> {
        self.active_board_mut()?
            .columns
            .iter_mut()
            .find_map(|l| l.cards.iter_mut().find(|c| c.id == card_id))
    }

    pub fn touch_board(&mut self) {
        if let Some(board) = self.active_board_mut() {
            board.updated_at = jiff::Timestamp::now();
        }
    }

    pub fn save_board_task(&self, board: Board) -> Task<cosmic::Action<Message>> {
        Task::perform(
            async move { Store::new().save_board(&board) },
            |result| match result {
                Ok(_) => cosmic::Action::App(Message::BoardSaved),
                Err(e) => cosmic::Action::App(Message::LoadError(e.to_string())),
            },
        )
    }

    pub fn save_active_board(&self) -> Task<cosmic::Action<Message>> {
        if let Some(board) = self.active_board() {
            let board = board.clone();
            self.save_board_task(board)
        } else {
            Task::none()
        }
    }

    pub fn activate_board(&mut self, id: Uuid) -> Task<cosmic::Action<Message>> {
        let ids: Vec<_> = self.nav.iter().collect();
        if let Some(nav_id) = ids
            .into_iter()
            .find(|&nid| self.nav.data::<Uuid>(nid) == Some(&id))
        {
            return self.on_nav_select(nav_id);
        }
        Task::none()
    }

    pub fn update_title(&mut self) -> Task<cosmic::Action<Message>> {
        let mut window_title = fl!("app-title");
        if let Some(page) = self.nav.text(self.nav.active()) {
            window_title.push_str(" — ");
            window_title.push_str(page);
        }
        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(window_title, id)
        } else {
            Task::none()
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
}
