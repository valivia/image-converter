use super::file_type::{AvifSettings, EncodingOptions};

#[derive(Clone)]
pub struct Settings {
    pub encoding_options: EncodingOptions,
    pub resize_options: ResizeOptions,
    pub name_extension: Option<String>,
    pub keep_exif: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            encoding_options: EncodingOptions::Avif(AvifSettings::default()),
            resize_options: ResizeOptions::None,
            name_extension: None,
            keep_exif: false,
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum ResizeOptions {
    None,
    Largest(u32),
    Exact(u32, u32),
    Smallest(u32),
}
