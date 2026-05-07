// SPDX-License-Identifier: GPL-3.0

use std::collections::HashMap;

use cosmic::app::context_drawer;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::{Alignment, Length, Subscription};
use cosmic::prelude::*;
use cosmic::widget::{self, about::About, icon, menu, nav_bar};
use uuid::Uuid;

use crate::config::Config;
use crate::db::DataStore;
use crate::db::store::Store;
use crate::fl;
use crate::models::board::Board;
use crate::models::card::Card;
use crate::models::checklist::ChecklistItem;
use crate::models::label::LabelColor;
use crate::models::list::List;
use crate::widgets::list_column::{view_list, view_list_with_input};
use crate::widgets::new_card_input::new_list_input;

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const APP_ICON: &[u8] = include_bytes!("../resources/icons/hicolor/scalable/apps/icon.svg");

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: cosmic::Core,
    /// COSMIC nav bar — one item per board.
    nav: nav_bar::Model,
    /// All boards loaded into memory.
    boards: HashMap<Uuid, Board>,
    /// Inline "add card" input state: (list_id, current_text).
    new_card_input: Option<(Uuid, String)>,
    /// Inline "add list" input state: current text.
    new_list_input: Option<String>,
    /// Inline "new board" title input state.
    new_board_title: String,
    /// The context drawer page currently shown.
    context_page: ContextPage,
    /// The card being edited in the card detail context page.
    detail_card_id: Option<Uuid>,
    /// The about page for this app.
    about: About,
    /// Key bindings for the application's menu bar.
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    /// Configuration data that persists between application runs.
    config: Config,
    /// Calendar state for the due-date picker in the card detail.
    calendar_model: widget::calendar::CalendarModel,
    /// Whether the calendar picker is expanded in the card detail.
    calendar_visible: bool,
    /// Draft text for a new checklist item being typed in.
    new_checklist_text: String,
    /// Draft text for renaming a board in Board Settings.
    rename_board_input: String,
    /// Current DnD hover state: `Some((list_id, Some(card_id)))` means the
    /// cursor is over `card_id` in `list_id` (insert before it);
    /// `Some((list_id, None))` means the cursor is over the trailing append
    /// zone of `list_id`; `None` means no drag is in progress / hovering.
    drag_hover: Option<(Uuid, Option<Uuid>)>,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    // Board CRUD
    CreateBoard,
    NewBoardTitleChanged(String),
    RenameBoard {
        id: Uuid,
        new_title: String,
    },
    DeleteBoard(Uuid),
    BoardLoaded(Board),

    // List CRUD
    CreateList,
    RenameList {
        list_id: Uuid,
        new_title: String,
    },
    DeleteList(Uuid),

    // Card CRUD
    CreateCard(Uuid), // list_id
    UpdateCardTitle {
        card_id: Uuid,
        new_title: String,
    },
    UpdateCardDescription {
        card_id: Uuid,
        new_description: String,
    },
    DeleteCard(Uuid),

    // Checklist
    AddChecklistItem {
        card_id: Uuid,
        text: String,
    },
    ToggleChecklistItem {
        card_id: Uuid,
        item_id: Uuid,
    },
    DeleteChecklistItem {
        card_id: Uuid,
        item_id: Uuid,
    },

    // UI state
    OpenCardDetail(Uuid),
    ToggleContextPage(ContextPage),
    OpenNewListInput,
    OpenNewCardInput(Uuid), // list_id
    NewListInputChanged(String),
    NewCardInputChanged(String),
    ConfirmNewList,
    ConfirmNewCard(Uuid), // list_id
    DismissNewInput,

    // Persistence
    SaveActiveBoard,
    BoardSaved,
    LoadError(String),

    // Misc
    LaunchUrl(String),
    UpdateConfig(Config),

    // Phase 2: Calendar / due date (operate on the currently-open card)
    CalendarDateSelected(jiff::civil::Date),
    CalendarPrevMonth,
    CalendarNextMonth,
    CalendarYearChanged(i32),
    ToggleCalendar,
    ClearDueDate,

    // Phase 2: Labels (operate on the currently-open card)
    ToggleLabel(LabelColor),
    RemoveLabelFromCard {
        card_id: Uuid,
        label_id: Uuid,
    },

    // Phase 2: Checklist inline input
    NewChecklistItemChanged(String),
    ConfirmChecklistItem,

    // Phase 2: Card ordering / moving
    MoveCardUp(Uuid),
    MoveCardDown(Uuid),
    MoveCardToList {
        card_id: Uuid,
        target_list_id: Uuid,
    },

    // Phase 2: Board rename
    RenameBoardInputChanged(String),
    ConfirmBoardRename(Uuid),

    // Phase 2: Drag-and-drop
    /// Emitted by the card's dnd_source when a drag gesture begins.
    DragCardStarted(Uuid),
    /// Emitted when the dragged card enters a card or append-zone drop target.
    ///
    /// `before_card_id = Some(id)` means the cursor is over that card (insert before it).
    /// `before_card_id = None` means the cursor is over the trailing append zone.
    DragHoverChanged {
        list_id: Uuid,
        before_card_id: Option<Uuid>,
    },
    /// Emitted when the dragged card leaves any drop zone.
    DragLeftDropZone,
    /// Emitted by a card or append-zone dnd_destination when a card is dropped.
    DragCardDropped {
        card_id: Uuid,
        target_list_id: Uuid,
        /// The card that should come *after* the dropped card, or None to append.
        before_card_id: Option<Uuid>,
    },
    /// Emitted when a drag is cancelled or finishes without a valid drop.
    DragCardCancelled,
}

/// Create a COSMIC application from the app model
impl cosmic::Application for AppModel {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = ();

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = "dev.edfloreshz.Boards";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    /// Initializes the application with any given flags and startup commands.
    fn init(core: cosmic::Core, _flags: ()) -> (Self, Task<cosmic::Action<Self::Message>>) {
        // Load board summaries from disk index
        let store = Store::new();
        let summaries = store.load_board_index().unwrap_or_default();

        let mut nav = nav_bar::Model::default();
        for summary in &summaries {
            nav.insert()
                .text(summary.title.clone())
                .icon(icon::from_name("view-grid-symbolic"))
                .data(summary.id);
        }

        // Read config
        let config = cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
            .map(|context| match Config::get_entry(&context) {
                Ok(config) => config,
                Err((_errors, config)) => config,
            })
            .unwrap_or_default();

        // Re-activate last board or the first available one
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
            new_list_input: None,
            new_board_title: String::new(),
            context_page: ContextPage::default(),
            detail_card_id: None,
            about,
            key_binds: HashMap::new(),
            config,
            calendar_model: widget::calendar::CalendarModel::now(),
            calendar_visible: false,
            new_checklist_text: String::new(),
            rename_board_input: String::new(),
            drag_hover: None,
        };

        // If a board is active, load it from disk
        let load_task = if let Some(board_id) = app.active_board_id() {
            Task::perform(
                async move { Store::new().load_board(board_id) },
                |result| match result {
                    Ok(board) => cosmic::Action::App(Message::BoardLoaded(board)),
                    Err(e) => cosmic::Action::App(Message::LoadError(e.to_string())),
                },
            )
        } else {
            Task::none()
        };

        let title_task = app.update_title();

        (app, Task::batch([load_task, title_task]))
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("view")).apply(Element::from),
            menu::items(
                &self.key_binds,
                vec![
                    menu::Item::Button(fl!("new-board"), None, MenuAction::NewBoard),
                    menu::Item::Button(fl!("board-settings"), None, MenuAction::BoardSettings),
                    menu::Item::Divider,
                    menu::Item::Button(fl!("about"), None, MenuAction::About),
                ],
            ),
        )]);
        vec![menu_bar.into()]
    }

    /// Elements to pack at the end of the header bar.
    fn header_end(&self) -> Vec<Element<'_, Self::Message>> {
        if self.active_board().is_some() {
            vec![
                widget::button::text(fl!("add-list"))
                    .on_press(Message::OpenNewListInput)
                    .into(),
                widget::button::icon(widget::icon::from_name("emblem-system-symbolic"))
                    .on_press(Message::ToggleContextPage(ContextPage::BoardSettings))
                    .into(),
            ]
        } else {
            vec![
                widget::button::suggested(fl!("new-board"))
                    .on_press(Message::ToggleContextPage(ContextPage::NewBoard))
                    .into(),
            ]
        }
    }

    /// Enables the COSMIC application to create a nav bar with this model.
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    /// Display a context drawer if the context page is requested.
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

            ContextPage::NewBoard => {
                let content = self.view_new_board_form();
                context_drawer::context_drawer(
                    content,
                    Message::ToggleContextPage(ContextPage::NewBoard),
                )
                .title(fl!("new-board"))
            }

            ContextPage::BoardSettings => {
                let content = self.view_board_settings();
                context_drawer::context_drawer(
                    content,
                    Message::ToggleContextPage(ContextPage::BoardSettings),
                )
                .title(fl!("board-settings"))
            }

            ContextPage::CardDetail => {
                let content = self.view_card_detail();
                context_drawer::context_drawer(
                    content,
                    Message::ToggleContextPage(ContextPage::CardDetail),
                )
                .title(fl!("card-detail"))
            }
        })
    }

    /// Describes the interface based on the current state of the application model.
    fn view(&self) -> Element<'_, Self::Message> {
        if self.nav.iter().next().is_none() {
            // No boards yet — empty state
            return self.view_empty_state();
        }

        match self.active_board() {
            None => self.view_empty_state(),
            Some(board) => self.view_board_with_new_list(board),
        }
    }

    /// Register subscriptions for this application.
    fn subscription(&self) -> Subscription<Self::Message> {
        let subscriptions = vec![
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| Message::UpdateConfig(update.config)),
        ];
        Subscription::batch(subscriptions)
    }

    /// Handles messages emitted by the application and its widgets.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            // ── Phase 1 handlers ──────────────────────────────────────────────
            Message::NewBoardTitleChanged(title) => {
                self.new_board_title = title;
            }

            Message::CreateBoard => {
                let title = self.new_board_title.trim().to_string();
                if title.is_empty() {
                    return Task::none();
                }
                let now = jiff::Timestamp::now();
                let board = Board {
                    id: Uuid::new_v4(),
                    title: title.clone(),
                    description: String::new(),
                    background: cosmic::iced::Color::from_rgb8(54, 95, 168),
                    lists: Vec::new(),
                    created_at: now,
                    updated_at: now,
                };
                let board_id = board.id;
                self.nav
                    .insert()
                    .text(title.clone())
                    .icon(icon::from_name("view-grid-symbolic"))
                    .data(board_id)
                    .activate();
                self.boards.insert(board_id, board.clone());
                self.new_board_title.clear();
                // Close the context drawer
                self.core.window.show_context = false;
                let save_task = self.save_board_task(board);
                return Task::batch([save_task, self.update_title()]);
            }

            Message::RenameBoard { id, new_title } => {
                if let Some(board) = self.boards.get_mut(&id) {
                    board.title = new_title.clone();
                    board.updated_at = jiff::Timestamp::now();
                }
                // Update nav label — collect IDs first to avoid borrow conflicts
                let ids: Vec<_> = self.nav.iter().collect();
                if let Some(nav_id) = ids
                    .into_iter()
                    .find(|&nid| self.nav.data::<Uuid>(nid) == Some(&id))
                {
                    self.nav.text_set(nav_id, new_title.clone());
                }
                return self.save_active_board();
            }

            Message::DeleteBoard(id) => {
                // Remove from nav
                let ids: Vec<_> = self.nav.iter().collect();
                let nav_id = ids
                    .into_iter()
                    .find(|&nid| self.nav.data::<Uuid>(nid) == Some(&id));
                if let Some(nid) = nav_id {
                    self.nav.remove(nid);
                }
                self.boards.remove(&id);

                // Activate next available board
                let first_id = {
                    let ids: Vec<_> = self.nav.iter().collect();
                    ids.into_iter().next()
                };
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
                return Task::batch([task, self.update_title()]);
            }

            Message::BoardLoaded(board) => {
                self.boards.insert(board.id, board);
            }

            Message::CreateList => {
                let title = if let Some(ref t) = self.new_list_input {
                    t.trim().to_string()
                } else {
                    return Task::none();
                };
                if title.is_empty() {
                    return Task::none();
                }
                if let Some(board) = self.active_board_mut() {
                    let position = board.lists.len() as u32;
                    let list = List {
                        id: Uuid::new_v4(),
                        title,
                        cards: Vec::new(),
                        position,
                    };
                    board.lists.push(list);
                    board.updated_at = jiff::Timestamp::now();
                }
                self.new_list_input = None;
                return self.save_active_board();
            }

            Message::RenameList { list_id, new_title } => {
                if let Some(board) = self.active_board_mut() {
                    if let Some(list) = board.lists.iter_mut().find(|l| l.id == list_id) {
                        list.title = new_title;
                        board.updated_at = jiff::Timestamp::now();
                    }
                }
                return self.save_active_board();
            }

            Message::DeleteList(list_id) => {
                if let Some(board) = self.active_board_mut() {
                    board.lists.retain(|l| l.id != list_id);
                    // Re-number positions
                    for (i, list) in board.lists.iter_mut().enumerate() {
                        list.position = i as u32;
                    }
                    board.updated_at = jiff::Timestamp::now();
                }
                return self.save_active_board();
            }

            Message::CreateCard(list_id) => {
                let title = if let Some((lid, ref t)) = self.new_card_input {
                    if lid == list_id {
                        t.trim().to_string()
                    } else {
                        return Task::none();
                    }
                } else {
                    return Task::none();
                };
                if title.is_empty() {
                    return Task::none();
                }
                let now = jiff::Timestamp::now();
                if let Some(board) = self.active_board_mut() {
                    if let Some(list) = board.lists.iter_mut().find(|l| l.id == list_id) {
                        let position = list.cards.len() as u32;
                        let card = Card {
                            id: Uuid::new_v4(),
                            title,
                            description: String::new(),
                            labels: Vec::new(),
                            checklist: Vec::new(),
                            due_date: None,
                            position,
                            created_at: now,
                            updated_at: now,
                        };
                        list.cards.push(card);
                    }
                    board.updated_at = now;
                }
                self.new_card_input = None;
                return self.save_active_board();
            }

            Message::UpdateCardTitle { card_id, new_title } => {
                if let Some(board) = self.active_board_mut() {
                    for list in &mut board.lists {
                        if let Some(card) = list.cards.iter_mut().find(|c| c.id == card_id) {
                            card.title = new_title;
                            card.updated_at = jiff::Timestamp::now();
                            break;
                        }
                    }
                    board.updated_at = jiff::Timestamp::now();
                }
                return self.save_active_board();
            }

            Message::UpdateCardDescription {
                card_id,
                new_description,
            } => {
                if let Some(board) = self.active_board_mut() {
                    for list in &mut board.lists {
                        if let Some(card) = list.cards.iter_mut().find(|c| c.id == card_id) {
                            card.description = new_description;
                            card.updated_at = jiff::Timestamp::now();
                            break;
                        }
                    }
                    board.updated_at = jiff::Timestamp::now();
                }
                return self.save_active_board();
            }

            Message::DeleteCard(card_id) => {
                if let Some(board) = self.active_board_mut() {
                    for list in &mut board.lists {
                        if list.cards.iter().any(|c| c.id == card_id) {
                            list.cards.retain(|c| c.id != card_id);
                            // Re-number positions
                            for (i, card) in list.cards.iter_mut().enumerate() {
                                card.position = i as u32;
                            }
                            break;
                        }
                    }
                    board.updated_at = jiff::Timestamp::now();
                }
                if self.detail_card_id == Some(card_id) {
                    self.detail_card_id = None;
                    self.core.window.show_context = false;
                }
                return self.save_active_board();
            }

            Message::AddChecklistItem { card_id, text } => {
                if let Some(board) = self.active_board_mut() {
                    for list in &mut board.lists {
                        if let Some(card) = list.cards.iter_mut().find(|c| c.id == card_id) {
                            let position = card.checklist.len() as u32;
                            card.checklist.push(ChecklistItem {
                                id: Uuid::new_v4(),
                                text,
                                completed: false,
                                position,
                            });
                            card.updated_at = jiff::Timestamp::now();
                            break;
                        }
                    }
                    board.updated_at = jiff::Timestamp::now();
                }
                return self.save_active_board();
            }

            Message::ToggleChecklistItem { card_id, item_id } => {
                if let Some(board) = self.active_board_mut() {
                    for list in &mut board.lists {
                        if let Some(card) = list.cards.iter_mut().find(|c| c.id == card_id) {
                            if let Some(item) = card.checklist.iter_mut().find(|i| i.id == item_id)
                            {
                                item.completed = !item.completed;
                                card.updated_at = jiff::Timestamp::now();
                            }
                            break;
                        }
                    }
                    board.updated_at = jiff::Timestamp::now();
                }
                return self.save_active_board();
            }

            Message::DeleteChecklistItem { card_id, item_id } => {
                if let Some(board) = self.active_board_mut() {
                    for list in &mut board.lists {
                        if let Some(card) = list.cards.iter_mut().find(|c| c.id == card_id) {
                            card.checklist.retain(|i| i.id != item_id);
                            for (idx, item) in card.checklist.iter_mut().enumerate() {
                                item.position = idx as u32;
                            }
                            card.updated_at = jiff::Timestamp::now();
                            break;
                        }
                    }
                    board.updated_at = jiff::Timestamp::now();
                }
                return self.save_active_board();
            }

            Message::OpenCardDetail(card_id) => {
                self.detail_card_id = Some(card_id);
                self.calendar_visible = false;
                self.new_checklist_text = String::new();

                // Initialize calendar from card's due date or today
                let due_date = self.active_board().and_then(|board| {
                    board
                        .lists
                        .iter()
                        .flat_map(|l| l.cards.iter())
                        .find(|c| c.id == card_id)
                        .and_then(|c| c.due_date)
                });
                let date = due_date.unwrap_or_else(|| jiff::Zoned::now().date());
                self.calendar_model = widget::calendar::CalendarModel::new(date, date);

                self.context_page = ContextPage::CardDetail;
                self.core.window.show_context = true;
            }

            Message::ToggleContextPage(page) => {
                if self.context_page == page {
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    // Initialize state for specific pages
                    if page == ContextPage::BoardSettings {
                        self.rename_board_input = self
                            .active_board()
                            .map(|b| b.title.clone())
                            .unwrap_or_default();
                    }
                    self.context_page = page;
                    self.core.window.show_context = true;
                }
            }

            Message::OpenNewListInput => {
                self.new_list_input = Some(String::new());
                self.new_card_input = None;
            }

            Message::OpenNewCardInput(list_id) => {
                self.new_card_input = Some((list_id, String::new()));
                self.new_list_input = None;
            }

            Message::NewListInputChanged(text) => {
                if let Some(ref mut t) = self.new_list_input {
                    *t = text;
                }
            }

            Message::NewCardInputChanged(text) => {
                if let Some((_, ref mut t)) = self.new_card_input {
                    *t = text;
                }
            }

            Message::ConfirmNewList => {
                return self.update(Message::CreateList);
            }

            Message::ConfirmNewCard(list_id) => {
                return self.update(Message::CreateCard(list_id));
            }

            Message::DismissNewInput => {
                self.new_card_input = None;
                self.new_list_input = None;
            }

            Message::SaveActiveBoard => {
                return self.save_active_board();
            }

            Message::BoardSaved => {}

            Message::LoadError(err) => {
                eprintln!("Boards error: {err}");
            }

            Message::LaunchUrl(url) => {
                if let Err(e) = open::that_detached(&url) {
                    eprintln!("failed to open {url:?}: {e}");
                }
            }

            Message::UpdateConfig(config) => {
                self.config = config;
            }

            // ── Phase 2 handlers ──────────────────────────────────────────────
            Message::ToggleCalendar => {
                self.calendar_visible = !self.calendar_visible;
            }

            Message::CalendarPrevMonth => {
                self.calendar_model.show_prev_month();
            }

            Message::CalendarNextMonth => {
                self.calendar_model.show_next_month();
            }

            Message::CalendarYearChanged(year) => {
                let vis = self.calendar_model.visible;
                // Shift visible month to new year, clamping day if needed
                if let Ok(new_vis) = jiff::civil::Date::new(year as i16, vis.month(), vis.day()) {
                    self.calendar_model.visible = new_vis;
                } else if let Ok(new_vis) = jiff::civil::Date::new(year as i16, vis.month(), 28) {
                    self.calendar_model.visible = new_vis;
                }
                // Also shift selected
                let sel = self.calendar_model.selected;
                if let Ok(new_sel) = jiff::civil::Date::new(year as i16, sel.month(), sel.day()) {
                    self.calendar_model.selected = new_sel;
                } else if let Ok(new_sel) = jiff::civil::Date::new(year as i16, sel.month(), 28) {
                    self.calendar_model.selected = new_sel;
                }
            }

            Message::CalendarDateSelected(date) => {
                self.calendar_model.set_selected_visible(date);
                // Persist to the card
                if let Some(card_id) = self.detail_card_id {
                    if let Some(board) = self.active_board_mut() {
                        let now = jiff::Timestamp::now();
                        for list in &mut board.lists {
                            if let Some(card) = list.cards.iter_mut().find(|c| c.id == card_id) {
                                card.due_date = Some(date);
                                card.updated_at = now;
                                break;
                            }
                        }
                        board.updated_at = now;
                    }
                }
                return self.save_active_board();
            }

            Message::ClearDueDate => {
                if let Some(card_id) = self.detail_card_id {
                    if let Some(board) = self.active_board_mut() {
                        let now = jiff::Timestamp::now();
                        for list in &mut board.lists {
                            if let Some(card) = list.cards.iter_mut().find(|c| c.id == card_id) {
                                card.due_date = None;
                                card.updated_at = now;
                                break;
                            }
                        }
                        board.updated_at = now;
                    }
                }
                return self.save_active_board();
            }

            Message::ToggleLabel(color) => {
                if let Some(card_id) = self.detail_card_id {
                    if let Some(board) = self.active_board_mut() {
                        let now = jiff::Timestamp::now();
                        for list in &mut board.lists {
                            if let Some(card) = list.cards.iter_mut().find(|c| c.id == card_id) {
                                if let Some(pos) = card.labels.iter().position(|l| l.color == color)
                                {
                                    card.labels.remove(pos);
                                } else {
                                    card.labels.push(crate::models::label::Label {
                                        id: Uuid::new_v4(),
                                        name: color.name().to_string(),
                                        color,
                                    });
                                }
                                card.updated_at = now;
                                break;
                            }
                        }
                        board.updated_at = now;
                    }
                }
                return self.save_active_board();
            }

            Message::RemoveLabelFromCard { card_id, label_id } => {
                if let Some(board) = self.active_board_mut() {
                    let now = jiff::Timestamp::now();
                    for list in &mut board.lists {
                        if let Some(card) = list.cards.iter_mut().find(|c| c.id == card_id) {
                            card.labels.retain(|l| l.id != label_id);
                            card.updated_at = now;
                            break;
                        }
                    }
                    board.updated_at = now;
                }
                return self.save_active_board();
            }

            Message::NewChecklistItemChanged(text) => {
                self.new_checklist_text = text;
            }

            Message::ConfirmChecklistItem => {
                let text = self.new_checklist_text.trim().to_string();
                if text.is_empty() {
                    return Task::none();
                }
                self.new_checklist_text.clear();
                if let Some(card_id) = self.detail_card_id {
                    return self.update(Message::AddChecklistItem { card_id, text });
                }
            }

            Message::MoveCardUp(card_id) => {
                if let Some(board) = self.active_board_mut() {
                    let now = jiff::Timestamp::now();
                    for list in &mut board.lists {
                        if let Some(idx) = list.cards.iter().position(|c| c.id == card_id) {
                            if idx > 0 {
                                list.cards.swap(idx - 1, idx);
                                for (i, card) in list.cards.iter_mut().enumerate() {
                                    card.position = i as u32;
                                }
                                break;
                            }
                        }
                    }
                    board.updated_at = now;
                }
                return self.save_active_board();
            }

            Message::MoveCardDown(card_id) => {
                if let Some(board) = self.active_board_mut() {
                    let now = jiff::Timestamp::now();
                    for list in &mut board.lists {
                        let len = list.cards.len();
                        if let Some(idx) = list.cards.iter().position(|c| c.id == card_id) {
                            if idx + 1 < len {
                                list.cards.swap(idx, idx + 1);
                                for (i, card) in list.cards.iter_mut().enumerate() {
                                    card.position = i as u32;
                                }
                                break;
                            }
                        }
                    }
                    board.updated_at = now;
                }
                return self.save_active_board();
            }

            Message::MoveCardToList {
                card_id,
                target_list_id,
            } => {
                if let Some(board) = self.active_board_mut() {
                    let now = jiff::Timestamp::now();
                    // Remove card from its current list
                    let mut moved_card: Option<crate::models::card::Card> = None;
                    for list in &mut board.lists {
                        if let Some(idx) = list.cards.iter().position(|c| c.id == card_id) {
                            moved_card = Some(list.cards.remove(idx));
                            // Renumber source list
                            for (i, c) in list.cards.iter_mut().enumerate() {
                                c.position = i as u32;
                            }
                            break;
                        }
                    }
                    // Insert into target list
                    if let Some(mut card) = moved_card {
                        if let Some(target) =
                            board.lists.iter_mut().find(|l| l.id == target_list_id)
                        {
                            card.position = target.cards.len() as u32;
                            card.updated_at = now;
                            target.cards.push(card);
                        }
                    }
                    board.updated_at = now;
                }
                return self.save_active_board();
            }

            Message::RenameBoardInputChanged(text) => {
                self.rename_board_input = text;
            }

            Message::ConfirmBoardRename(board_id) => {
                let new_title = self.rename_board_input.trim().to_string();
                if new_title.is_empty() {
                    return Task::none();
                }
                return self.update(Message::RenameBoard {
                    id: board_id,
                    new_title,
                });
            }

            // ── Drag-and-drop ────────────────────────────────────────────
            Message::DragCardStarted(_card_id) => {
                // The card widget has already initiated the system DnD;
                // nothing extra needed in the model.
            }

            Message::DragHoverChanged {
                list_id,
                before_card_id,
            } => {
                self.drag_hover = Some((list_id, before_card_id));
            }

            Message::DragLeftDropZone => {
                self.drag_hover = None;
            }

            Message::DragCardCancelled => {
                self.drag_hover = None;
            }

            Message::DragCardDropped {
                card_id,
                target_list_id,
                before_card_id,
            } => {
                self.drag_hover = None;
                // No-op: dropping a card on itself.
                if before_card_id == Some(card_id) {
                    return Task::none();
                }
                if let Some(board) = self.active_board_mut() {
                    let now = jiff::Timestamp::now();

                    // Determine the real insertion anchor *before* mutating the
                    // list, so both indices are still valid.
                    //
                    // Dropping card X onto card Y means "X takes Y's place":
                    //   • Moving DOWN (X above Y in the same list): insert X
                    //     *after* Y (before Y's successor), so Y slides up.
                    //   • Moving UP or cross-list: insert X *before* Y
                    //     (existing behaviour, already correct).
                    let actual_before_id: Option<Uuid> = match before_card_id {
                        None => None, // append zone — no adjustment needed
                        Some(target_card_id) => {
                            let target_list = board.lists.iter().find(|l| l.id == target_list_id);
                            let src_idx = target_list
                                .and_then(|l| l.cards.iter().position(|c| c.id == card_id));
                            let tgt_idx = target_list
                                .and_then(|l| l.cards.iter().position(|c| c.id == target_card_id));

                            match (src_idx, tgt_idx) {
                                (Some(s), Some(t)) if s < t => {
                                    // Moving down: anchor on Y's successor.
                                    // If Y is the last card this yields None → append.
                                    board
                                        .lists
                                        .iter()
                                        .find(|l| l.id == target_list_id)
                                        .and_then(|l| l.cards.get(t + 1))
                                        .map(|c| c.id)
                                }
                                _ => Some(target_card_id), // moving up / cross-list
                            }
                        }
                    };

                    // Remove card from its current list
                    let mut moved_card: Option<crate::models::card::Card> = None;
                    for list in &mut board.lists {
                        if let Some(idx) = list.cards.iter().position(|c| c.id == card_id) {
                            moved_card = Some(list.cards.remove(idx));
                            for (i, c) in list.cards.iter_mut().enumerate() {
                                c.position = i as u32;
                            }
                            break;
                        }
                    }
                    // Insert into target list at the right position
                    if let Some(mut card) = moved_card {
                        if let Some(target) =
                            board.lists.iter_mut().find(|l| l.id == target_list_id)
                        {
                            let insert_at = match actual_before_id {
                                Some(bid) => target
                                    .cards
                                    .iter()
                                    .position(|c| c.id == bid)
                                    .unwrap_or(target.cards.len()),
                                None => target.cards.len(),
                            };
                            card.updated_at = now;
                            target.cards.insert(insert_at, card);
                            for (i, c) in target.cards.iter_mut().enumerate() {
                                c.position = i as u32;
                            }
                        }
                    }
                    board.updated_at = now;
                }
                return self.save_active_board();
            }
        }

        Task::none()
    }

    /// Called when a nav item is selected.
    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<cosmic::Action<Self::Message>> {
        self.nav.activate(id);
        self.new_card_input = None;
        self.new_list_input = None;
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
    fn active_board_id(&self) -> Option<Uuid> {
        self.nav.active_data::<Uuid>().copied()
    }

    fn active_board(&self) -> Option<&Board> {
        self.active_board_id().and_then(|id| self.boards.get(&id))
    }

    fn active_board_mut(&mut self) -> Option<&mut Board> {
        let id = self.active_board_id()?;
        self.boards.get_mut(&id)
    }

    fn save_board_task(&self, board: Board) -> Task<cosmic::Action<Message>> {
        Task::perform(
            async move { Store::new().save_board(&board) },
            |result| match result {
                Ok(_) => cosmic::Action::App(Message::BoardSaved),
                Err(e) => cosmic::Action::App(Message::LoadError(e.to_string())),
            },
        )
    }

    fn save_active_board(&self) -> Task<cosmic::Action<Message>> {
        if let Some(board) = self.active_board() {
            let board = board.clone();
            self.save_board_task(board)
        } else {
            Task::none()
        }
    }

    /// Updates the header and window titles.
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

    fn view_empty_state(&self) -> Element<'_, Message> {
        let space_m = cosmic::theme::spacing().space_m;

        let content = widget::column::with_capacity(3)
            .push(widget::text::title2(fl!("no-boards")))
            .push(widget::Space::new().height(space_m))
            .push(
                widget::button::suggested(fl!("create-first-board"))
                    .on_press(Message::ToggleContextPage(ContextPage::NewBoard)),
            )
            .align_x(Alignment::Center)
            .spacing(space_m);

        widget::container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(cosmic::iced::alignment::Horizontal::Center)
            .align_y(cosmic::iced::alignment::Vertical::Center)
            .into()
    }

    fn view_board_with_new_list<'a>(&'a self, board: &'a Board) -> Element<'a, Message> {
        let space_m = cosmic::theme::spacing().space_m;
        let space_l = cosmic::theme::spacing().space_l;

        let mut row = widget::row::with_capacity(board.lists.len() + 1)
            .spacing(space_m)
            .padding([space_m, space_l])
            .height(Length::Fill);

        for list in &board.lists {
            // Decompose the drag hover state for this specific list.
            let (drag_hovered_card_id, tail_active) = match self.drag_hover {
                Some((lid, Some(cid))) if lid == list.id => (Some(cid), false),
                Some((lid, None)) if lid == list.id => (None, true),
                _ => (None, false),
            };
            let col = if let Some((ref active_list_id, ref input_text)) = self.new_card_input {
                if *active_list_id == list.id {
                    view_list_with_input(list, input_text, drag_hovered_card_id, tail_active)
                } else {
                    view_list(list, drag_hovered_card_id, tail_active)
                }
            } else {
                view_list(list, drag_hovered_card_id, tail_active)
            };
            row = row.push(col);
        }

        // "Add list" inline input column
        if let Some(ref input_text) = self.new_list_input {
            let add_list_col = widget::container(new_list_input(input_text))
                .width(280)
                .class(cosmic::theme::Container::Secondary);
            row = row.push(add_list_col);
        }

        widget::scrollable::horizontal(row)
            .height(Length::Fill)
            .into()
    }

    fn view_new_board_form(&self) -> Element<'_, Message> {
        let space_s = cosmic::theme::spacing().space_s;
        let space_m = cosmic::theme::spacing().space_m;

        let title_input = widget::text_input(fl!("board-title"), self.new_board_title.as_str())
            .on_input(Message::NewBoardTitleChanged)
            .on_submit(|_| Message::CreateBoard)
            .width(Length::Fill);

        let create_btn = widget::button::suggested(fl!("new-board")).on_press(Message::CreateBoard);

        widget::column::with_capacity(3)
            .push(widget::text::heading(fl!("board-title")))
            .push(title_input)
            .push(create_btn)
            .spacing(space_s)
            .padding(space_m)
            .into()
    }

    fn view_board_settings(&self) -> Element<'_, Message> {
        let space_s = cosmic::theme::spacing().space_s;
        let space_m = cosmic::theme::spacing().space_m;

        let Some(board) = self.active_board() else {
            return widget::text::body("No board selected.").into();
        };
        let board_id = board.id;

        // Rename section
        let rename_input = widget::text_input(fl!("board-name"), self.rename_board_input.as_str())
            .on_input(Message::RenameBoardInputChanged)
            .on_submit(move |_| Message::ConfirmBoardRename(board_id))
            .width(Length::Fill);

        let rename_btn = widget::button::suggested(fl!("rename-board"))
            .on_press(Message::ConfirmBoardRename(board_id));

        let delete_btn =
            widget::button::destructive(fl!("delete")).on_press(Message::DeleteBoard(board_id));

        widget::column::with_capacity(7)
            .push(widget::text::heading(board.title.as_str()))
            .push(widget::divider::horizontal::default())
            .push(widget::text::body(fl!("board-name")))
            .push(rename_input)
            .push(rename_btn)
            .push(widget::divider::horizontal::default())
            .push(delete_btn)
            .spacing(space_s)
            .padding(space_m)
            .into()
    }

    fn view_card_detail(&self) -> Element<'_, Message> {
        use cosmic::iced::widget::container::Style as ContainerStyle;
        use cosmic::iced::{Background, Border, Color};

        let space_xxs = cosmic::theme::spacing().space_xxs;
        let space_xs = cosmic::theme::spacing().space_xs;
        let space_s = cosmic::theme::spacing().space_s;
        let space_m = cosmic::theme::spacing().space_m;

        let Some(card_id) = self.detail_card_id else {
            return widget::text::body("No card selected.").into();
        };
        let Some(board) = self.active_board() else {
            return widget::text::body("No board.").into();
        };

        let mut found_card: Option<&Card> = None;
        let mut list_name = String::new();
        let mut current_list_id = Uuid::nil();
        for list in &board.lists {
            if let Some(card) = list.cards.iter().find(|c| c.id == card_id) {
                found_card = Some(card);
                list_name = list.title.clone();
                current_list_id = list.id;
                break;
            }
        }
        let Some(card) = found_card else {
            return widget::text::body("Card not found.").into();
        };

        // ── Title ──
        let title_input = widget::text_input("Card title…", card.title.as_str())
            .on_input(move |t| Message::UpdateCardTitle {
                card_id,
                new_title: t,
            })
            .on_submit(move |_| Message::SaveActiveBoard)
            .width(Length::Fill);

        let in_list = widget::text::caption(format!("{}: {}", fl!("in-list"), list_name));

        // ── Due Date ──
        let due_date_text = match card.due_date {
            Some(d) => format!("📅 {d}"),
            None => fl!("no-due-date"),
        };
        let mut due_row = widget::row::with_capacity(3)
            .spacing(space_xs)
            .align_y(cosmic::iced::Alignment::Center);
        due_row = due_row.push(widget::text::body(due_date_text));
        if card.due_date.is_some() {
            due_row = due_row
                .push(widget::button::text(fl!("clear-date")).on_press(Message::ClearDueDate));
        }
        let toggle_label = if self.calendar_visible {
            fl!("hide-date-picker")
        } else {
            fl!("pick-date")
        };
        due_row =
            due_row.push(widget::button::text(toggle_label).on_press(Message::ToggleCalendar));

        // ── Calendar (shown when calendar_visible) ──
        let calendar_section: Option<Element<'_, Message>> = if self.calendar_visible {
            let year = self.calendar_model.visible.year() as i32;
            let year_spin = widget::spin_button(
                year.to_string(), // label
                "Year",           // a11y name
                year,             // value
                1i32,             // step
                1900i32,          // min
                2200i32,          // max
                |y| Message::CalendarYearChanged(y),
            );
            let year_row = widget::row::with_capacity(2)
                .push(widget::text::body("Year:"))
                .push(year_spin)
                .spacing(space_s)
                .align_y(cosmic::iced::Alignment::Center);

            let cal = widget::calendar::calendar(
                &self.calendar_model,
                |date| Message::CalendarDateSelected(date),
                || Message::CalendarPrevMonth,
                || Message::CalendarNextMonth,
                jiff::civil::Weekday::Monday,
            );

            Some(
                widget::column::with_capacity(2)
                    .push(year_row)
                    .push(cal)
                    .spacing(space_s)
                    .into(),
            )
        } else {
            None
        };

        // ── Labels ──
        // Palette: one colored square per LabelColor, border when active
        let all_colors = [
            LabelColor::Red,
            LabelColor::Orange,
            LabelColor::Yellow,
            LabelColor::Green,
            LabelColor::Blue,
            LabelColor::Purple,
            LabelColor::Pink,
            LabelColor::Teal,
            LabelColor::Gray,
        ];
        let mut palette_row = widget::row::with_capacity(all_colors.len()).spacing(space_xs);
        for &color in &all_colors {
            let iced_color = color.as_color();
            let is_active = card.labels.iter().any(|l| l.color == color);
            let dot = widget::container(widget::Space::new().width(28).height(28)).class(
                cosmic::theme::Container::custom(move |_theme| ContainerStyle {
                    background: Some(Background::Color(iced_color)),
                    border: Border {
                        radius: 4.0.into(),
                        width: if is_active { 3.0 } else { 0.0 },
                        color: Color::WHITE,
                    },
                    ..Default::default()
                }),
            );
            palette_row =
                palette_row.push(widget::mouse_area(dot).on_press(Message::ToggleLabel(color)));
        }

        // Currently active labels as pills
        let mut labels_row = widget::row::with_capacity(card.labels.len()).spacing(space_xxs);
        for label in &card.labels {
            let lc = label.color.as_color();
            let lid = label.id;
            let pill = widget::container(widget::text::caption(label.name.as_str()))
                .padding([space_xxs, space_xs])
                .class(cosmic::theme::Container::custom(move |_theme| {
                    ContainerStyle {
                        background: Some(Background::Color(lc)),
                        border: Border {
                            radius: 4.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                }));
            let remove_btn = widget::button::icon(widget::icon::from_name("window-close-symbolic"))
                .on_press(Message::RemoveLabelFromCard {
                    card_id,
                    label_id: lid,
                });
            labels_row = labels_row.push(
                widget::row::with_capacity(2)
                    .push(pill)
                    .push(remove_btn)
                    .align_y(cosmic::iced::Alignment::Center)
                    .spacing(space_xxs),
            );
        }

        let labels_col = {
            let mut col = widget::column::with_capacity(3)
                .push(widget::text::heading(fl!("labels")))
                .push(palette_row)
                .spacing(space_xs);
            if !card.labels.is_empty() {
                col = col.push(labels_row);
            }
            col
        };

        // ── Description ──
        let desc_input = widget::text_input("Add a description…", card.description.as_str())
            .on_input(move |d| Message::UpdateCardDescription {
                card_id,
                new_description: d,
            })
            .on_submit(move |_| Message::SaveActiveBoard)
            .width(Length::Fill);

        // ── Checklist ──
        let total = card.checklist.len();
        let done = card.checklist.iter().filter(|i| i.completed).count();
        let progress_text = if total > 0 {
            format!("{done}/{total}")
        } else {
            String::new()
        };
        let mut checklist_header =
            widget::row::with_capacity(2).align_y(cosmic::iced::Alignment::Center);
        checklist_header =
            checklist_header.push(widget::text::heading(fl!("checklist")).width(Length::Fill));
        if total > 0 {
            checklist_header = checklist_header.push(widget::text::caption(progress_text));
        }

        let mut checklist_col =
            widget::column::with_capacity(card.checklist.len() + 2).spacing(space_xs);
        for item in &card.checklist {
            let item_id = item.id;
            let check = widget::checkbox(item.completed)
                .label(item.text.as_str())
                .on_toggle(move |_: bool| Message::ToggleChecklistItem { card_id, item_id })
                .width(Length::Fill);
            let del_btn = widget::button::icon(widget::icon::from_name("user-trash-symbolic"))
                .on_press(Message::DeleteChecklistItem { card_id, item_id });
            checklist_col = checklist_col.push(
                widget::row::with_capacity(2)
                    .push(check)
                    .push(del_btn)
                    .align_y(cosmic::iced::Alignment::Center),
            );
        }
        // Add-item input row
        let add_item_input = widget::text_input("New item…", self.new_checklist_text.as_str())
            .on_input(Message::NewChecklistItemChanged)
            .on_submit(|_| Message::ConfirmChecklistItem)
            .width(Length::Fill);
        let add_item_btn =
            widget::button::suggested(fl!("add-item")).on_press(Message::ConfirmChecklistItem);
        checklist_col = checklist_col.push(
            widget::row::with_capacity(2)
                .push(add_item_input)
                .push(add_item_btn)
                .spacing(space_xs)
                .align_y(cosmic::iced::Alignment::Center),
        );

        // ── Move card ──
        // Compute card position info
        let card_idx = board
            .lists
            .iter()
            .find(|l| l.id == current_list_id)
            .and_then(|l| l.cards.iter().position(|c| c.id == card_id));
        let list_len = board
            .lists
            .iter()
            .find(|l| l.id == current_list_id)
            .map(|l| l.cards.len())
            .unwrap_or(0);

        // Move up / move down buttons using Vec to avoid is_empty on Row
        let mut order_items: Vec<Element<'_, Message>> = Vec::new();
        if card_idx.map(|i| i > 0).unwrap_or(false) {
            order_items.push(
                widget::button::text(fl!("move-up"))
                    .on_press(Message::MoveCardUp(card_id))
                    .into(),
            );
        }
        if card_idx.map(|i| i + 1 < list_len).unwrap_or(false) {
            order_items.push(
                widget::button::text(fl!("move-down"))
                    .on_press(Message::MoveCardDown(card_id))
                    .into(),
            );
        }

        // Other lists to move card into
        let other_lists: Vec<_> = board
            .lists
            .iter()
            .filter(|l| l.id != current_list_id)
            .collect();

        let move_section: Option<Element<'_, Message>> = if !other_lists.is_empty()
            || !order_items.is_empty()
        {
            let mut move_col = widget::column::with_capacity(4)
                .push(widget::text::heading(fl!("move-card")))
                .spacing(space_xs);

            if !other_lists.is_empty() {
                let mut move_row = widget::row::with_capacity(other_lists.len()).spacing(space_xs);
                for list in &other_lists {
                    let target_id = list.id;
                    move_row = move_row.push(widget::button::text(list.title.as_str()).on_press(
                        Message::MoveCardToList {
                            card_id,
                            target_list_id: target_id,
                        },
                    ));
                }
                move_col = move_col.push(move_row);
            }

            if !order_items.is_empty() {
                move_col = move_col.push(widget::row::with_children(order_items).spacing(space_xs));
            }

            Some(move_col.into())
        } else {
            None
        };

        // ── Delete ──
        let delete_btn =
            widget::button::destructive(fl!("delete-card")).on_press(Message::DeleteCard(card_id));

        // ── Assemble all sections ──
        let mut col = widget::column::with_capacity(20)
            .spacing(space_s)
            .padding(space_m);

        col = col
            .push(title_input)
            .push(in_list)
            .push(widget::divider::horizontal::default())
            // Due date
            .push(widget::text::heading(fl!("due-date")))
            .push(due_row);

        if let Some(cal_section) = calendar_section {
            col = col.push(cal_section);
        }

        col = col
            .push(widget::divider::horizontal::default())
            // Labels
            .push(labels_col)
            .push(widget::divider::horizontal::default())
            // Description
            .push(widget::text::heading(fl!("description")))
            .push(desc_input)
            .push(widget::divider::horizontal::default())
            // Checklist
            .push(checklist_header)
            .push(checklist_col)
            .push(widget::divider::horizontal::default());

        if let Some(mv) = move_section {
            col = col.push(mv).push(widget::divider::horizontal::default());
        }

        col = col.push(delete_btn);

        widget::scrollable::vertical(col).into()
    }
}

/// The context drawer page.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
    NewBoard,
    BoardSettings,
    CardDetail,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
    NewBoard,
    BoardSettings,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
            MenuAction::NewBoard => Message::ToggleContextPage(ContextPage::NewBoard),
            MenuAction::BoardSettings => Message::ToggleContextPage(ContextPage::BoardSettings),
        }
    }
}
