use image::{ExtendedColorType, ImageEncoder, codecs::png::PngEncoder};
use yurherdr_native_ui_probe::{
    clipboard::{self, ClipboardDraft, PasteError},
    panes::{MoveError, PaneId, Panes, WindowId},
};

#[test]
fn every_pane_rolls_back_failed_destination_and_rejects_late_readiness() {
    for pane in [PaneId::Composer, PaneId::Document, PaneId::Terminal] {
        let mut panes = Panes::default();
        let source = panes.reopen(pane, WindowId(1)).unwrap();
        let failed = panes.reserve(source, WindowId(2)).unwrap();
        assert_eq!(panes.view(pane), Some(source));
        assert_eq!(panes.reserve(source, WindowId(3)), Err(MoveError::Busy));
        panes.rollback(failed).unwrap();
        let retry = panes.reserve(source, WindowId(2)).unwrap();
        assert_eq!(panes.ready(failed), Err(MoveError::Stale));
        let moved = panes.ready(retry).unwrap();
        assert!(!panes.accepts(source));
        assert!(panes.accepts(moved));
        assert_eq!(panes.ready(retry), Err(MoveError::Stale));
        assert_eq!(panes.close(source), Err(MoveError::Stale));
    }
}
#[test]
fn repeated_close_reopen_and_detach_never_resurrects_a_stale_view() {
    let mut panes = Panes::default();
    let mut old = vec![];
    for i in 1..100 {
        let source = panes.reopen(PaneId::Terminal, WindowId(1)).unwrap();
        let ticket = panes.reserve(source, WindowId(i + 1)).unwrap();
        panes.close(source).unwrap();
        assert_eq!(panes.ready(ticket), Err(MoveError::Stale));
        let reopen = panes.reopen(PaneId::Terminal, WindowId(1)).unwrap();
        assert!(old.iter().all(|view| !panes.accepts(*view)));
        let ticket = panes.reserve(reopen, WindowId(i + 1)).unwrap();
        let moved = panes.ready(ticket).unwrap();
        panes.close(moved).unwrap();
        old.extend([source, reopen, moved]);
    }
}
fn png() -> Vec<u8> {
    let mut bytes = vec![];
    PngEncoder::new(&mut bytes)
        .write_image(&[10, 20, 30, 255], 1, 1, ExtendedColorType::Rgba8)
        .unwrap();
    bytes
}
#[test]
fn images_are_decoded_and_late_replies_cannot_replace_a_new_draft() {
    let bytes = png();
    let image = clipboard::decode("image/png", &bytes).unwrap();
    assert_eq!(image.rgba.as_ref(), [10, 20, 30, 255]);
    let mut draft = ClipboardDraft::default();
    let old = draft.begin().unwrap();
    assert_eq!(draft.begin(), Err(PasteError::Busy));
    draft.close_view();
    draft.reopen_view();
    let current = draft.begin().unwrap();
    assert!(!draft.finish(old, Ok(image.clone())));
    assert!(draft.image.is_none());
    assert!(draft.finish(current, Ok(image.clone())));
    assert!(draft.image.is_some());
    let late = draft.begin().unwrap();
    draft.remove_image();
    assert!(!draft.finish(late, Ok(image)));
    assert!(draft.image.is_none());
}
#[test]
fn malformed_mismatched_and_bounded_images_leave_existing_preview_intact() {
    assert!(matches!(
        clipboard::decode("image/png", &[]),
        Err(PasteError::Empty)
    ));
    assert!(matches!(
        clipboard::decode("image/png", &[0; 32]),
        Err(PasteError::Malformed)
    ));
    assert!(matches!(
        clipboard::decode("image/svg+xml", &png()),
        Err(PasteError::Unsupported)
    ));
    assert!(matches!(
        clipboard::decode("image/jpeg", &png()),
        Err(PasteError::Malformed)
    ));
    assert!(matches!(
        clipboard::decode("image/png", &vec![0; clipboard::MAX_ENCODED + 1]),
        Err(PasteError::Oversized)
    ));
    let mut bomb = png();
    bomb[16..20].copy_from_slice(&5000u32.to_be_bytes());
    // Recompute IHDR CRC with an independent simple implementation so this is
    // an otherwise valid header, not just a checksum failure.
    let mut crc = !0u32;
    for byte in &bomb[12..29] {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ if crc & 1 != 0 { 0xedb88320 } else { 0 };
        }
    }
    bomb[29..33].copy_from_slice(&(!crc).to_be_bytes());
    assert!(matches!(
        clipboard::decode("image/png", &bomb),
        Err(PasteError::Oversized)
    ));
    let mut draft = ClipboardDraft::default();
    let t = draft.begin().unwrap();
    draft.finish(t, clipboard::decode("image/png", &png()));
    let t = draft.begin().unwrap();
    draft.finish(t, Err(PasteError::Malformed));
    assert!(draft.image.is_some());
}
