use cosmic::{Element, Task};

/// A page that can optionally show a modal dialog over itself.
/// Implementors own their own dialog state as a real struct field —
/// this trait never stores anything; it only describes how to read/update it.
pub trait DialogHost {
    type Message;

    /// Render the current dialog, if any. Called every frame; must be cheap
    /// and must read from `&self`, never construct fresh default state.
    fn dialog(&self) -> Option<Element<'_, Self::Message>>;

    /// Route a message into this page's `update`, which is where dialog
    /// state actually gets mutated (e.g. text input changes, confirm/cancel).
    fn update(&mut self, message: Self::Message) -> Task<Self::Message>;
}

/// A single-slot dialog holder a page can embed as a field.
/// Generic over the page's own dialog-kind enum, so state stays
/// page-specific while the open/dismiss mechanics are shared.
#[derive(Debug, Clone)]
pub struct DialogSlot<D> {
    current: Option<D>,
}

impl<D> DialogSlot<D> {
    #[allow(unused)]
    pub fn new(dialog: D) -> Self {
        Self {
            current: Some(dialog),
        }
    }

    #[allow(unused)]
    pub fn open(&mut self, dialog: D) {
        self.current = Some(dialog);
    }

    #[allow(unused)]
    pub fn dismiss(&mut self) {
        self.current = None;
    }

    #[allow(unused)]
    pub fn get(&self) -> Option<&D> {
        self.current.as_ref()
    }

    #[allow(unused)]
    pub fn get_mut(&mut self) -> Option<&mut D> {
        self.current.as_mut()
    }

    #[allow(unused)]
    pub fn is_open(&self) -> bool {
        self.current.is_some()
    }
}
