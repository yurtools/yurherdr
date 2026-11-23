# ADR 0001: Evaluate the pinned native GPUI path

Status: **Proposed** · 2026-11-23 · YUR-005
Feature `36be795f4590407eafd32028de2fc56e`.

## Context

Yurherdr needs native Windows, macOS and Linux text/IME, image clipboard, rich
documents, terminal surfaces and detachable panes. The product toolkit remains
undecided. The person confirmed **Yurherdr / yurherdr** on 2026-11-23; that
confirmation does not complete YUR-007 storage/platform acceptance or YUR-008
environments. The repository's product plans remain separate from this probe.

## Proposed evaluation

Evaluate Rust with GPUI Kit first using the exact native monorepo pin in the
[runbook](../../prototypes/native-ui/README.md). Use Kit's re-exports, exact
gpui-pre snapshot family and a generated locked graph. Default native features
are explicitly component/assets/x11; resolved Wayland/native transitive features
are recorded in the [inventory](../../prototypes/native-ui/DEPENDENCIES.csv).
Do not enable Fast/webview or the optional JavaScript shell as a side effect.

The single experimental crate owns guard, core fixture, panes, terminal and
clipboard behavior. The core harness is deliberately narrow, with independently
pinned source/build, fresh state/session/socket and peer/identity verification.
It is not YUR-010's client. Runtime requires both the fail-closed prelaunch guard
and owned-core checks before GPUI/fixtures start.

The optional `gpui-shell` is assessed separately: its defaults enable
Fast/webview, and its JS runtime/license closure remains incomplete. A plugin
host is unnecessary for this native evaluation and is excluded. Process
separation alone would not establish a security sandbox.

An independent portable-pty/vt100 fixture avoids copying the GPL Zed terminal
crates or inheriting their application services. Its synthetic PTY and bounded
renderer/input contract exercise the small documented slice. All pane types use
one transfer transaction; closing views retains domain/session ownership.

## Evidence obtained and limits

On the observed Linux x86_64 container, the pinned native dependency check/build
and independent pinned-core build succeeded with Rust 1.100.0 and local Zig
0.16.0. Automated guard, malformed-image, stale-reply, PTY and mock-platform
window tests exercise actual failure paths. Mock-platform tests include nonzero
document scroll before/after detach/re-dock, public painted range highlights,
logical Copy, source retirement and both window-close orders. They do not prove
OS window gestures, IME candidate windows, accessibility or hardware rendering.

The stock Kit clipboard backend reads uncapped bytes even for text-only Paste.
This probe captures those entrypoints and blocks them. A narrow Linux Wayland
pipe adapter applies bounded reads, helper deadlines/resource limits and bounded
off-render decode. No broad toolkit fork is introduced. Real Wayland clipboard
interoperability, X11/macOS/Windows adapters and working bounded text paste remain
open requirements. This gap is not toolkit selection evidence.

The local proof binds final source to exact command exits and any attempted
native launch, including failed attempts. No startup/input/render/resource
benchmark is claimed. Research numerical thresholds remain proposals. No Windows
or macOS host, mixed-DPI hardware or remote/cloud execution has been used.

## Narrow alternatives

Qt's pinned QDockWidget source provides a known floating-dock comparison point;
see the [source/terms record](../../prototypes/native-ui/NOTICE.md). It was not
implemented or benchmarked. No concrete measured GPUI gap currently justifies
broad Iced/Slint/Qt implementation work. A missing environment prerequisite is
not a toolkit defect. Revisit alternatives only against a reproduced capability
gap, with exact versions, terms and equivalent workloads.

## Decision gate

No toolkit is selected by this ADR. Keep it Proposed until the owner reviews
measured three-platform evidence: native windows, real platform text/IME and
candidate/focus/scale behavior, bounded native image paste, rich content, real
PTY surface and detachable panes. Include lifecycle/close/rollback, accessibility,
GPU/compositor/scale/monitor versions, performance/resource workloads and complete
distribution notice review. Resolve unsupported platform guards/core harnesses,
clipboard paths and documented selection/history limits first. The Feature
Assistant owns the next host selection and acceptance steps; this local increment
does not accept YUR-005 or authorize deployment/publication.
