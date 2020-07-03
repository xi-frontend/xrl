use serde::{self, Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone)]
pub struct StyleDef {
    pub offset: i64,
    pub length: u64,
    pub style_id: u64,
}

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Line {
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub cursor: Vec<u64>,
    #[serde(deserialize_with = "deserialize_styles")]
    pub styles: Vec<StyleDef>,
    #[serde(rename = "ln")]
    pub line_num: Option<u64>,
}

// FIXME: it's not super efficient to create an intermediate vector, this might
// become a problem when we have big updates with a lot of styles.
pub fn deserialize_styles<'de, D>(deserializer: D) -> Result<Vec<StyleDef>, D::Error>
where
    D: Deserializer<'de>,
{
    let v = Vec::<i64>::deserialize(deserializer)?;
    if v.len() % 3 != 0 {
        return Err(serde::de::Error::custom(format!(
            "styles length is not a multiple of 3: {}",
            v.len()
        )));
    }

    let nb_styles = v.len() / 3;
    let mut styles = Vec::with_capacity(nb_styles);
    #[cfg_attr(feature = "clippy", allow(needless_range_loop))]
    for i in 0..nb_styles {
        styles.push(StyleDef {
            offset: v[i * 3],
            length: v[i * 3 + 1] as u64,   // FIXME: this can panic
            style_id: v[i * 3 + 2] as u64, // FIXME: this can panic
        });
    }
    Ok(styles)
}
