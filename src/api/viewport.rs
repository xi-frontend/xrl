use crate::protocol::ScrollTo;

#[derive(Default)]
pub struct ViewPort {
    pub width: u64,
    pub height: u64,
    pub horizontal_offset: u64,
    pub vertical_offset: u64,
}

impl ViewPort {
    pub fn resize(&mut self, width: u64, height: u64) {
        self.width = width;
        self.height = height;
    }

    pub fn scroll_to(&mut self, scroll: ScrollTo) {
        let line = scroll.line;
        let column = scroll.column;
        let vertical_offset = self.vertical_offset;
        let horizontal_offset = self.horizontal_offset;
        let height = self.height;
        let width = self.width;
        if line >= vertical_offset && line - vertical_offset >= height {
            self.vertical_offset = line - height + 1;
        } else if line < vertical_offset {
            self.vertical_offset = line;
        }
        if column >= horizontal_offset && column - horizontal_offset >= width {
            self.horizontal_offset = column - width + 1;
        } else if column < horizontal_offset && horizontal_offset > 0 {
            self.horizontal_offset = column;
        }
    }
}
