//! Native Kit dependency smoke surface. Runtime is gated separately.
use gpui_kit::{component::label::Label, *};

pub struct Workspace;

impl Render for Workspace {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().child(Label::new("Yurherdr native UI evaluation"))
    }
}
