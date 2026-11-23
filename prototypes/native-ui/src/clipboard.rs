//! Validated encoded images and stale-reply rejection. Call decode on a worker;
//! no file/URL resolver or provider delivery exists here.
use image::{ImageDecoder, ImageFormat, ImageReader, Limits};
use std::{
    io::Cursor,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

pub const MAX_ENCODED: usize = 4 * 1024 * 1024;
pub const MAX_PIXELS: u64 = 4 * 1024 * 1024;
pub const MAX_DIMENSION: u32 = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasteError {
    Empty,
    Unsupported,
    NativeUnavailable,
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
            Self::NativeUnavailable => {
                "Image paste needs a supported Wayland desktop. Use a fixture image for this evaluation."
            }
            Self::Busy => "Wait a moment, then paste again.",
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
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_DIMENSION);
    limits.max_image_height = Some(MAX_DIMENSION);
    limits.max_alloc = Some(MAX_PIXELS * 8);
    let mut header = ImageReader::with_format(Cursor::new(bytes), format);
    header.limits(limits.clone());
    let (width, height) = header.into_dimensions().map_err(image_error)?;
    if width == 0
        || height == 0
        || width > MAX_DIMENSION
        || height > MAX_DIMENSION
        || u64::from(width) * u64::from(height) > MAX_PIXELS
    {
        return Err(PasteError::Oversized);
    }
    let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
    reader.limits(limits.clone());
    let mut decoder = reader.into_decoder().map_err(image_error)?;
    let orientation = decoder.orientation().map_err(image_error)?;
    limits.reserve(decoder.total_bytes()).map_err(image_error)?;
    decoder.set_limits(limits).map_err(image_error)?;
    let mut image = image::DynamicImage::from_decoder(decoder).map_err(image_error)?;
    image.apply_orientation(orientation);
    let image = image.into_rgba8();
    let (width, height) = image.dimensions();
    Ok(DraftImage {
        width,
        height,
        rgba: image.into_raw().into(),
    })
}
fn image_error(error: image::ImageError) -> PasteError {
    match error {
        image::ImageError::Limits(_) => PasteError::Oversized,
        _ => PasteError::Malformed,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PasteTicket(u64);

pub struct ClipboardDraft {
    worker: Arc<AtomicBool>,
    generation: u64,
    pending: Option<PasteTicket>,
    open: bool,
    pub image: Option<DraftImage>,
    pub error: Option<PasteError>,
}
impl Default for ClipboardDraft {
    fn default() -> Self {
        Self {
            worker: Arc::new(AtomicBool::new(false)),
            generation: 0,
            pending: None,
            open: true,
            image: None,
            error: None,
        }
    }
}
impl ClipboardDraft {
    /// One physical read/decode remains in flight even if its logical reply is
    /// cancelled by Remove/Close. A late helper cannot cause a worker flood.
    pub fn begin_work(&mut self) -> Result<(PasteTicket, PasteWorker), PasteError> {
        self.worker
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| PasteError::Busy)?;
        let worker = PasteWorker(self.worker.clone());
        let ticket = self.begin()?;
        Ok((ticket, worker))
    }
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
pub struct PasteWorker(Arc<AtomicBool>);
impl Drop for PasteWorker {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}
