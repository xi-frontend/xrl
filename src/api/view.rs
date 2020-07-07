use super::ViewPort;
use crate::api::{CharRef, LineCache, LineRef};
use crate::protocol::{ConfigChanges, FindStatus, Status, StyleDef, UpdateNotification, ViewId};

pub struct View {
    pub id: ViewId,
    pub language: Option<String>,
    pub cache: LineCache,
    pub viewport: ViewPort,
    pub config: Option<ConfigChanges>,
    pub plugins: Vec<String>,
    pub find_status: Option<FindStatus>,
    pub replace_status: Option<Status>,
}

impl View {
    pub fn new(id: ViewId) -> View {
        View {
            id,
            language: None,
            cache: LineCache::new(),
            viewport: ViewPort::default(),
            config: None,
            plugins: vec![],
            find_status: None,
            replace_status: None,
        }
    }

    pub fn update(&mut self, update: UpdateNotification) {
        self.cache.update(update.update);
    }

    pub fn render_lines(&self) -> impl Iterator<Item = LineRef<'_>> {
        let horizontal_offset = self.viewport.horizontal_offset as usize;
        self.cache
            .lines
            .iter()
            .skip(self.viewport.vertical_offset as usize)
            .take(self.viewport.height as usize)
            .filter_map(|item| item.as_ref())
            .map(move |line| LineRef {
                text: &line.text[horizontal_offset..],
                cursor: &line.cursor,
                styles: render_line_styles(horizontal_offset, &line.styles),
                line_num: line.line_num,
            })
    }

    pub fn render_chars(&self) -> impl Iterator<Item = impl Iterator<Item = CharRef> + '_> {
        self.render_lines().enumerate().map(move |(y_pos, line)| {
            line.text
                .chars()
                .enumerate()
                .map(move |(x_pos, character)| CharRef {
                    character,
                    style_id: get_index_style(x_pos, &line.styles),
                    position: (x_pos as u32, y_pos as u32),
                })
        })
    }
}

fn get_index_style(offset: usize, styles: &[StyleDef]) -> Option<u64> {
    let mut current_step: usize = 0;

    for style in styles {
        if offset > current_step {
            return None;
        } else if offset > current_step + style.offset as usize + style.length as usize {
            return Some(style.style_id);
        }
        current_step += style.offset as usize + style.length as usize;
    }
    None
}

fn render_line_styles(offset: usize, styles: &[StyleDef]) -> Vec<StyleDef> {
    let mut new_styles = vec![];
    let mut current_index: i64 = 0;
    for style in styles {
        let offset = offset as i64;
        let length = style.length as i64;
        let style_offset = style.offset as i64;
        let style_id = style.style_id;
        println!(
            "current_step: {}, offset: {}, style: {:?}",
            current_index, offset, style
        );
        if current_index + style_offset < offset && current_index + style_offset + length < offset {
            println!("Removing style");
            continue;
        } else if current_index + style_offset >= offset && offset > current_index {
            println!("Adding style with smaller offset");
            let offset = current_index + style_offset - offset;
            new_styles.push(StyleDef {
                offset,
                style_id,
                length: length as u64,
            });
        } else if current_index + style_offset + length > offset
            && offset > current_index + style_offset
        {
            println!("Adding style with smaller length");
            let length =
                current_index + style_offset + length - current_index + style_offset - offset;
            new_styles.push(StyleDef {
                offset: 0,
                style_id,
                length: length as u64,
            });
        } else {
            println!("adding default style");
            new_styles.push(style.clone());
        }
        current_index += style_offset + length;
    }
    new_styles
}
