#[derive(PartialEq, Clone, Copy, Debug)]
pub enum FileType {
    WebP,
    Avif,
    Jpeg,
}

impl std::fmt::Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WebP => write!(f, "WebP"),
            Self::Avif => write!(f, "AVIF"),
            Self::Jpeg => write!(f, "JPEG"),
        }
    }
}