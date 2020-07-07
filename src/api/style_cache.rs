use crate::protocol::Style;

use std::collections::HashMap;

/// Style cache used to store syntax highlighting styles.
/// Just a simple wrapper around an internal HashMap<u64, Style>.
#[derive(Default)]
pub struct StyleCache(HashMap<u64, Style>);

impl StyleCache {

    /// Return an iterator of all styles in the style cache.
    pub fn styles(&self) -> impl Iterator<Item = (&u64, &Style)> {
        self.0.iter()
    }

    /// Insert a new style into the StyleCache.
    pub fn insert(&mut self, id: u64, style: Style) {
        self.0.insert(id, style);
    }

    /// Fetch an id from Style.
    pub fn get(&self, id: u64) -> Option<&Style> {
        self.0.get(&id)
    }

    /// Remove a style by it's id.
    pub fn remove(&mut self, id: u64) {
        self.0.remove(&id);
    }
}
