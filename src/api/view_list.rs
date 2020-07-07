use indexmap::IndexMap;
use log::error;

use crate::protocol::ViewId;

/// View List to store xi views. Can retrieve views as well as move between them using the
/// `next` and `prev` methods.
pub struct ViewList<T> {
    index: Option<ViewId>,
    views: IndexMap<ViewId, T>,
}

impl<T> ViewList<T> {

    /// Get a reference to a view.
    pub fn get(&self, id: &ViewId) -> Option<&T> {
        self.views.get(id)
    }

    /// Get a mutable reference to the current view if there is any.
    pub fn get_current_mut(&mut self) -> Option<&mut T> {
        if let Some(index) = self.index {
            self.views.get_mut(&index)
        } else {
            None
        }
    }

    /// Get a mutable reference to the vew `id`.
    pub fn get_mut(&mut self, id: &ViewId) -> Option<&mut T> {
        self.views.get_mut(id)
    }

    /// Get a reference to the current view if there is any.
    pub fn get_current(&self) -> Option<&T> {
        self.index.and_then(|item| self.views.get(&item))
    }

    /// Get the current ViewId.
    pub fn get_current_id(&self) -> Option<ViewId> {
        self.index
    }

    /// Get the index of the current view in the list.
    pub fn get_current_index(&self) -> Option<usize> {
        self.index.and_then(|item| self.views.get_index_of(&item))
    }

    /// Returns an iterator of Views in the list.
    pub fn get_all(&self) -> impl Iterator<Item = &T> {
        self.views.values()
    }

    /// Return a list of ViewsIds for each in the list.
    pub fn keys(&self) -> impl Iterator<Item = &ViewId> {
        self.views.keys()
    }

    /// Get a reference to the view with id `index`.
    pub fn get_index(&self, index: usize) -> Option<&T> {
        self.views.get_index(index).map(|item| item.1)
    }

    /// Get the number of views in the list.
    pub fn len(&self) -> usize {
        self.views.len()
    }

    /// Whether the list contains any views.
    pub fn is_empty(&self) -> bool {
        self.views.len() == 0
    }

    /// Add a new view to the list.
    pub fn add(&mut self, id: ViewId, view: T) {
        self.index = Some(id);
        self.views.insert(id, view);
    }

    /// Move to the previous view in the list.
    pub fn prev(&mut self) {
        if let Some(current_view) = self.index {
            if let Some((dex, _, _)) = self.views.get_full(&current_view) {
                if dex == 0 {
                    if let Some((view, _)) = self.views.get_index(self.views.len() - 1) {
                        self.index = Some(*view);
                    }
                } else if let Some((view, _)) = self.views.get_index(dex - 1) {
                    self.index = Some(*view);
                }
            } else {
                error!(
                    "Current view was set to a non existant view: {}",
                    current_view
                );
            }
        } else {
            error!("Current View was not set");
        }
    }

    /// Move to the next view in the list.
    pub fn next(&mut self) {
        if let Some(current_view) = self.index {
            if let Some((dex, _, _)) = self.views.get_full(&current_view) {
                if dex + 1 == self.views.len() {
                    if let Some((view, _)) = self.views.get_index(0) {
                        self.index = Some(*view);
                    }
                } else if let Some((view, _)) = self.views.get_index(dex + 1) {
                    self.index = Some(*view);
                }
            } else {
                error!(
                    "Current view was set to a non existant view: {}",
                    current_view
                );
            }
        } else {
            error!("Current View was not set");
        }
    }
}

impl<T> Default for ViewList<T> {
    fn default() -> ViewList<T> {
        ViewList {
            index: None,
            views: IndexMap::new(),
        }
    }
}
