//! Mock-platform GPUI windows + a real guarded local PTY. Not desktop proof.
use super::{FixtureRuntime, Model, SharedModel, open_workspace};
use crate::{
    clipboard::ClipboardDraft,
    panes::{PaneId, Panes, WindowId},
    terminal::{FixtureMode, Terminal},
};
use gpui_kit::component::ActiveTheme;
use gpui_kit::{
    AnyWindowHandle, ClipboardItem, ScrollHandle, TestAppContext, VisualTestContext, base, point,
    px,
};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(1);
struct Layout(PathBuf);
impl Drop for Layout {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn fixture(mode: FixtureMode) -> (Layout, SharedModel) {
    let number = NEXT.fetch_add(1, Ordering::Relaxed);
    let parent =
        std::env::temp_dir().join(format!("yurherdr-ui-tests-{}-{number}", std::process::id()));
    fs::create_dir(&parent).unwrap();
    fs::set_permissions(&parent, fs::Permissions::from_mode(0o700)).unwrap();
    let nonce = format!("{:08x}{number:08x}{:048x}", std::process::id(), 0);
    let root = crate::isolation::prepare(&parent, &nonce).unwrap();
    let run = crate::isolation::validate(&root, &nonce, None).unwrap();
    let executable = std::env::var_os("YURHERDR_TEST_PROBE")
        .expect("supply the independently built task-owned probe executable");
    let terminal = Terminal::start(&run, &PathBuf::from(executable), mode).unwrap();
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
    (Layout(parent), model)
}
fn frame(window: AnyWindowHandle, cx: &mut TestAppContext) {
    window
        .update(cx, |_, window, cx| {
            window.simulate_next_frame(cx);
            let _ = window.draw(cx);
        })
        .unwrap();
    cx.run_until_parked();
}
fn close(window: AnyWindowHandle, cx: &mut TestAppContext) {
    let mut visual = VisualTestContext::from_window(window, cx);
    assert!(visual.simulate_close());
    window
        .update(cx, |_, window, _| window.remove_window())
        .unwrap();
    cx.run_until_parked();
}
fn detach(model: &SharedModel, pane: PaneId, cx: &mut TestAppContext) -> AnyWindowHandle {
    let main = model.borrow().main.as_ref().unwrap().upgrade().unwrap();
    let main_window = model.borrow().main_window.unwrap();
    main_window
        .update(cx, |_, window, cx| {
            main.update(cx, |workspace, cx| {
                workspace.views[&pane].update(cx, |view, cx| view.move_view(window, cx))
            })
        })
        .unwrap();
    let destination = model.borrow().detached.values().next().unwrap().0;
    frame(destination, cx);
    frame(main_window, cx);
    destination
}
fn selection_quads(window: AnyWindowHandle, cx: &mut TestAppContext) -> usize {
    window
        .update(cx, |_, window, cx| {
            let background: gpui_kit::Background = cx.theme().selection.into();
            window
                .painted_quads()
                .iter()
                .filter(|quad| quad.background == background)
                .count()
        })
        .unwrap()
}

#[gpui_kit::test]
fn closing_main_and_detached_in_both_orders_keeps_fixture_alive(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);
    for main_first in [true, false] {
        let (_layout, model) = fixture(FixtureMode::Interactive);
        cx.update(|cx| assert!(open_workspace(model.clone(), cx)));
        let main = model.borrow().main_window.unwrap();
        frame(main, cx);
        let detached = detach(&model, PaneId::Terminal, cx);
        assert_eq!(
            model.borrow().panes.view(PaneId::Terminal).unwrap().window,
            WindowId(1)
        );
        let source = model.borrow().main.as_ref().unwrap().upgrade().unwrap();
        cx.read(|cx| assert!(!source.read(cx).views.contains_key(&PaneId::Terminal)));
        if main_first {
            close(main, cx);
            close(detached, cx);
        } else {
            close(detached, cx);
            close(main, cx);
        }
        assert!(model.borrow().panes.view(PaneId::Terminal).is_none());
        assert!(
            !model.borrow().fixture.frame.lock().unwrap().eof,
            "closing views must not stop fixture"
        );
        cx.update(|cx| assert!(open_workspace(model.clone(), cx)));
        let lease = model.borrow().panes.view(PaneId::Terminal).unwrap();
        model.borrow().fixture.send(super::TerminalCommand::Input(
            model.borrow().panes.clone(),
            lease,
            b"still alive".to_vec(),
        ));
        let until = std::time::Instant::now() + std::time::Duration::from_secs(2);
        loop {
            let frame = model.borrow().fixture.frame.lock().unwrap().clone();
            let text: String = frame.cells.iter().map(|cell| cell.text.as_str()).collect();
            if text.contains("still alive") {
                break;
            }
            assert!(
                std::time::Instant::now() < until,
                "fixture did not survive view close"
            );
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let reopened = model.borrow().main_window.unwrap();
        close(reopened, cx);
        drop(source);
        drop(model);
    }
}

#[gpui_kit::test]
fn document_domain_and_scroll_survive_transfer_and_stock_paste_is_captured(
    cx: &mut TestAppContext,
) {
    cx.update(gpui_kit::init);
    let (_layout, model) = fixture(FixtureMode::Interactive);
    cx.update(|cx| assert!(open_workspace(model.clone(), cx)));
    let main = model.borrow().main_window.unwrap();
    frame(main, cx);
    let document = model.borrow().document.clone().unwrap();
    main.update(cx, |_, window, cx| {
        let focus = document.read(cx).focus_handle().clone();
        focus.focus(window, cx);
    })
    .unwrap();
    frame(main, cx);
    cx.dispatch_action(main, base::input::SelectAll);
    cx.run_until_parked();
    let selected = cx.read(|cx| document.read(cx).selected_text());
    assert!(!selected.is_empty());
    model
        .borrow()
        .document_scroll
        .set_offset(point(px(0.), px(-25.)));
    frame(main, cx);
    let offset = model.borrow().document_scroll.offset();
    assert!(f32::from(offset.y) < 0., "fixture must actually scroll");
    let detached = detach(&model, PaneId::Document, cx);
    assert_eq!(
        document.entity_id(),
        model.borrow().document.as_ref().unwrap().entity_id()
    );
    assert_eq!(model.borrow().document_scroll.offset(), offset);
    assert_eq!(
        model.borrow().document_selection.as_ref().unwrap().text,
        selected
    );
    cx.dispatch_action(detached, base::input::Copy);
    cx.run_until_parked();
    cx.read(|cx| assert_eq!(cx.read_from_clipboard().unwrap().text().unwrap(), selected));
    // Remove the toolkit's transient gesture; the public range highlight must
    // still paint. Compare actual scene quads with highlights cleared/restored.
    cx.update(|cx| document.update(cx, |state, cx| state.clear_selection(cx)));
    frame(detached, cx);
    let highlighted = selection_quads(detached, cx);
    cx.update(|cx| document.update(cx, |state, cx| state.clear_range_highlights(cx)));
    frame(detached, cx);
    let plain = selection_quads(detached, cx);
    assert!(highlighted > plain, "restored selection must be painted");
    cx.update(|cx| {
        super::document::restore(&document, model.borrow().document_selection.as_ref(), cx)
    });
    frame(detached, cx);
    assert_eq!(selection_quads(detached, cx), highlighted);
    let view = model
        .borrow()
        .detached
        .values()
        .next()
        .unwrap()
        .1
        .upgrade()
        .unwrap();
    detached
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| view.move_view(window, cx))
        })
        .unwrap();
    frame(main, cx);
    assert!(model.borrow().detached.is_empty());
    assert_eq!(model.borrow().document_scroll.offset(), offset);
    assert!(selection_quads(main, cx) > 0);
    cx.dispatch_action(main, base::input::Copy);
    cx.run_until_parked();
    cx.read(|cx| assert_eq!(cx.read_from_clipboard().unwrap().text().unwrap(), selected));
    drop(view);
    let workspace = model.borrow().main.as_ref().unwrap().upgrade().unwrap();
    let composer = cx.read(|cx| workspace.read(cx).views[&PaneId::Composer].clone());
    main.update(cx, |_, window, cx| {
        composer.update(cx, |view, cx| {
            view.input.as_ref().unwrap().update(cx, |input, cx| {
                input.set_value("draft", window, cx);
                input.focus(window, cx);
            });
            cx.write_to_clipboard(ClipboardItem::new_string("must not paste".into()));
        })
    })
    .unwrap();
    frame(main, cx);
    cx.dispatch_action(main, base::input::Paste);
    cx.run_until_parked();
    cx.read(|cx| {
        let view = composer.read(cx);
        assert_eq!(
            view.input.as_ref().unwrap().read(cx).value().as_ref(),
            "draft"
        );
        assert!(view.message.contains("Text paste is disabled"));
    });
    cx.simulate_keystrokes(main, "ctrl-v");
    cx.read(|cx| {
        assert_eq!(
            composer
                .read(cx)
                .input
                .as_ref()
                .unwrap()
                .read(cx)
                .value()
                .as_ref(),
            "draft"
        )
    });
    close(main, cx);
    drop(composer);
    drop(workspace);
    drop(model);
}

#[gpui_kit::test]
fn closed_destination_source_close_and_readiness_timeout_roll_back(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);
    let (_layout, model) = fixture(FixtureMode::Interactive);
    cx.update(|cx| assert!(open_workspace(model.clone(), cx)));
    let main = model.borrow().main_window.unwrap();
    frame(main, cx);
    let workspace = model.borrow().main.as_ref().unwrap().upgrade().unwrap();
    let original = model.borrow().panes.view(PaneId::Composer).unwrap();
    main.update(cx, |_, window, cx| {
        workspace.update(cx, |workspace, cx| {
            workspace.views[&PaneId::Composer].update(cx, |view, cx| view.move_view(window, cx))
        })
    })
    .unwrap();
    let provisional = model.borrow().detached.values().next().unwrap().0;
    close(provisional, cx);
    frame(main, cx);
    assert_eq!(model.borrow().panes.view(PaneId::Composer), Some(original));
    assert!(!model.borrow().panes.moving(PaneId::Composer));
    let detached = detach(&model, PaneId::Terminal, cx);
    let view = model
        .borrow()
        .detached
        .values()
        .next()
        .unwrap()
        .1
        .upgrade()
        .unwrap();
    detached
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| view.move_view(window, cx))
        })
        .unwrap();
    close(detached, cx);
    frame(main, cx);
    drop(view);
    assert!(
        cx.windows().contains(&main),
        "failed re-dock must not close workspace"
    );
    assert!(model.borrow().panes.view(PaneId::Terminal).is_none());
    assert_eq!(model.borrow().panes.view(PaneId::Composer), Some(original));
    main.update(cx, |_, window, cx| {
        workspace.update(cx, |workspace, cx| {
            workspace.views[&PaneId::Composer].update(cx, |view, cx| view.move_view(window, cx))
        })
    })
    .unwrap();
    cx.run_until_parked();
    cx.background_executor
        .advance_clock(std::time::Duration::from_secs(4));
    cx.run_until_parked();
    assert_eq!(model.borrow().panes.view(PaneId::Composer), Some(original));
    assert!(!model.borrow().panes.moving(PaneId::Composer));
    assert!(!model.borrow().fixture.frame.lock().unwrap().eof);
    close(main, cx);
    drop(workspace);
    drop(model);
}

#[gpui_kit::test]
fn runtime_drop_interrupts_stalled_input_without_waiting_for_its_queue(_cx: &mut TestAppContext) {
    let (_layout, model) = fixture(FixtureMode::Stalled);
    let runtime = match Rc::try_unwrap(model) {
        Ok(model) => model.into_inner().fixture,
        Err(_) => panic!("unexpected view owner"),
    };
    let cleanup = runtime.stop.clone();
    let mut panes = Panes::default();
    let view = panes.reopen(PaneId::Terminal, WindowId(1)).unwrap();
    for _ in 0..8 {
        runtime.send(super::TerminalCommand::Input(
            panes.clone(),
            view,
            vec![b'x'; 16384],
        ));
    }
    std::thread::sleep(std::time::Duration::from_millis(100));
    let (done, result) = std::sync::mpsc::channel();
    let began = std::time::Instant::now();
    let dropper = std::thread::spawn(move || {
        drop(runtime);
        done.send(()).unwrap();
    });
    let completed = result.recv_timeout(std::time::Duration::from_secs(2));
    let _ = cleanup.request();
    dropper.join().unwrap();
    assert!(
        completed.is_ok(),
        "Drop must bypass blocked Input and queued work"
    );
    assert!(began.elapsed() < std::time::Duration::from_secs(2));
}
