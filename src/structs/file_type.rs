#[derive(Clone, PartialEq)]
pub enum EncodingOptions {
    Avif(AvifSettings),
    WebP(WebpSettings),
    Jpeg(JpegSettings),
}

impl std::fmt::Display for EncodingOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncodingOptions::Avif(_) => write!(f, "avif"),
            EncodingOptions::WebP(_) => write!(f, "webp"),
            EncodingOptions::Jpeg(_) => write!(f, "jpg"),
        }
    }
}

// Avif settings
#[derive(Clone, PartialEq)]
pub struct AvifSettings {
    pub quality: u8,
    pub speed: u8,
    pub lossless: bool,
}

impl Default for AvifSettings {
    fn default() -> Self {
        Self {
            quality: 75,
            speed: 3,
            lossless: false,
        }
    }
}


// Webp settings
#[derive(Clone, PartialEq)]
pub struct WebpSettings {
    pub quality: u8,
    pub lossless: bool,
}

impl Default for WebpSettings {
    fn default() -> Self {
        Self {
            quality: 90,
            lossless: false,
        }
    }
}

// Jpeg settings
#[derive(Clone, PartialEq)]
pub struct JpegSettings {
    pub quality: u8,
}

impl Default for JpegSettings {
    fn default() -> Self {
        Self { quality: 90 }
    }
}
