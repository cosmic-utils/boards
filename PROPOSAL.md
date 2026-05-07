# Boards — Trello-like App for COSMIC: Architecture Proposal

> A native Kanban board application for the COSMIC desktop, built with `libcosmic`.

---

## Table of Contents

1. [Vision & Core Concepts](#vision--core-concepts)
2. [UI / UX Layout](#ui--ux-layout)
3. [Module Structure](#module-structure)
4. [Data Models](#data-models)
5. [Persistence Strategy](#persistence-strategy)
6. [Message Architecture](#message-architecture)
7. [AppModel Design](#appmodel-design)
8. [Widget Design](#widget-design)
9. [Feature Roadmap](#feature-roadmap)
10. [Dependencies](#dependencies)

---

## Vision & Core Concepts

**Boards** is a local-first, offline Kanban board application for the COSMIC desktop. It mirrors the core workflow of Trello while integrating natively with the COSMIC design language and theming system.

### Hierarchy

```
Workspace
 └── Board  (e.g. "Home Renovation", "Work Sprint")
      └── List / Column  (e.g. "To Do", "In Progress", "Done")
           └── Card  (e.g. "Buy paint", "Fix login bug")
                ├── Description (Markdown)
                ├── Labels (color tags)
                ├── Checklist items
                └── Due date
```

---

## UI / UX Layout

Boards live in the **COSMIC nav bar** (left sidebar). Selecting a board immediately shows its Kanban columns in the main content area — no separate home screen.

### Main Window Layout

```
┌────────────────────────────────────────────────────────────────────────────────────┐
│  [≡ View]   Work Sprint                                      [+ Add list]  [⋮]    │  ← Header bar
├───────────────────┬────────────────────────────────────────────────────────────────┤
│  📋 Boards        │  ←──────────────── Horizontally Scrollable ─────────────────→  │
│  ─────────────    │                                                                │
│  ▶ Work Sprint   │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│    Personal       │  │ To Do        [+]│  │ In Progress  [+]│  │ Done         [+]│ │
│    Home Reno      │  ├─────────────────┤  ├─────────────────┤  ├─────────────────┤ │
│  ─────────────    │  │ ┌─────────────┐ │  │ ┌─────────────┐ │  │ ┌─────────────┐ │ │
│  [+ New Board]    │  │ │ 🔴 Bug      │ │  │ │ 🟡 Feature  │ │  │ │ 🟢 Released │ │ │
│                   │  │ │ Fix login   │ │  │ │ Dark mode   │ │  │ │ v1.0 done   │ │ │
│                   │  │ │ crash       │ │  │ └─────────────┘ │  │ └─────────────┘ │ │
│                   │  │ └─────────────┘ │  │ ┌─────────────┐ │  │ ┌─────────────┐ │ │
│                   │  │ ┌─────────────┐ │  │ │ 🔵 Chore    │ │  │ │ 🟢 CI done  │ │ │
│                   │  │ │ 🟡 Feature  │ │  │ │ Refactor DB │ │  │ └─────────────┘ │ │
│                   │  │ │ Add export  │ │  │ └─────────────┘ │  │                 │ │
│                   │  │ └─────────────┘ │  │                 │  │ [+ Add a card]  │ │
│                   │  │ [+ Add a card]  │  │ [+ Add a card]  │  └─────────────────┘ │
│                   │  └─────────────────┘  └─────────────────┘                     │
└───────────────────┴────────────────────────────────────────────────────────────────┘
```

### Empty State (no boards yet)

```
┌────────────────────────────────────────────────────────────────┐
│  [≡ View]   Boards                                             │
├────────────────┬───────────────────────────────────────────────┤
│  📋 Boards     │                                               │
│  ──────────    │         No boards yet.                        │
│                │                                               │
│  [+ New Board] │     [ + Create your first board ]            │
│                │                                               │
└────────────────┴───────────────────────────────────────────────┘
```

### Card Detail — Context Drawer (right side)

_(Slides in from the right when a card is clicked, without leaving the board view)_

### Kanban Columns — Board View
### Kanban Columns — Detail

Each column has a **pinned header** (title + add-card button) and a **vertically scrollable card area** that fills the remaining height. All columns scroll independently.

```
  ←──────────────────────────── Horizontally Scrollable ──────────────────────→
  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐  [+ List]
  │ To Do         [+]│  │ In Progress   [+]│  │ Done          [+]│  ← pinned header
  ├──────────────────┤  ├──────────────────┤  ├──────────────────┤
  │ ┌──────────────┐ │↑ │ ┌──────────────┐ │↑ │ ┌──────────────┐ │↑
  │ │ 🔴 Bug Fix   │ │  │ │ 🟡 Feature   │ │  │ │ 🟢 Released  │ │ ← vertically
  │ │ Fix login    │ │  │ │ Dark mode    │ │  │ │ v1.0 shipped │ │   scrollable
  │ │ page crash   │ │  │ │ support      │ │  │ │              │ │   area
  │ │ ─────────── │ │  │ │ ─────────── │ │  │ └──────────────┘ │
  │ │ ☐ Write test│ │  │ │ Due: Jan 15  │ │  │ ┌──────────────┐ │
  │ │ ☑ Repro'd   │ │  │ └──────────────┘ │  │ │ 🟢 Deployed  │ │
  │ └──────────────┘ │  │ ┌──────────────┐ │  │ │ CI pipeline  │ │
  │ ┌──────────────┐ │  │ │ 🔵 Chore     │ │  │ └──────────────┘ │
  │ │ 🟡 Feature   │ │↓ │ │ Refactor DB  │ │↓ │                  │↓
  │ │ Add export   │ │  │ └──────────────┘ │  │                  │
  │ └──────────────┘ │  │                  │  └──────────────────┘
  ├──────────────────┤  ├──────────────────┤  ← pinned footer
  │ [+ Add a card]   │  │ [+ Add a card]   │
  └──────────────────┘  └──────────────────┘
```

**Column layout anatomy:**
```
  ┌─────────────────────────────────────────────────────────────┐
  │  header  │ title (editable_input)   [+ add card icon btn]  │ height::Shrink (pinned)
  ├─────────────────────────────────────────────────────────────┤
  │          │ ┌─────────┐                                      │
  │ vertical │ │ card 1  │                                      │
  │  scroll  │ ├─────────┤                                      │ height::Fill
  │   area   │ │ card 2  │                                      │ (takes all
  │          │ ├─────────┤                                      │  remaining
  │          │ │ card 3  │  ← may extend beyond visible area    │  space)
  │          │ └─────────┘                                      │
  ├─────────────────────────────────────────────────────────────┤
  │  footer  │ [+ Add a card]                                   │ height::Shrink (pinned)
  └─────────────────────────────────────────────────────────────┘
```

### Card Detail — Context Drawer

```
┌───────────────────────────────────────────┐
│  Fix login page crash            [✕ Close]│
│  In list: To Do                           │
├───────────────────────────────────────────┤
│  Labels                                   │
│  [🔴 Bug] [+ Add label]                   │
│                                           │
│  Due Date                                 │
│  [📅 Pick a date…]                        │
│                                           │
│  Description                              │
│  ┌─────────────────────────────────────┐  │
│  │ The login page crashes when the     │  │
│  │ user enters an empty password.      │  │
│  │ Steps to reproduce:                 │  │
│  │ 1. Go to /login                     │  │
│  └─────────────────────────────────────┘  │
│                                           │
│  Checklist                      [+ Item]  │
│  ☑ Reproduce the bug                      │
│  ☑ Write failing test                     │
│  ☐ Fix the crash                          │
│  ☐ Open PR                                │
│                                           │
│  ──────────────────────────────────────   │
│  [Move to list ▾]  [🗑 Delete card]       │
└───────────────────────────────────────────┘
```

---

## Module Structure

```
src/
├── main.rs                  # Entry point, settings, run
├── app.rs                   # AppModel, nav_bar population, top-level view/update
├── config.rs                # Persisted COSMIC config (last active board ID, etc.)
├── i18n.rs                  # i18n initialization
│
├── models/
│   ├── mod.rs               # Re-exports
│   ├── board.rs             # Board struct + serde
│   ├── list.rs              # List (column) struct + serde
│   ├── card.rs              # Card struct + serde
│   ├── label.rs             # Label + LabelColor enum
│   └── checklist.rs         # ChecklistItem struct
│
├── pages/
│   ├── mod.rs               # Page enum (Board | Empty)
│   └── board.rs             # Board view: horizontal scrollable column layout
│
├── widgets/
│   ├── mod.rs               # Re-exports
│   ├── card_widget.rs       # Individual card widget (dnd_source wrapper)
│   ├── list_column.rs       # Column widget (dnd_destination wrapper)
│   ├── new_card_input.rs    # Inline "add card" / "add list" text input
│   └── label_badge.rs       # Colored label pill widget
│
└── db/
    ├── mod.rs               # Storage trait definition
    └── store.rs        # TOML file-based persistence (XDG data dir)
```

> **Note:** There is no dedicated `home.rs` page. The nav bar replaces the home screen entirely. The `Page::Empty` variant is shown only when no boards exist yet.

---

## Data Models

### `Board`

```src/models/board.rs#L1-30
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::list::List;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    /// Board header background colour — serialises as a TOML inline table
    #[serde(default = "default_background")]
    pub background: cosmic::iced::Color,
    /// Ordered list of columns
    pub lists: Vec<List>,
    /// RFC 3339 UTC timestamp — serialises as `"2025-01-15T09:00:00Z"`
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}

fn default_background() -> cosmic::iced::Color {
    cosmic::iced::Color::from_rgb8(54, 95, 168) // a pleasant default blue
}
```

### `List` (Column)

```src/models/list.rs#L1-20
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::card::Card;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct List {
    pub id: Uuid,
    pub title: String,
    /// Ordered list of cards
    pub cards: Vec<Card>,
    /// Sort position (for ordering columns)
    pub position: u32,
}
```

### `Card`

```src/models/card.rs#L1-45
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::{label::Label, checklist::ChecklistItem};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: Uuid,
    pub title: String,
    /// Markdown-formatted description
    pub description: String,
    pub labels: Vec<Label>,
    pub checklist: Vec<ChecklistItem>,
    /// Civil (timezone-free) date — serialises as `"2025-01-25"`
    pub due_date: Option<jiff::civil::Date>,
    /// Sort position within its list
    pub position: u32,
    /// RFC 3339 UTC timestamp — serialises as `"2025-01-15T09:00:00Z"`
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}
```

### `Label`

```src/models/label.rs#L1-30
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Label {
    pub id: Uuid,
    pub name: String,
    pub color: LabelColor,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum LabelColor {
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Purple,
    Pink,
    Teal,
    Gray,
}

impl LabelColor {
    /// Returns the hex color string for rendering
    pub fn as_hex(&self) -> &'static str {
        match self {
            LabelColor::Red    => "#ef4444",
            LabelColor::Orange => "#f97316",
            LabelColor::Yellow => "#eab308",
            LabelColor::Green  => "#22c55e",
            LabelColor::Blue   => "#3b82f6",
            LabelColor::Purple => "#a855f7",
            LabelColor::Pink   => "#ec4899",
            LabelColor::Teal   => "#14b8a6",
            LabelColor::Gray   => "#6b7280",
        }
    }
}
```

### `ChecklistItem`

```src/models/checklist.rs#L1-12
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub id: Uuid,
    pub text: String,
    pub completed: bool,
    pub position: u32,
}
```

---

## Persistence Strategy

All data is stored **locally** in the XDG data directory (`~/.local/share/dev.edfloreshz.Boards/`). TOML is chosen over JSON because it is human-readable and hand-editable — a user can open any board file in a text editor, fix a typo, or copy data between boards without needing special tooling.

```
~/.local/share/dev.edfloreshz.Boards/
├── boards.index.toml        # List of all board IDs + titles (for nav/home screen)
├── boards/
│   ├── <uuid>.toml          # Full board data (lists + cards) per board
│   └── <uuid>.toml
└── labels.toml              # Global label palette (shared across boards)
```

A board file looks like this on disk:

```boards/PROPOSAL.md#L1-1
# ~/.local/share/dev.edfloreshz.Boards/boards/550e8400-e29b-41d4-a716-446655440000.toml

title = "Work Sprint"
description = ""
created_at = "2025-01-15T09:00:00Z"
updated_at = "2025-01-20T14:32:00Z"

[background]
r = 0.22
g = 0.47
b = 0.94
a = 1.0

[[lists]]
id = "a1b2c3d4-..."
title = "To Do"
position = 0

[[lists.cards]]
id = "f1e2d3c4-..."
title = "Fix login page crash"
description = "Crashes when password field is empty."
position = 0
due_date = "2025-01-25"

[[lists.cards.checklist]]
id = "11223344-..."
text = "Reproduce the bug"
completed = true
position = 0
```

### Storage Trait

```src/db/mod.rs#L1-30
use crate::models::board::Board;
use uuid::Uuid;

pub trait DataStore {
    type Error: std::error::Error;

    /// Load all board summaries (id + title + color only)
    fn load_board_index(&self) -> Result<Vec<BoardSummary>, Self::Error>;

    /// Load a full board with all its lists and cards
    fn load_board(&self, id: Uuid) -> Result<Board, Self::Error>;

    /// Persist a full board
    fn save_board(&self, board: &Board) -> Result<(), Self::Error>;

    /// Delete a board and its data file
    fn delete_board(&self, id: Uuid) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BoardSummary {
    pub id: Uuid,
    pub title: String,
    pub background: cosmic::iced::Color,
    pub list_count: usize,
    pub card_count: usize,
}
```

### `Store` Implementation

```src/db/store.rs#L1-65
use std::io::Write;
use tempfile::NamedTempFile;

pub struct Store {
    data_dir: PathBuf,
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

    /// Write `content` to `path` atomically:
    /// 1. Create a NamedTempFile in the **same directory** (guarantees same filesystem).
    /// 2. Write and sync the data to disk.
    /// 3. Rename the temp file over the target — POSIX rename(2) is atomic.
    /// If the process crashes at any point before step 3, the target is untouched.
    fn write_atomic(&self, path: &Path, content: &str) -> Result<(), StoreError> {
        let dir = path.parent().expect("path has no parent");
        let mut tmp = NamedTempFile::new_in(dir)?;
        tmp.write_all(content.as_bytes())?;
        tmp.flush()?;
        tmp.as_file().sync_all()?;  // flush OS buffers to disk before rename
        tmp.persist(path)?;         // atomic rename: target is never partially written
        Ok(())
    }
}

impl DataStore for Store {
    type Error = StoreError;

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
        std::fs::remove_file(self.board_path(id))?;
        self.rebuild_index()
    }
    // ...
}
```

Saves are **debounced**: after any mutation the app schedules a `tokio::time::sleep(200ms)` task. If another mutation arrives before the timeout, it is cancelled and reset — preventing excessive disk writes during rapid editing.

### TOML-specific Notes

- **`iced::Color`** serialises as an inline TOML table `{ r = 0.22, g = 0.47, b = 0.94, a = 1.0 }` which is valid and human-readable. No wrapper needed — `iced_core` is already compiled with `features = ["serde"]` by libcosmic.
- **`jiff::civil::Date`** serialises as a quoted string `"2025-01-25"` and **`jiff::Timestamp`** as `"2025-01-15T09:00:00Z"`. Both use strings rather than TOML's native datetime tokens, which round-trips correctly with the `toml` crate.
- **`uuid::Uuid`** serialises as a string `"550e8400-e29b-41d4-a716-446655440000"`, which is fine in TOML.
- **Nested arrays** (`[[lists]]`, `[[lists.cards]]`) are standard TOML array-of-tables syntax and are fully supported by the `toml` crate.

---

## Message Architecture

```src/app.rs#L1-80
#[derive(Debug, Clone)]
pub enum Message {
    // ── Board CRUD ───────────────────────────────────────────────────
    /// Called when the user confirms the "New Board" dialog
    CreateBoard { title: String, background: cosmic::iced::Color },
    RenameBoard { id: Uuid, new_title: String },
    SetBoardBackground { id: Uuid, background: cosmic::iced::Color },
    DeleteBoard(Uuid),
    /// Board data successfully loaded from disk (async result)
    BoardLoaded(Board),

    // ── List (Column) CRUD ───────────────────────────────────────────
    CreateList { title: String },           // always on the active board
    RenameList { list_id: Uuid, new_title: String },
    DeleteList(Uuid),
    MoveList { list_id: Uuid, new_position: u32 },

    // ── Card CRUD ────────────────────────────────────────────────────
    CreateCard { list_id: Uuid, title: String },
    UpdateCardTitle { card_id: Uuid, new_title: String },
    UpdateCardDescription { card_id: Uuid, new_description: String },
    SetCardDueDate { card_id: Uuid, date: Option<jiff::civil::Date> },
    DeleteCard(Uuid),
    MoveCard { card_id: Uuid, target_list_id: Uuid, new_position: u32 },

    // ── Labels ───────────────────────────────────────────────────────
    AddLabelToCard { card_id: Uuid, label: Label },
    RemoveLabelFromCard { card_id: Uuid, label_id: Uuid },

    // ── Checklist ────────────────────────────────────────────────────
    AddChecklistItem { card_id: Uuid, text: String },
    ToggleChecklistItem { card_id: Uuid, item_id: Uuid },
    DeleteChecklistItem { card_id: Uuid, item_id: Uuid },

    // ── Drag & Drop ──────────────────────────────────────────────────
    DragCardStarted(Uuid),
    DragCardEnteredList(Uuid),
    DragCardLeftList,
    DragCardDropped { card_id: Uuid, target_list_id: Uuid, position: u32 },

    // ── UI State ─────────────────────────────────────────────────────
    OpenCardDetail(Uuid),
    ToggleContextPage(ContextPage),
    OpenNewListInput,
    OpenNewCardInput(Uuid),      // Uuid = target list
    NewListInputChanged(String),
    NewCardInputChanged(String),
    ConfirmNewList,
    ConfirmNewCard(Uuid),        // Uuid = target list
    DismissNewInput,

    // ── Persistence ──────────────────────────────────────────────────
    SaveActiveBoard,
    BoardSaved,
    LoadError(String),
}
```

## AppModel Design

### `Config`

`Config` is persisted by the COSMIC config system (not the TOML store) and survives app restarts independently of board data. It derives `CosmicConfigEntry`, which uses serde under the hood, so all fields must implement `Serialize + Deserialize`.

```src/config.rs#L1-20
use cosmic::cosmic_config::{self, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry};
use uuid::Uuid;

#[derive(Debug, Default, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 1]
pub struct Config {
    /// The ID of the board that was active when the app was last closed.
    /// On startup the nav bar re-activates this board automatically.
    pub last_board_id: Option<Uuid>,
}
```

`Uuid` is already serialisable via `uuid = { version = "1", features = ["v4", "serde"] }`. `Option<Uuid>` defaults to `None`, satisfying the `Default` bound required by `CosmicConfigEntry`.

### `AppModel`

The `nav_bar::Model` is the single source of truth for which boards exist and which one is active. Each nav item carries a `Uuid` as its data, which maps to the board stored in `AppModel::boards`.

```src/app.rs#L1-60
pub struct AppModel {
    core: cosmic::Core,

    /// COSMIC nav bar — one item per board.
    /// The active nav item ID determines which board is displayed.
    nav: nav_bar::Model,

    /// All boards loaded into memory (keyed by board Uuid).
    /// A board is loaded on first nav selection and cached here.
    boards: HashMap<Uuid, Board>,

    /// Inline input state: which list (if any) has the "add card" input open.
    new_card_input: Option<(Uuid, String)>,  // (list_id, current text)

    /// Inline input state for adding a new list column.
    /// Stores the widget `Id` of the active text input so libcosmic can
    /// focus it and route keyboard events to the correct field.
    new_list_input: Option<cosmic::widget::Id>,

    /// The context drawer page currently shown.
    context_page: ContextPage,

    /// About page widget.
    about: About,

    /// Key bindings.
    key_binds: HashMap<menu::KeyBind, MenuAction>,

    /// Persisted config (remembers last active board, etc.).
    config: Config,
}

/// Returns the Uuid of the currently active board, if any.
impl AppModel {
    fn active_board_id(&self) -> Option<Uuid> {
        self.nav.active_data::<Uuid>().copied()
    }

    fn active_board(&self) -> Option<&Board> {
        self.active_board_id().and_then(|id| self.boards.get(&id))
    }

    fn active_board_mut(&mut self) -> Option<&mut Board> {
        self.active_board_id().and_then(|id| self.boards.get_mut(&id))
    }
}
```

### Nav Bar Lifecycle

```src/app.rs#L1-40
// On init: load the board index from disk, populate nav items
fn init(core: Core, _flags: ()) -> (Self, Task<Action<Message>>) {
    let mut nav = nav_bar::Model::default();

    // Load board summaries (id + title) from the index file.
    // Full board data is loaded lazily on first selection.
    let summaries = Store::new().load_board_index().unwrap_or_default();
    for summary in &summaries {
        nav.insert()
            .text(&summary.title)
            .icon(icon::from_name("view-grid-symbolic"))
            .data(summary.id);
    }

    // Re-activate the last open board from config
    if let Some(last_id) = config.last_board_id {
        if let Some(item) = nav.iter().find(|i| nav.data::<Uuid>(*i) == Some(&last_id)) {
            nav.activate(item);
        }
    }
    // ...
}

// When a nav item is selected: load board data if not cached
fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<Action<Message>> {
    self.nav.activate(id);
    self.update_title();

    if let Some(board_id) = self.active_board_id() {
        if !self.boards.contains_key(&board_id) {
            // Load from disk asynchronously
            return Task::perform(
                async move { Store::new().load_board(board_id) },
                |result| match result {
                    Ok(board) => cosmic::Action::App(Message::BoardLoaded(board)),
                    Err(e)    => cosmic::Action::App(Message::LoadError(e.to_string())),
                },
            );
        }
    }
    Task::none()
}
```

### Context Pages

```src/app.rs#L1-15
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
    /// Card detail editor — carries the ID of the card being viewed
    CardDetail(Uuid),
    /// New board creation form
    NewBoard,
    /// Board settings (rename, color, delete)
    BoardSettings,
}
```

### State Flow Diagram

```
 Nav bar                   AppModel                  Context Drawer
 ────────                  ────────                  ──────────────
 [Board A] ─on_nav_select─► active_board_id = A  
 [Board B]                  load_board(A) if miss
 [Board C]                  boards[A] cached
 [+ New]  ─────────────────► ContextPage::NewBoard ──► "New Board" form
                             CreateBoard msg
                             nav.insert() + boards.insert()

                            [Card clicked]
                             ContextPage::CardDetail(card_id) ──► Card editor
                             (board view stays visible beneath)
```

---

## Widget Design

### Card Widget (`widgets/card_widget.rs`)

Each card is a `mouse_area`-wrapped `container` that renders:

- **Labels row**: Colored pill badges (max 3 visible, then `+N more`)
- **Title**: `widget::text::title4`
- **Metadata row**: Due date badge, checklist progress (`2/4 ☐`)
- **Click**: opens `ContextPage::CardDetail` in the context drawer

```src/widgets/card_widget.rs#L1-50
pub fn view_card<'a>(card: &'a Card, is_dragging: bool) -> Element<'a, Message> {
    let labels_row = widget::row::with_capacity(card.labels.len())
        .extend(card.labels.iter().take(3).map(label_badge))
        .spacing(space_xxs);

    let checklist_progress = {
        let done = card.checklist.iter().filter(|i| i.completed).count();
        let total = card.checklist.len();
        if total > 0 {
            Some(widget::text::caption(format!("☐ {done}/{total}")))
        } else {
            None
        }
    };

    let body = widget::column::with_capacity(4)
        .push(labels_row)
        .push(widget::text::title4(&card.title))
        .push_maybe(card.due_date.map(|d| {
            widget::text::caption(format!("📅 {}", d.format("%b %d")))
        }))
        .push_maybe(checklist_progress)
        .spacing(space_xxs)
        .padding([space_s, space_s]);

    let card_container = widget::container(body)
        .style(cosmic::theme::Container::Card)
        .width(Length::Fill);

    // Wrap in mouse_area to detect hover for action overlay
    widget::mouse_area(card_container)
        .on_press(Message::OpenCardDetail(card.id))
        .on_right_press(Message::OpenCardContextMenu(card.id))
        .into()
}
```

### List Column Widget (`widgets/list_column.rs`)

Each column has three vertical sections: a **pinned header**, a **vertically scrollable card list** that expands to fill all remaining height, and a **pinned footer** with the add-card button. The key is giving the scrollable `height(Length::Fill)` so it claims the space between the two pinned sections, and giving the outer container `height(Length::Fill)` so the column itself stretches to the window height.

```src/widgets/list_column.rs#L1-75
pub fn view_list<'a>(
    list: &'a List,
    drag_target: Option<Uuid>,
) -> Element<'a, Message> {
    let is_drop_target = drag_target == Some(list.id);

    // ── Pinned header: list title (inline-editable) + add-card shortcut ──
    let header = widget::row::with_capacity(2)
        .push(
            widget::editable_input("List name…", &list.title, false, |new_title| {
                Message::RenameList { list_id: list.id, new_title }
            })
            .width(Length::Fill),
        )
        .push(
            widget::button::icon(icon::from_name("list-add-symbolic"))
                .on_press(Message::OpenNewCardInput(list.id))
        )
        .align_y(Alignment::Center)
        .spacing(space_xs)
        .padding([space_xs, space_s]);

    // ── Scrollable body: card list expands to fill remaining column height ──
    let cards_column = widget::column::with_capacity(list.cards.len())
        .extend(list.cards.iter().map(|card| view_card(card, false)))
        .spacing(space_xs)
        .padding([0, space_xs]);  // inner side padding so cards don't touch column edges

    let scrollable_body = widget::scrollable::vertical(cards_column)
        .height(Length::Fill);  // ← critical: consumes all space between header and footer

    // ── Pinned footer: "add a card" inline input or button ──
    let footer = widget::button::text("+ Add a card")
        .width(Length::Fill)
        .on_press(Message::OpenNewCardInput(list.id));

    // ── Outer column: header / scroll-area / footer stacked vertically ──
    let body = widget::container(
        widget::column::with_capacity(3)
            .push(header)
            .push(widget::divider::horizontal::default())
            .push(scrollable_body)   // height::Fill — grows to fill
            .push(widget::divider::horizontal::default())
            .push(footer)
            .padding(space_xs),
    )
    .width(280)
    .height(Length::Fill)   // ← column itself stretches to full window height
    .style(if is_drop_target {
        cosmic::theme::Container::Primary
    } else {
        cosmic::theme::Container::Secondary
    });

    // Make the entire column a drop destination for dragged cards
    widget::dnd_destination::for_data::<CardDragData>(
        body,
        move |data, _action| {
            if let Some(d) = data {
                Message::DragCardDropped {
                    card_id: d.card_id,
                    target_list_id: list.id,
                    position: list.cards.len() as u32,
                }
            } else {
                Message::DragCardLeftList
            }
        },
    )
    .on_enter(move |_, _, _| Message::DragCardEnteredList(list.id))
    .on_leave(|| Message::DragCardLeftList)
    .into()
}
```

### Board View (`pages/board.rs`)

The outer board view scrolls **horizontally** across columns. The `height(Length::Fill)` chain flows from the outer `scrollable` all the way down into each column's inner `scrollable`, so every column independently fills and scrolls vertically:

```src/pages/board.rs#L1-45
pub fn view_board<'a>(board: &'a Board, drag_target: Option<Uuid>) -> Element<'a, Message> {
    let columns = board.lists.iter().map(|list| view_list(list, drag_target));

    // The row holds all columns side by side.
    // height(Length::Fill) is essential: it tells each child column to
    // stretch to the full available height, which in turn allows each
    // column's inner scrollable to do the same.
    let board_row = widget::row::with_capacity(board.lists.len())
        .extend(columns)
        .spacing(space_m)
        .padding([space_m, space_l])
        .height(Length::Fill);  // ← propagates Fill down into every column

    // Horizontal scroll wraps the whole row so that columns beyond the
    // window width are reachable. The vertical axis is NOT scrolled here;
    // each column handles its own vertical scroll independently.
    widget::scrollable::horizontal(board_row)
        .height(Length::Fill)  // ← outer container also Fill so the board takes the whole pane
        .into()
}
```

**Height propagation chain:**
```
  view() [Length::Fill]
    └─ scrollable::horizontal [height: Fill]
         └─ row [height: Fill]
              └─ container (column shell) [width: 280, height: Fill]
                   └─ column (header / scroll / footer)
                        └─ scrollable::vertical [height: Fill]  ← clips & scrolls cards
                             └─ column (cards)  [height: Shrink]  ← as tall as its cards
```

---

## Feature Roadmap

### Phase 1 — Core (MVP)
- [x] Project scaffolding (already done)
- [x] Data models with serde
- [x] TOML persistence layer (`db/store.rs`)
- [x] Populate nav bar from board index on startup
- [x] Create / rename / delete boards (via context drawer)
- [x] Board view: horizontal scrollable columns as main view
- [x] Create / rename / delete lists (inline input)
- [x] Create / rename / delete cards (inline input)
- [x] Card detail in context drawer (title + description)
- [x] Empty state when no boards exist
- [x] Remember last active board across restarts (config)

### Phase 2 — Productivity Features
- [x] Labels: color palette + assign to cards
- [x] Checklist items with progress tracking
- [x] Due date picker
- [x] Drag-and-drop card reordering within a list
- [x] Drag-and-drop card moving across lists
- [ ] Markdown rendering in card descriptions (libcosmic `markdown` feature)
- [ ] Right-click context menu on cards

### Phase 3 — Polish & Power Features
- [ ] Board color/background customization
- [ ] List column reordering via drag
- [ ] Card archive / undo-delete
- [ ] Filter cards by label or due date
- [ ] Full-text search across all boards
- [ ] Keyboard navigation (Tab, Enter, Escape)
- [ ] Configurable key bindings
- [ ] Export board as JSON / Markdown

---

## Dependencies

The following crates should be added to `Cargo.toml`:

```boards/Cargo.toml#L1-30
[dependencies]
# Serialization
serde = { version = "1", features = ["derive"] }
toml = "1.1.2"

# UUIDs for entity IDs
uuid = { version = "1", features = ["v4", "serde"] }

# Date/time for due dates and timestamps
jiff = { version = "0.2.24", features = ["serde"] }

# XDG base directories (data/config paths)
directories = "6.0.0"

# Atomic temp-file writes
tempfile = "3"
```

> `serde_json` is **not** needed. `iced::Color`'s serde support comes for free from libcosmic's own `iced_core` dependency, which is already compiled with `features = ["serde"]`.

The libcosmic features already in `Cargo.toml` that this app will use:
- **`markdown`** — card description rendering
- **`about`** — About dialog
- **`single-instance`** — prevent duplicate windows
- **`xdg-portal`** — (future) file attachments

---

## App ID

The application should use the RDNN:

```
dev.edfloreshz.Boards
```

This aligns with the author's RDNN and ensures the XDG data dir is `~/.local/share/dev.edfloreshz.Boards/`.
