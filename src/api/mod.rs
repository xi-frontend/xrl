//! This module holds common structures that can be mix and matched to help when creating an xi
//! based editor. The `Editor` is the main struct and can handle all xi related actions.
//! Their are 2 methods that can be used for rendering. `render_lines` that can be used to render
//! line by line and `render_chars` that can be used to render individual characters at a time.

mod line_cache;
pub use self::line_cache::LineCache;

mod style_cache;
pub use self::style_cache::StyleCache;

mod view_list;
pub use self::view_list::ViewList;

mod view;
pub use self::view::View;

mod editor;
pub use self::editor::Editor;

mod viewport;
pub use self::viewport::ViewPort;

mod render;
pub use self::render::{CharRef, LineRef};
