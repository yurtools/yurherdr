//! Logical document selection belongs to the session. Kit's window gesture
//! selection is transient; public range highlights and Copy restore the same
//! bounded source range after transfer without cloning private toolkit state.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DocumentSelection {
    pub source: Range<usize>,
    pub text: String,
}

pub(super) fn capture(state: &base::text::TextViewState) -> Option<DocumentSelection> {
    let source = state.selected_source_range()?;
    DOCUMENT.get(source.clone())?;
    let text = state.selected_text();
    if text.is_empty() || text.len() > DOCUMENT.len() * 2 {
        return None;
    }
    Some(DocumentSelection { source, text })
}

pub(super) fn restore(
    document: &Entity<base::text::TextViewState>,
    selection: Option<&DocumentSelection>,
    cx: &mut App,
) {
    let Some(selection) = selection else {
        return;
    };
    let rendered = document.read(cx).rendered_text();
    if rendered.source() != DOCUMENT {
        return;
    }
    if let Some(range) = rendered.range_for_source(selection.source.clone()) {
        let color = cx.theme().selection;
        let _ = document.update(cx, |state, cx| {
            state.set_range_highlights([base::text::RangeHighlight::new(range, color)], cx)
        });
    }
}
