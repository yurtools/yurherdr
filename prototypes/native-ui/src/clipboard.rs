//! Validated encoded images and stale-reply rejection. Call decode on a worker;
//! no file/URL resolver or provider delivery exists here.
use image::{ImageFormat, ImageReader, Limits};
use std::{io::Cursor, sync::Arc};

pub const MAX_ENCODED: usize = 4 * 1024 * 1024;
pub const MAX_PIXELS: u64 = 4 * 1024 * 1024;
pub const MAX_DIMENSION: u32 = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasteError {
    Empty,
    Unsupported,
    Busy,
    Malformed,
    Oversized,
    Closed,
    Exhausted,
}
impl PasteError {
    pub fn message(self) -> &'static str {
        match self {
            Self::Empty => "Copy an image, then paste again.",
            Self::Unsupported => "Copy a PNG or JPEG image, then paste again.",
            Self::Busy => "Wait for the current paste to finish.",
            Self::Malformed => "That image could not be read. Copy it again.",
            Self::Oversized => "Use an image up to four megapixels and four MiB.",
            Self::Closed => "Reopen the composer to paste an image.",
            Self::Exhausted => "Restart the evaluation to paste again.",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DraftImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Arc<[u8]>,
}

pub fn decode(mime: &str, bytes: &[u8]) -> Result<DraftImage, PasteError> {
    if bytes.is_empty() {
        return Err(PasteError::Empty);
    }
    if bytes.len() > MAX_ENCODED {
        return Err(PasteError::Oversized);
    }
    let format = match mime {
        "image/png" => ImageFormat::Png,
        "image/jpeg" => ImageFormat::Jpeg,
        _ => return Err(PasteError::Unsupported),
    };
    if image::guess_format(bytes).map_err(|_| PasteError::Malformed)? != format {
        return Err(PasteError::Malformed);
    }
    let (width, height) = ImageReader::with_format(Cursor::new(bytes), format)
        .into_dimensions()
        .map_err(|_| PasteError::Malformed)?;
    if width == 0
        || height == 0
        || width > MAX_DIMENSION
        || height > MAX_DIMENSION
        || u64::from(width) * u64::from(height) > MAX_PIXELS
    {
        return Err(PasteError::Oversized);
    }
    let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_DIMENSION);
    limits.max_image_height = Some(MAX_DIMENSION);
    limits.max_alloc = Some(MAX_PIXELS * 8);
    reader.limits(limits);
    let image = reader
        .decode()
        .map_err(|_| PasteError::Malformed)?
        .into_rgba8();
    Ok(DraftImage {
        width,
        height,
        rgba: image.into_raw().into(),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PasteTicket(u64);

pub struct ClipboardDraft {
    generation: u64,
    pending: Option<PasteTicket>,
    open: bool,
    pub image: Option<DraftImage>,
    pub error: Option<PasteError>,
}
impl Default for ClipboardDraft {
    fn default() -> Self {
        Self {
            generation: 0,
            pending: None,
            open: true,
            image: None,
            error: None,
        }
    }
}
impl ClipboardDraft {
    pub fn begin(&mut self) -> Result<PasteTicket, PasteError> {
        if !self.open {
            return Err(PasteError::Closed);
        }
        if self.pending.is_some() {
            return Err(PasteError::Busy);
        }
        self.generation = self
            .generation
            .checked_add(1)
            .ok_or(PasteError::Exhausted)?;
        let ticket = PasteTicket(self.generation);
        self.pending = Some(ticket);
        self.error = None;
        Ok(ticket)
    }
    pub fn finish(&mut self, ticket: PasteTicket, result: Result<DraftImage, PasteError>) -> bool {
        if !self.open || self.pending != Some(ticket) {
            return false;
        }
        self.pending = None;
        match result {
            Ok(image) => {
                self.image = Some(image);
                self.error = None;
            }
            Err(e) => self.error = Some(e),
        }
        true
    }
    pub fn close_view(&mut self) {
        self.open = false;
        self.pending = None;
    }
    pub fn reopen_view(&mut self) {
        self.open = true;
    }
    pub fn remove_image(&mut self) {
        self.pending = None;
        self.image = None;
        self.error = None;
    }
}
