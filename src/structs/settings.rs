use super::file_type::FileType;

#[derive(Clone)]
pub struct Settings {
    pub file_type: FileType,
    pub quality: u32,
    pub resize_options: ResizeOptions,
    pub name_extension: Option<String>,
    pub keep_exif: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            file_type: FileType::WebP,
            quality: 95,
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
