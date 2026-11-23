# ADR 0001: Provisionally select the pinned native GPUI path

Status: **Provisionally selected for continued implementation; native support unverified** · 2026-11-23 · YUR-005
Feature `36be795f4590407eafd32028de2fc56e`.

## Context

Yurherdr needs native Windows, macOS and Linux text/IME, image clipboard, rich
documents, terminal surfaces and detachable panes. The person revised the testing
scope on 2026-11-23: “for now, lets skip thing we cant test on this box or on docker
desktop. lets just make sure decisions made and implementations chosen dont lock
some of those OS out”. This supersedes the earlier requirement to withhold every
toolkit decision until native three-platform tests are available. It defers those
tests, not their results or later support validation. The person confirmed
**Yurherdr / yurherdr** on 2026-11-23; that
confirmation does not complete YUR-007 storage/platform acceptance or YUR-008
environments. The repository's product plans remain separate from this probe.

## Implementation decision

Select **Rust + native GPUI Kit 0.11.0** provisionally using the exact monorepo pin in the
[runbook](../../prototypes/native-ui/README.md). Use Kit's re-exports, exact
gpui-pre snapshot family and a generated locked graph. Default native features
are explicitly component/assets/x11; resolved Wayland/native transitive features
are recorded in the [inventory](../../prototypes/native-ui/DEPENDENCIES.csv).
Do not enable Fast/webview or the optional JavaScript shell as a side effect.
Retain Windows, macOS and Linux as native targets. This decision authorizes a
direction for subsequent scoped implementation, not a release, Feature acceptance
or acceptance/merge of the inherited prototype.

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

The inherited implementation and its author-review/runtime evidence are frozen at
`d43d64d5a61e01a103d9e5df1d2cd1c31b1f449d` (YURH-3), against approved YUR-004
baseline `9d4d0cd5856f05c2b266d53294609aa756aade5a`. YURH-4 changes documentation
only. Its read-only source/manifest audit, capability matrix, exact carry-forward
results and follow-ups are in [the portability record](../NATIVE_UI_PORTABILITY.md).

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
open requirements. These gaps condition the choice; they do not establish OS
exclusion by the toolkit or prove that replacement adapters work.

The local proof binds final source to exact command exits and attempted
launches, including failed attempts. A later guarded Linux container run reported
the public GPUI backend **Wayland**, exited zero and removed its owned core
socket. No presentation screenshot or native interaction was observed. The first
exit-zero run lacked a backend observation and stays backend-unverified. Linux
native mode now rejects headless fallback. No startup/input/render/resource
benchmark is claimed. Research numerical thresholds remain proposals. No Windows
or macOS host, mixed-DPI hardware or remote/cloud execution has been used.

The first explicit headless rejection required wrapper termination (exit 124).
The probe now schedules startup on the running foreground loop, covering both
headless rejection and initial-window-open failure; synchronous quit in Linux's
pre-loop launch callback was erased by calloop's startup reset. The regression
test requires prompt nonzero exit and owned-core cleanup without a wrapper kill.
The failed attempt remains in local evidence; it is not a passing isolation run.

## Narrow alternatives

Qt's pinned QDockWidget source provides a known floating-dock comparison point;
see the [source/terms record](../../prototypes/native-ui/NOTICE.md). It was not
implemented or benchmarked. No concrete measured GPUI gap currently justifies
broad Iced/Slint/Qt implementation work. A missing environment prerequisite is
not a toolkit defect. Revisit alternatives only against a reproduced capability
gap, with exact versions, terms and equivalent workloads.

## Portability conditions and revisit gates

The inspected native toolkit/backend and core source paths show no fundamental
exclusion of Windows, macOS or Linux within this audit's scope. Target resolution
is not compilation, and source support is not runtime proof. The probe currently
fails closed on Windows ownership checks and Windows/macOS core harnesses. The
pinned core itself has Windows named pipes/ConPTY/job objects and separate Unix/
macOS paths; do not transplant Linux sockets, peer credentials, PIDs or signals
into a Windows harness, or substitute WSL for native Windows.

Implement platform services at the existing guard/core/clipboard/PTY boundaries;
keep pane/session identities and domain contracts independent of OS handles.
Resolve the specific gaps and triggers in the portability matrix before claiming
the affected capability. In particular, Windows PTY deadline/cancellation behavior
and macOS core process ownership need implementation evidence, not cfg removal.

Continue checks that are meaningful on this box or already configured Docker
Desktop. Missing native hosts alone do not reopen this decision. Revisit the
choice if a required native capability cannot be supplied through those boundaries,
if a reproduced toolkit/API limitation requires pervasive forks or changes shared
domain/UI contracts, or if dependency/distribution terms cannot be satisfied.
Compare an alternative only against that concrete blocker and equivalent work.

Before announcing or releasing native support, obtain actual OS evidence for
windows, text/IME/candidates/focus/scale, bounded clipboard, rich content, PTY,
detach/re-dock/close/rollback, accessibility and performance/resource/lifecycle
workloads, plus packaging and notice clearance. Declare tested OS versions and
architectures then. The Feature owner retains acceptance and later test scheduling;
this local decision does not accept YUR-005 or authorize publication/deployment.
