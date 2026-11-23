//! Experimental native surfaces. No runtime entry bypasses either guard.
//! Uses the pinned Kit's public input/text APIs; no upstream terminal code.
use crate::{
    clipboard::{self, ClipboardDraft, DraftImage, PasteError},
    core::DevelopmentCore,
    isolation::{APP_ID, GuardError, GuardedRun},
    panes::{PaneId, Panes, Transfer, ViewLease, WindowId},
    terminal::{FixtureMode, Snapshot, Terminal},
};
use gpui_kit::{
    component::{
        ActiveTheme, StyledExt,
        button::Button,
        input::{Input, InputEvent, InputState},
    },
    *,
};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    ops::Range,
    rc::Rc,
    sync::{Arc, Mutex, mpsc},
    thread::JoinHandle,
    time::Duration,
};
mod document;
use document::DocumentSelection;

const DOCUMENT: &str = "# Synthetic evaluation\n\n**Rich text**, Unicode 界 and `inline code`.\n\n```rust\nlet draft = \"synthetic\";\n```\n\n| Pane | State |\n| --- | --- |\n| Composer | Draft |\n| Terminal | Owned child |\n\n[Local link](#composer) · links are disabled in this evaluation.\n\n![Local synthetic pixels](yurherdr://fixture/pixels)\n";
const IDS: [PaneId; 3] = [PaneId::Composer, PaneId::Document, PaneId::Terminal];
enum ImageOffer {
    Native,
    Fixture,
}
fn title(id: PaneId) -> &'static str {
    match id {
        PaneId::Composer => "Composer",
        PaneId::Document => "Rich document",
        PaneId::Terminal => "Synthetic terminal",
    }
}

enum TerminalCommand {
    Input(Panes, ViewLease, Vec<u8>),
    Resize(Panes, ViewLease),
}
struct FixtureRuntime {
    tx: Option<mpsc::SyncSender<TerminalCommand>>,
    thread: Option<JoinHandle<()>>,
    frame: Arc<Mutex<Arc<Snapshot>>>,
    stop: crate::terminal::OwnedStop,
}
impl FixtureRuntime {
    fn new(mut terminal: Terminal) -> Self {
        let frame = Arc::new(Mutex::new(terminal.snapshot()));
        let stop = terminal.stop_handle();
        let worker_stop = stop.clone();
        let output = frame.clone();
        let (tx, rx) = mpsc::sync_channel(8);
        let thread = std::thread::spawn(move || {
            let mut operation_error = None;
            loop {
                if worker_stop.is_requested() {
                    break;
                }
                match rx.recv_timeout(Duration::from_millis(20)) {
                    Ok(TerminalCommand::Input(p, v, bytes)) => {
                        operation_error = terminal.input(&p, v, &bytes).err();
                    }
                    Ok(TerminalCommand::Resize(p, v)) => {
                        operation_error = terminal.resize(&p, v, 20, 100).err();
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                }
                let mut frame = (*terminal.snapshot()).clone();
                if frame.error.is_none() {
                    frame.error = operation_error;
                }
                *output.lock().unwrap() = Arc::new(frame);
            }
            let _ = terminal.stop();
            *output.lock().unwrap() = terminal.snapshot();
        });
        Self {
            tx: Some(tx),
            thread: Some(thread),
            frame,
            stop,
        }
    }
    fn send(&self, command: TerminalCommand) -> bool {
        self.tx
            .as_ref()
            .is_some_and(|tx| tx.try_send(command).is_ok())
    }
    fn stop(&self) -> bool {
        self.stop.request().is_ok()
    }
}
impl Drop for FixtureRuntime {
    fn drop(&mut self) {
        self.stop();
        self.tx.take();
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

#[derive(Default, Clone)]
struct Draft {
    text: SharedString,
    selection: Range<usize>,
    reversed: bool,
    scroll: Point<Pixels>,
    focused: bool,
}
struct Model {
    run: GuardedRun,
    panes: Panes,
    drafts: BTreeMap<PaneId, Draft>,
    clipboard: ClipboardDraft,
    preview: Option<Arc<RenderImage>>,
    fixture: FixtureRuntime,
    main: Option<WeakEntity<Workspace>>,
    main_window: Option<AnyWindowHandle>,
    detached: BTreeMap<WindowId, (AnyWindowHandle, WeakEntity<PaneView>)>,
    next_window: u64,
    document: Option<Entity<base::text::TextViewState>>,
    document_scroll: ScrollHandle,
    document_focused: bool,
    document_selection: Option<DocumentSelection>,
}
type SharedModel = Rc<RefCell<Model>>;
struct AppOwner {
    _model: SharedModel,
    _core: DevelopmentCore,
}
impl Global for AppOwner {}

/// Every close only hides views. The global owner retains sessions until the
/// explicit Quit evaluation action (or the explicit bounded smoke timeout).
pub fn launch(run: GuardedRun, core: DevelopmentCore, smoke: bool) -> Result<(), GuardError> {
    run.native_profile(|name| std::env::var_os(name))?;
    core.verify_for(&run)?;
    let executable = std::env::current_exe().map_err(|_| GuardError::Io)?;
    let terminal =
        Terminal::start(&run, &executable, FixtureMode::Interactive).map_err(|_| GuardError::Io)?;
    let model = Rc::new(RefCell::new(Model {
        run,
        panes: Panes::default(),
        drafts: BTreeMap::new(),
        clipboard: ClipboardDraft::default(),
        preview: None,
        fixture: FixtureRuntime::new(terminal),
        main: None,
        main_window: None,
        detached: BTreeMap::new(),
        next_window: 1,
        document: None,
        document_scroll: ScrollHandle::new(),
        document_focused: false,
        document_selection: None,
    }));
    let opened = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let observed = opened.clone();
    let app = gpui_kit::application().with_assets(gpui_kit::assets::Assets);
    let reopen_model = model.clone();
    app.on_reopen(move |cx| {
        if reopen_model.borrow().main_window.is_none() {
            let _ = open_workspace(reopen_model.clone(), cx);
        }
    });
    app.run(move |cx| {
        // Linux invokes its launch callback before calloop::run resets the
        // stop signal. A foreground task is dispatched by the running loop;
        // both headless rejection and initial-window failure can now quit.
        cx.spawn(async move |cx| {
            cx.update(move |cx| {
                cx.set_app_identity(APP_ID, "Yurherdr UI evaluation");
                gpui_kit::init(cx);
                cx.set_global(AppOwner {
                    _model: model.clone(),
                    _core: core,
                });
                if smoke {
                    // Public connected-platform observation, not an inference from
                    // DISPLAY/WAYLAND_DISPLAY or successful headless window creation.
                    eprintln!("Yurherdr probe display backend: {}", cx.compositor_name());
                }
                #[cfg(target_os = "linux")]
                if cx.graphical_environment().is_none() {
                    eprintln!("Yurherdr probe: headless backend; OS window unavailable");
                    cx.quit();
                    return;
                }
                if smoke {
                    cx.spawn(async move |cx| {
                        cx.background_executor().timer(Duration::from_secs(5)).await;
                        cx.update(|cx| cx.quit());
                    })
                    .detach();
                }
                if !open_workspace(model.clone(), cx) {
                    cx.quit();
                } else {
                    observed.store(true, std::sync::atomic::Ordering::Release);
                }
            });
        })
        .detach();
    });
    if opened.load(std::sync::atomic::Ordering::Acquire) {
        Ok(())
    } else {
        Err(GuardError::Io)
    }
}
fn open_workspace(model: SharedModel, cx: &mut App) -> bool {
    gpui_kit::open_window(options("Yurherdr native evaluation"), cx, |window, cx| {
        let workspace = cx.new(|cx| Workspace::new(model.clone(), window, cx));
        model.borrow_mut().main = Some(workspace.downgrade());
        model.borrow_mut().main_window = Some(window.window_handle());
        let weak = workspace.downgrade();
        window.on_window_should_close(cx, move |window, cx| {
            let _ = weak.update(cx, |this, cx| {
                for view in this.views.values() {
                    view.update(cx, |view, cx| {
                        view.save(window, cx);
                        view.close();
                    });
                }
                this.views.clear();
                let mut model = this.model.borrow_mut();
                model.main = None;
                model.main_window = None;
            });
            true
        });
        workspace
    })
    .is_ok()
}
fn options(title: &str) -> WindowOptions {
    WindowOptions {
        titlebar: Some(TitlebarOptions {
            title: Some(title.to_owned().into()),
            ..Default::default()
        }),
        app_id: Some(APP_ID.into()),
        ..Default::default()
    }
}

struct Workspace {
    model: SharedModel,
    views: BTreeMap<PaneId, Entity<PaneView>>,
    _poll: Task<()>,
}
impl Workspace {
    fn new(model: SharedModel, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut views = BTreeMap::new();
        for id in IDS {
            let Ok(lease) = model.borrow_mut().panes.reopen(id, WindowId(0)) else {
                continue;
            };
            views.insert(
                id,
                cx.new(|cx| PaneView::new(model.clone(), lease, None, None, window, cx)),
            );
        }
        let _poll = cx.spawn(async move |weak, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(33))
                    .await;
                if weak.update(cx, |_, cx| cx.notify()).is_err() {
                    break;
                }
            }
        });
        Self {
            model,
            views,
            _poll,
        }
    }
    fn reopen(&mut self, id: PaneId, window: &mut Window, cx: &mut Context<Self>) {
        let lease = self.model.borrow_mut().panes.reopen(id, WindowId(0));
        if let Ok(lease) = lease {
            let view =
                cx.new(|cx| PaneView::new(self.model.clone(), lease, None, None, window, cx));
            self.views.insert(id, view);
            cx.notify();
        }
    }
}
impl Render for Workspace {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.model.borrow();
        let mut root = div()
            .v_flex()
            .size_full()
            .p_4()
            .gap_3()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child("Yurherdr · experimental native evaluation")
            .child("Closing a window keeps the fixture alive. Quit evaluation stops it.")
            .child(
                Button::new("quit")
                    .label("Quit evaluation")
                    .on_click(|_, _, cx| cx.quit()),
            );
        for id in IDS {
            if model
                .panes
                .view(id)
                .is_some_and(|v| v.window == WindowId(0))
            {
                root = root.child(self.views[&id].clone());
            } else if model.panes.view(id).is_none() {
                root = root.child(
                    Button::new(title(id))
                        .label(format!("Reopen {}", title(id)))
                        .on_click(
                            cx.listener(move |this, _, window, cx| this.reopen(id, window, cx)),
                        ),
                );
            }
        }
        root.id("workspace").overflow_y_scroll()
    }
}

struct PaneView {
    model: SharedModel,
    lease: ViewLease,
    input: Option<Entity<InputState>>,
    document: Option<Entity<base::text::TextViewState>>,
    pending: Option<Transfer>,
    message: SharedString,
    _subscriptions: Vec<Subscription>,
    _poll: Option<Task<()>>,
    _readiness: Option<Task<()>>,
    fixture_image: Arc<RenderImage>,
}
impl PaneView {
    fn new(
        model: SharedModel,
        lease: ViewLease,
        pending: Option<Transfer>,
        retire_window: Option<AnyWindowHandle>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let draft = model
            .borrow()
            .drafts
            .get(&lease.pane)
            .cloned()
            .unwrap_or_default();
        let input = if lease.pane != PaneId::Document {
            Some(cx.new(|cx| {
                let mut state = InputState::new(window, cx)
                    .placeholder(if lease.pane == PaneId::Terminal {
                        "Commit text with Enter"
                    } else {
                        "Type a synthetic draft; use Paste image for pictures"
                    })
                    .validate(|value, _| value.len() <= 65536);
                state.set_value(draft.text, window, cx);
                state.set_selected_range_with_direction(draft.selection, draft.reversed, cx);
                state.set_scroll_offset(draft.scroll, cx);
                if draft.focused {
                    state.focus(window, cx);
                }
                state
            }))
        } else {
            None
        };
        let document = if lease.pane == PaneId::Document {
            let existing = model.borrow().document.clone();
            let document = existing
                .unwrap_or_else(|| cx.new(|cx| base::text::TextViewState::markdown(DOCUMENT, cx)));
            model.borrow_mut().document = Some(document.clone());
            if pending.is_none() {
                let selection = model.borrow().document_selection.clone();
                document::restore(&document, selection.as_ref(), cx);
            }
            Some(document)
        } else {
            None
        };
        let mut subscriptions = vec![];
        if let Some(input) = &input {
            subscriptions.push(cx.subscribe_in(
                input,
                window,
                |this, input, event: &InputEvent, window, cx| {
                    if !this.model.borrow().panes.accepts(this.lease) {
                        return;
                    }
                    this.save(window, cx);
                    if matches!(event, InputEvent::PressEnter { .. })
                        && this.lease.pane == PaneId::Terminal
                    {
                        let mut text = input.read(cx).value().to_string();
                        text.push('\r');
                        if this.send_terminal(text.into_bytes()) {
                            this.message =
                                "Commit queued to the synthetic fixture; draft kept.".into();
                        }
                    }
                    cx.notify();
                },
            ));
        }
        if lease.pane == PaneId::Composer {
            model.borrow_mut().clipboard.reopen_view();
        }
        if lease.window != WindowId(0) {
            let weak = cx.entity().downgrade();
            window.on_window_should_close(cx, move |window, cx| {
                let _ = weak.update(cx, |this, cx| {
                    this.save(window, cx);
                    this.close();
                });
                true
            });
        }
        if let Some(ticket) = pending {
            let weak = cx.entity().downgrade();
            window.on_next_frame(move |window, cx| {
                let committed = weak
                    .update(cx, |this, cx| {
                        if this.model.borrow_mut().panes.ready(ticket).is_ok() {
                            let draft = this
                                .model
                                .borrow()
                                .drafts
                                .get(&this.lease.pane)
                                .cloned()
                                .unwrap_or_default();
                            if let Some(input) = &this.input {
                                input.update(cx, |state, cx| {
                                    state.set_value(draft.text, window, cx);
                                    state.set_selected_range_with_direction(
                                        draft.selection,
                                        draft.reversed,
                                        cx,
                                    );
                                    state.set_scroll_offset(draft.scroll, cx);
                                    if draft.focused {
                                        state.focus(window, cx);
                                    }
                                });
                            }
                            this.pending = None;
                            this._readiness.take();
                            if let Some(document) = &this.document {
                                // TextViewState's focus map belongs to App. We
                                // retain its logical selection/list state and
                                // explicitly rebind layout/focus at destination.
                                document.update(cx, |state, cx| state.invalidate_inline_layout(cx));
                                let selection = this.model.borrow().document_selection.clone();
                                document::restore(document, selection.as_ref(), cx);
                                if this.model.borrow().document_focused {
                                    let focus = document.read(cx).focus_handle().clone();
                                    focus.focus(window, cx);
                                }
                            }
                            cx.notify();
                            true
                        } else {
                            this.abandon_destination(window, cx);
                            false
                        }
                    })
                    .unwrap_or(false);
                if committed {
                    if let Some(old) = retire_window {
                        if let Some(model) = weak.upgrade().map(|view| view.read(cx).model.clone())
                        {
                            let retired = model
                                .borrow_mut()
                                .detached
                                .remove(&ticket.source_lease().window);
                            if let Some((_, view)) = retired {
                                let _ = view.update(cx, |view, _| view.close());
                            }
                        }
                        let _ = old.update(cx, |_, window, _| window.remove_window());
                    }
                    if ticket.source_lease().window == WindowId(0) {
                        let main = weak
                            .upgrade()
                            .and_then(|view| view.read(cx).model.borrow().main.clone());
                        if let Some(main) = main {
                            let _ = main.update(cx, |workspace, cx| {
                                if let Some(source) =
                                    workspace.views.remove(&ticket.source_lease().pane)
                                {
                                    source.update(cx, |view, _| view.close());
                                }
                                cx.notify();
                            });
                        }
                    }
                }
            });
        }
        let _poll = cx.spawn(async move |weak, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(33))
                    .await;
                if weak.update(cx, |_, cx| cx.notify()).is_err() {
                    break;
                }
            }
        });
        let _readiness = if pending.is_some() {
            Some(cx.spawn_in(window, async move |weak, cx| {
                cx.background_executor().timer(Duration::from_secs(3)).await;
                let _ = weak.update_in(cx, |this, window, cx| {
                    if this.pending.is_some() {
                        this.abandon_destination(window, cx);
                    }
                });
            }))
        } else {
            None
        };
        let fixture_image = render_image(DraftImage {
            width: 2,
            height: 2,
            rgba: vec![
                40, 160, 210, 255, 210, 160, 40, 255, 210, 160, 40, 255, 40, 160, 210, 255,
            ]
            .into(),
        });
        Self {
            model,
            lease,
            input,
            document,
            pending,
            message: "".into(),
            _subscriptions: subscriptions,
            _poll: Some(_poll),
            _readiness,
            fixture_image,
        }
    }
    fn save(&self, window: &Window, cx: &App) {
        if !self.model.borrow().panes.accepts(self.lease) {
            return;
        }
        if let Some(input) = &self.input {
            let state = input.read(cx);
            self.model.borrow_mut().drafts.insert(
                self.lease.pane,
                Draft {
                    text: state.value(),
                    selection: state.selected_range(),
                    reversed: state.is_selection_reversed(),
                    scroll: state.scroll_offset(),
                    focused: state.focus_handle(cx).is_focused(window),
                },
            );
        }
        if let Some(document) = &self.document {
            let state = document.read(cx);
            let mut model = self.model.borrow_mut();
            model.document_focused = state.focus_handle().is_focused(window);
            if let Some(selection) = document::capture(state) {
                model.document_selection = Some(selection);
            }
        }
    }
    fn close(&mut self) {
        self._poll.take();
        self._readiness.take();
        self._subscriptions.clear();
        let mut model = self.model.borrow_mut();
        if let Some(ticket) = self.pending.take() {
            let _ = model.panes.rollback(ticket);
        }
        if model.panes.accepts(self.lease) {
            let _ = model.panes.close(self.lease);
            if self.lease.pane == PaneId::Composer {
                model.clipboard.close_view();
            }
        }
    }
    fn abandon_destination(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.close();
        if self.lease.window != WindowId(0) {
            window.remove_window();
            return;
        }
        // A failed re-dock must not close the existing workspace and its other
        // panes. Retire only this provisional destination.
        let main = self.model.borrow().main.clone();
        let lease = self.lease;
        cx.defer(move |cx| {
            if let Some(main) = main {
                let _ = main.update(cx, |workspace, cx| {
                    let matches = workspace
                        .views
                        .get(&lease.pane)
                        .is_some_and(|view| view.read(cx).lease == lease);
                    if matches {
                        workspace.views.remove(&lease.pane);
                    }
                    cx.notify();
                });
            }
        });
    }
    fn clear_document_selection(&mut self, cx: &mut Context<Self>) {
        self.model.borrow_mut().document_selection = None;
        if let Some(document) = &self.document {
            document.update(cx, |state, cx| {
                state.clear_selection(cx);
                state.clear_range_highlights(cx);
            });
        }
    }
    fn copy_document(&mut self, window: &Window, cx: &mut Context<Self>) {
        if self.lease.pane != PaneId::Document {
            cx.propagate();
            return;
        }
        self.save(window, cx);
        let selection = self.model.borrow().document_selection.clone();
        if let Some(selection) = selection {
            cx.write_to_clipboard(ClipboardItem::new_string(selection.text));
            cx.stop_propagation();
        } else {
            cx.propagate();
        }
    }
    fn send_terminal(&mut self, bytes: Vec<u8>) -> bool {
        let model = self.model.borrow();
        if bytes.len() > 16384
            || !model.panes.accepts(self.lease)
            || !model.fixture.send(TerminalCommand::Input(
                model.panes.clone(),
                self.lease,
                bytes,
            ))
        {
            self.message = "Input busy or too large; try a shorter commit.".into();
            false
        } else {
            true
        }
    }
    fn move_view(&mut self, source_window: &mut Window, cx: &mut Context<Self>) {
        if let Some(input) = &self.input {
            let composing = input.update(cx, |state, cx| {
                state.marked_text_range(source_window, cx).is_some()
            });
            if composing {
                self.message = "Finish text composition before moving this pane.".into();
                cx.notify();
                return;
            }
        }
        self.save(source_window, cx);
        let destination = if self.lease.window == WindowId(0) {
            let mut m = self.model.borrow_mut();
            let id = WindowId(m.next_window);
            m.next_window = m.next_window.checked_add(1).expect("window ID exhausted");
            id
        } else {
            WindowId(0)
        };
        let ticket = match self
            .model
            .borrow_mut()
            .panes
            .reserve(self.lease, destination)
        {
            Ok(t) => t,
            Err(_) => {
                self.message = "Move already pending. Try again.".into();
                return;
            }
        };
        if self.lease.pane == PaneId::Composer {
            self.model.borrow_mut().clipboard.close_view();
        }
        let model = self.model.clone();
        let lease = ticket.destination_lease();
        let success = if destination == WindowId(0) {
            let handle = model.borrow().main_window;
            let main = model.borrow().main.clone();
            handle.is_some_and(|handle| {
                handle
                    .update(cx, |_, window, cx| {
                        main.as_ref().is_some_and(|main| {
                            main.update(cx, |workspace, cx| {
                                let view = cx.new(|cx| {
                                    PaneView::new(
                                        model.clone(),
                                        lease,
                                        Some(ticket),
                                        Some(source_window.window_handle()),
                                        window,
                                        cx,
                                    )
                                });
                                workspace.views.insert(lease.pane, view);
                                cx.notify();
                            })
                            .is_ok()
                        })
                    })
                    .unwrap_or(false)
            })
        } else {
            match gpui_kit::open_window(options(title(self.lease.pane)), cx, |window, cx| {
                cx.new(|cx| PaneView::new(model.clone(), lease, Some(ticket), None, window, cx))
            }) {
                Ok((handle, view)) => {
                    model
                        .borrow_mut()
                        .detached
                        .insert(destination, (handle, view.downgrade()));
                    true
                }
                Err(_) => false,
            }
        };
        if !success {
            let _ = self.model.borrow_mut().panes.rollback(ticket);
            if self.lease.pane == PaneId::Composer {
                self.model.borrow_mut().clipboard.reopen_view();
            }
            self.message = "Window could not open. Your pane stayed here.".into();
        }
        cx.notify();
    }
    fn read_image(&mut self, offer: ImageOffer, cx: &mut Context<Self>) {
        let (ticket, worker) = match self.model.borrow_mut().clipboard.begin_work() {
            Ok(t) => t,
            Err(e) => {
                self.message = e.message().into();
                cx.notify();
                return;
            }
        };
        let run = self.model.borrow().run.clone();
        let decode = cx.background_spawn(async move {
            let draft = match offer {
                ImageOffer::Native => crate::clipboard_native::read_image(&run)?,
                ImageOffer::Fixture => {
                    use image::{ExtendedColorType, ImageEncoder, codecs::png::PngEncoder};
                    let mut bytes = vec![];
                    PngEncoder::new(&mut bytes)
                        .write_image(&[40, 160, 210, 255], 1, 1, ExtendedColorType::Rgba8)
                        .map_err(|_| PasteError::Malformed)?;
                    clipboard::decode("image/png", &bytes)?
                }
            };
            let preview = render_image(draft.clone());
            Ok::<_, PasteError>((draft, preview))
        });
        cx.spawn(async move |weak, cx| {
            let _worker = worker;
            let decoded = decode.await;
            let _ = weak.update(cx, |this, cx| {
                let mut model = this.model.borrow_mut();
                match decoded {
                    Ok((image, preview)) => {
                        if model.clipboard.finish(ticket, Ok(image)) {
                            model.preview = Some(preview);
                        }
                    }
                    Err(error) => {
                        model.clipboard.finish(ticket, Err(error));
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }
}
impl Drop for PaneView {
    fn drop(&mut self) {
        self.close();
        if self.lease.window != WindowId(0) {
            self.model.borrow_mut().detached.remove(&self.lease.window);
        }
    }
}
fn render_image(image: DraftImage) -> Arc<RenderImage> {
    let mut pixels = image.rgba.to_vec();
    for pixel in pixels.as_chunks_mut::<4>().0 {
        pixel.swap(0, 2);
    }
    let buffer = image::RgbaImage::from_raw(image.width, image.height, pixels)
        .expect("validated RGBA dimensions");
    Arc::new(RenderImage::new(vec![image::Frame::new(buffer)]))
}

mod render;

#[cfg(all(test, feature = "ui-tests", target_os = "linux"))]
mod tests;
