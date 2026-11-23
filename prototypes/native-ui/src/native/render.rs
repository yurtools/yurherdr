//! Pure native presentation; I/O and parsing are owned outside rendering.
use super::*;

impl Render for PaneView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.model.borrow().panes.accepts(self.lease) {
            return div()
                .child(if self.pending.is_some() {
                    "Opening pane…"
                } else {
                    "Pane moved. You can close this view."
                })
                .into_any_element();
        }
        let mut content =
            div()
                .capture_action(cx.listener(|this, _: &base::input::Copy, window, cx| {
                    this.copy_document(window, cx)
                }))
                .capture_action(cx.listener(|this, _: &base::input::SelectAll, _, cx| {
                    if this.lease.pane == PaneId::Document {
                        this.clear_document_selection(cx);
                    }
                    cx.propagate();
                }))
                .capture_action(cx.listener(|this, _: &base::input::Paste, _, cx| {
                    // Before both the stock paste handler and its on_paste hook:
                    // that hook itself performs the same uncapped native read.
                    cx.stop_propagation();
                    this.message =
                    "Text paste is disabled in this evaluation. Typing and IME remain available."
                        .into();
                    cx.notify();
                }))
                .v_flex()
                .gap_2()
                .p_3()
                .border_1()
                .border_color(cx.theme().border)
                .child(
                    div()
                        .h_flex()
                        .gap_2()
                        .child(title(self.lease.pane))
                        .child(
                            Button::new("move")
                                .label(if self.lease.window == WindowId(0) {
                                    "Detach"
                                } else {
                                    "Re-dock"
                                })
                                .on_click(
                                    cx.listener(|this, _, window, cx| this.move_view(window, cx)),
                                ),
                        )
                        .child(
                            Button::new("close")
                                .label("Close view")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.save(window, cx);
                                    this.close();
                                    if this.lease.window == WindowId(0) {
                                        let main = this.model.borrow().main.clone();
                                        let pane = this.lease.pane;
                                        cx.defer(move |cx| {
                                            if let Some(main) = main {
                                                let _ = main.update(cx, |workspace, cx| {
                                                    workspace.views.remove(&pane);
                                                    cx.notify();
                                                });
                                            }
                                        });
                                    } else {
                                        window.remove_window();
                                    }
                                    cx.notify();
                                })),
                        ),
                );
        content = content
            .child(
                Button::new("quit")
                    .label("Quit evaluation")
                    .on_click(|_, _, cx| cx.quit()),
            )
            .child(
                Button::new("workspace")
                    .label("Open workspace")
                    .on_click(cx.listener(|this, _, _, cx| {
                        let absent = this.model.borrow().main_window.is_none();
                        if absent {
                            let _ = open_workspace(this.model.clone(), cx);
                        }
                    })),
            );
        match self.lease.pane {
            PaneId::Composer => {
                content = content
                    .child(
                        Input::new(self.input.as_ref().unwrap())
                            .disabled(self.model.borrow().panes.moving(self.lease.pane)),
                    )
                    .child(Button::new("paste-image").label("Paste image").on_click(
                        cx.listener(|this, _, _, cx| this.read_image(ImageOffer::Native, cx)),
                    ))
                    .child(
                        Button::new("fixture-image")
                            .label("Use fixture image")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.read_image(ImageOffer::Fixture, cx)
                            })),
                    )
                    .child(Button::new("remove-image").label("Remove image").on_click(
                        cx.listener(|this, _, _, cx| {
                            let mut model = this.model.borrow_mut();
                            model.clipboard.remove_image();
                            model.preview = None;
                            cx.notify();
                        }),
                    ));
                let model = self.model.borrow();
                if let Some(preview) = &model.preview {
                    content = content.child(img(preview.clone()).max_w(px(240.)).max_h(px(160.)));
                }
                if let Some(error) = model.clipboard.error {
                    content = content.child(error.message());
                }
            }
            PaneId::Document => {
                let image = self.fixture_image.clone();
                let scroll = self.model.borrow().document_scroll.clone();
                content = content.child(
                    div()
                        .id("document-scroll")
                        .capture_any_mouse_down(cx.listener(
                            |this, event: &MouseDownEvent, _, cx| {
                                if event.button != MouseButton::Left {
                                    return;
                                }
                                if this.model.borrow().panes.moving(this.lease.pane) {
                                    cx.stop_propagation();
                                } else {
                                    this.clear_document_selection(cx);
                                }
                            },
                        ))
                        .max_h(px(220.))
                        .overflow_y_scroll()
                        .track_scroll(&scroll)
                        .child(
                            base::text::TextView::new(self.document.as_ref().unwrap())
                                .selectable(!self.model.borrow().panes.moving(self.lease.pane))
                                .scrollable(false)
                                .image_source(move |_| ImageSource::Render(image.clone()))
                                .on_link_click(|_, _, _, _| {}),
                        ),
                );
            }
            PaneId::Terminal => {
                let frame = self.model.borrow().fixture.frame.lock().unwrap().clone();
                let mut grid = div()
                    .v_flex()
                    .font_family("monospace")
                    .text_size(px(13.))
                    .bg(rgb(0x101318));
                for row in 0..frame.rows {
                    let mut line = div().h_flex().h(px(17.));
                    for col in 0..frame.cols {
                        let cell = &frame.cells
                            [usize::from(row) * usize::from(frame.cols) + usize::from(col)];
                        if cell.continuation {
                            continue;
                        }
                        line = line.child(
                            div()
                                .id((
                                    "cell",
                                    u64::from(row) * u64::from(frame.cols) + u64::from(col),
                                ))
                                .w(px(if cell.wide { 16. } else { 8. }))
                                .h(px(17.))
                                .text_color(terminal_color(cell.fg, true))
                                .bg(if frame.cursor == (row, col) {
                                    rgb(0x525966)
                                } else {
                                    terminal_color(cell.bg, false)
                                })
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |this, _, _, _| {
                                        if this
                                            .model
                                            .borrow()
                                            .fixture
                                            .frame
                                            .lock()
                                            .unwrap()
                                            .sgr_mouse
                                        {
                                            this.send_terminal(
                                                format!("\x1b[<0;{};{}M", col + 1, row + 1)
                                                    .into_bytes(),
                                            );
                                        }
                                    }),
                                )
                                .on_mouse_up(
                                    MouseButton::Left,
                                    cx.listener(move |this, _, _, _| {
                                        if this
                                            .model
                                            .borrow()
                                            .fixture
                                            .frame
                                            .lock()
                                            .unwrap()
                                            .sgr_mouse
                                        {
                                            this.send_terminal(
                                                format!("\x1b[<0;{};{}m", col + 1, row + 1)
                                                    .into_bytes(),
                                            );
                                        }
                                    }),
                                )
                                .child(if cell.text.is_empty() {
                                    " ".to_owned()
                                } else {
                                    cell.text.clone()
                                }),
                        );
                    }
                    grid = grid.child(line);
                }
                content =
                    content
                        .child(grid)
                        .child(
                            Input::new(self.input.as_ref().unwrap())
                                .disabled(self.model.borrow().panes.moving(self.lease.pane)),
                        )
                        .child(Button::new("resize").label("Resize terminal").on_click(
                            cx.listener(|this, _, _, _| {
                                let m = this.model.borrow();
                                m.fixture
                                    .send(TerminalCommand::Resize(m.panes.clone(), this.lease));
                            }),
                        ))
                        .child(
                            Button::new("stop")
                                .label("Stop fixture")
                                .on_click(cx.listener(|this, _, _, _| {
                                    this.model.borrow().fixture.stop();
                                })),
                        );
                if frame.eof {
                    content =
                        content.child("Fixture ended. Close the evaluation to start a new run.");
                }
                if let Some(error) = frame.error {
                    content = content.child(if error == crate::terminal::TerminalError::Flood {
                        "Output limit reached. Stop the fixture."
                    } else {
                        "Input could not complete. Draft kept; stop or retry."
                    });
                }
            }
        }
        content.child(self.message.clone()).into_any_element()
    }
}
fn terminal_color(color: vt100::Color, foreground: bool) -> Rgba {
    match color {
        vt100::Color::Default => rgb(if foreground { 0xe4e7ec } else { 0x101318 }),
        vt100::Color::Rgb(r, g, b) => {
            rgb((u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b))
        }
        vt100::Color::Idx(i) => rgb([
            0x101318, 0xdc6068, 0x84c58a, 0xe5c58c, 0x7aa4df, 0xc493d6, 0x79bdc4, 0xe4e7ec,
        ][usize::from(i % 8)]),
    }
}
