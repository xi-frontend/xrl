use crate::protocol::StyleDef;

#[derive(Debug, PartialEq)]
pub struct LineRef<'a> {
    pub text: &'a str,
    pub styles: Vec<StyleDef>,
    pub cursor: &'a [u64],
    pub line_num: Option<u64>,
}

#[derive(Debug)]
pub struct CharRef {
    pub position: (u32, u32),
    pub character: char,
    pub style_id: Option<u64>,
}
