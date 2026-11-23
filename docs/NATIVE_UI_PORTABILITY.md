# Native toolkit portability record

YUR-005 · Feature `36be795f4590407eafd32028de2fc56e` · YURH-4 · 2026-11-23.

**Decision: provisionally select Rust + the pinned native GPUI Kit path for
continued implementation. Windows, macOS and Linux remain native targets.**
The owner's deferral permits unavailable OS/desktop/hardware tests to wait; it
does not pass them. [ADR 0001](adr/0001-native-ui-toolkit.md) records authority,
conditions and revisit gates. Neither YURH-3 nor this follow-up accepts the
Feature, establishes supported OS versions or authorizes release/publication.

## Frozen inputs and evidence meaning

Implementation inspected: `d43d64d5a61e01a103d9e5df1d2cd1c31b1f449d` (YURH-3),
based on approved YUR-004 `9d4d0cd5856f05c2b266d53294609aa756aade5a`.
This follow-up changes documentation only; manifests, lock, notices, source and
tests stay byte-identical to that implementation. No native runtime is newly run.

| Input | Exact pin / role |
| --- | --- |
| GPUI Kit | 0.11.0, `e97fe438129c5e4ccab504f2e6fafd9a4e396bba`; Kit re-exports, native component/assets/x11 selected explicitly with defaults off |
| GPUI snapshot | Native `gpui-pre-*` family `=0.3.14`; package metadata references Zed `f126950f9d3b62666f97437ae5d4ccac2272c4db`; published package adjustments are not identical-source claims |
| Terminal fixture | portable-pty `=0.9.0`, vt100 `=0.16.2`; independent synthetic terminal, not the core's agent terminal |
| Clipboard | wl-clipboard-rs `=0.9.4`, **Linux-target dependency only**; image `=0.25.10`, PNG/JPEG only in the probe's direct decoder configuration |
| Independent core | herdr-core 0.1.22, `300a455b8cfa91f8f068b49bb9d78b94d3bd4f0e`; Ghostty vendor gitlink `1e5cdb865807abc990181efb8ebfa9122955cce2`; Zig 0.16.0 for its build |

The [notice](../prototypes/native-ui/NOTICE.md), [lock](../prototypes/native-ui/Cargo.lock)
and [inventory](../prototypes/native-ui/DEPENDENCIES.csv) retain integrity and
attribution details. All 868 lock package records were parsed; 614 distinct
package/version entries occur across the native normal/build target queries.
The lock also contains inactive optional/test packages: membership alone does
not mean a package is compiled. Fresh Linux-host queries for
`x86_64-unknown-linux-gnu`, `aarch64-apple-darwin` and `x86_64-pc-windows-msvc`
reproduced the tracked inventory byte-for-byte. These are dependency/feature
queries, **not builds, cross-compiles or native execution**. Other architectures
and OS minimum versions are not established by those triples.

### Primary source paths used

These references identify the inspected immutable source, not marketing support
claims. Registry source was read from archives matching Cargo.lock checksums.

- **K — native Kit:** [workspace pins](https://github.com/longbridge/gpui-kit/blob/e97fe438129c5e4ccab504f2e6fafd9a4e396bba/Cargo.toml),
  [Kit features](https://github.com/longbridge/gpui-kit/blob/e97fe438129c5e4ccab504f2e6fafd9a4e396bba/crates/kit/Cargo.toml),
  [application/open_window re-exports](https://github.com/longbridge/gpui-kit/blob/e97fe438129c5e4ccab504f2e6fafd9a4e396bba/crates/kit/src/lib.rs),
  `crates/component/src/input/state.rs`, `crates/base/src/input/base/state.rs`,
  `crates/base/src/text/` and `crates/assets/` at that same revision.
- **G — GPUI:** [platform selector/manifest](https://docs.rs/crate/gpui-pre-platform/0.3.14/source/)
  (`src/gpui_platform.rs`) selects MacPlatform, WindowsPlatform or Linux by target
  cfg. `gpui-pre-macos` `src/{platform,window}.rs` implements windows and
  NSTextInputClient/marked text; `gpui-pre-windows` `src/{platform,window,events}.rs`
  implements windows and WM_IME composition; `gpui-pre-linux`
  `src/linux/{platform,wayland/client,x11/client}.rs` implements windows,
  Wayland text-input-v3 and XIM paths. All are package version 0.3.14.
- **P — portable-pty:** [0.9.0 source](https://docs.rs/crate/portable-pty/0.9.0/source/),
  `src/{lib,cmdbuilder,unix}.rs`, `src/win/{conpty,psuedocon}.rs` (upstream spelling).
  The Windows builder produces UTF-16 executable/argv/environment/cwd with quoting;
  the native PTY selector has Unix and Windows implementations. vt100 0.16.2 is
  the independent parser, not an OS process launcher.
- **C — core:** [platform selector](https://github.com/herdrdev/herdr-core/blob/300a455b8cfa91f8f068b49bb9d78b94d3bd4f0e/crates/herdr-core/src/platform/mod.rs),
  [Windows service paths](https://github.com/herdrdev/herdr-core/blob/300a455b8cfa91f8f068b49bb9d78b94d3bd4f0e/crates/herdr-core/src/platform/windows.rs),
  `platform/{unix,windows_exec,windows_session,macos_launchd,macos_session}.rs`,
  `src/paths.rs`, `crates/herdr-core/Cargo.toml` and `crates/ghostty-vt/build.rs`
  at the same core revision. The cached complete tree identifies the Ghostty
  gitlink above; both core and populated vendor bytes were checked against archives.

## Capability matrix

“Local” below carries forward the exact YURH-3 Linux container results described
after the table; it is not new runtime proof. No additional Docker workload was
needed for this docs-only audit. Every native check listed is deferred/not run.
The implementing task owns the adapter and its checks; the Feature owner schedules
native validation before announcing the corresponding support.

Docker Desktop does not always mean Linux containers: an already configured
Windows-container engine could establish specifically tested Windows build/non-GUI
contracts within recorded image/kernel/isolation limits. Linux-container results
are Linux evidence. No container result establishes native desktop/IME/monitor
behavior. No Windows-container engine was verified for this task; no mode switch,
installation, host probing or new tests are requested. See the [testing strategy's primary
Docker/Microsoft references](TEST_STRATEGY.md#docker-hosted-ci-and-aws).

| Capability | Available local / Docker evidence | Source / dependency path | Contained gap or adapter needed | Deferred native check and concrete revisit trigger |
| --- | --- | --- | --- | --- |
| Native windows / rendering | Linux native build; Wayland backend smoke exit 0, no presentation capture; three target queries only | K/G selector, public `open_window`; Mac Metal, Windows DirectX, Linux WGPU | Probe guard/core prerequisites block non-Linux launch; no OS presentation observed | Real windows, focus, GPU/compositor, close/reopen on each OS; revisit if required window operations lack a usable public path or require pervasive forks |
| Text / IME | Kit mock component state/layout; no native composition or candidate-window evidence | K input/base state; G NSTextInputClient, WM_IME, Wayland text-input-v3/XIM | Transfer blocks active composition; undo-history migration and candidate/focus behavior open | Multilingual composition, commit/cancel, focus and scale through move/re-dock; revisit on reproducible input loss or unfixable candidate placement |
| Bounded image clipboard and text paste | Malformed/oversize/stale-image and pipe fixture tests; no compositor interoperability | Probe `clipboard_native.rs` → Linux wl-clipboard-rs; `clipboard.rs::decode` → bounded DraftImage; K/G stock read bypassed | X11/macOS/Windows adapters absent; stock text Paste blocked on every OS, including context menu; Wayland requires compatible data-control | Native PNG/JPEG and platform-format conversion, empty/stalled/oversize offers, bounded text paste; revisit if pre-allocation byte/decode/time bounds cannot be enforced at the native boundary |
| Real terminal surface | Owned synthetic Unix PTY/grid tests: UTF-8 splits, wide/combining cells, alternate screen, color/cursor, controlled resize/paste/mouse | P → vt100 → probe snapshots/render; separate C ConPTY/Unix PTYs for future core-backed client | Windows blocking writer/cancellation and non-Linux parent cleanup unproven; full emulator, native geometry and scrollback UI absent | Stalled reader, Stop/Drop/reap, EOF/flood, resize and glyph hit geometry on each OS; revisit before enabling Windows fixture or generalizing Linux deadline claims |
| Rich text / images / Copy | Bounded synthetic Markdown/code/table/image; mock nonzero scroll and painted range highlights/Copy | K base text/component; probe `native/document.rs` and session-owned source/ranges | Logical selection retained, native gesture/keyboard/accessibility selection and undo not restored/proven | Native selection, Copy formats, scrolling, shaping and transfer at scale; revisit if required selection/history needs private toolkit state or loses content |
| Detachable panes / multiple windows | Same transaction for every current pane; mock readiness failure/timeout/rollback, retirement and both close orders | Probe `panes.rs`, `native.rs` → K public windows/frame callbacks | Native monitor/focus/gesture behavior unverified; plugin panes are future contract work | Detach/re-dock, monitor removal, mixed DPI and close orders without session restart; revisit on identity loss or window lifetime owning an agent |
| Core IPC / session / process identity | Owned Linux core hash/peer/hello/status/on-disk identity + stop/reap; no installed discovery | Probe `core.rs::DevelopmentCore`; C platform Stream/Listener + Paths | macOS/Windows harnesses fail closed. Windows needs named-pipe server/user identity and owned handles; macOS needs safe owned launch/peer/lifecycle handling | Foreign endpoint, wrong session/build, startup failure, cancellation and parent exit per OS; revisit if required isolation cannot be implemented without live discovery or loss of process ownership |
| Arguments / environment / Unicode paths | Probe uses args_os/Path/OsStr, explicit exe/argv/cwd and cleared/allowlisted env; observed Linux fixtures only | Probe main/terminal/core/launcher; P UTF-16 builder; C windows_exec and environment paths | Guard requires UTF-8 clean canonical roots; Windows extended/UNC/case/reparse semantics and spaces/Unicode launches untested. Launcher is Linux runbook, not a portable launch claim | Drive/UNC/extended paths, spaces, quotes, Unicode, environment-key case and missing system vars; revisit if adapter requires shell-string interpolation or rejects required native paths |
| App directories / files / ownership | Linux private run/alias/hard-link/live-root rejection tests; normal HOME retained | Probe `isolation.rs`; C Paths XDG overrides, Windows LOCALAPPDATA/APPDATA defaults | Probe uses experiment-owned XDG/YURHERDR roots; Windows ACL/reparse/file identity absent; macOS/Windows live-profile exclusion and product locations unresolved | Private ACL/mode, aliases/reparse, collisions, atomic state and unchanged live stores; revisit before enabling each platform, never weaken the guard to launch |
| Fonts / assets / accessibility | Dependency paths and mock painting only; no screen-reader or font coverage proof | K assets and input semantics; G DirectWrite, Apple text/Metal, Linux font-kit; AccessKit Windows/macOS/Unix adapters | System font fallback and terminal monospace geometry unverified; custom terminal/document semantics incomplete/unproven; icon terms retained | Missing fonts/scripts, emoji, scaling, keyboard/focus and real screen readers on each OS; revisit if essential semantics cannot reach the native accessibility tree |
| Toolchains / packaging / distribution | Linux Rust 1.100.0 native build and independent Zig 0.16.0 core build inherited; target queries renewed | G Windows resource manifest + release `fxc.exe`; Apple `runtime_shaders`; C Ghostty build target mapping | Windows MSVC/SDK/resource/link/release shader path, Apple SDK/framework/signing, Linux native libraries/drivers and all installers untested | Native debug/release packaging with exact tools, notices, clean profile and uninstall; revisit on unsatisfiable tool/OS minimum or distribution terms |
| Performance / long session | No startup/input/render/resource benchmark; test durations are not benchmarks | G native dispatch/render + probe bounded workers/queues/history | Hardware/DPI/monitor coverage, growth/recovery/cancellation soak deferred | Measured equivalent workloads and budgets before support/release; revisit if required workloads cannot meet owner-approved budgets without replacing the toolkit |

## Platform conditions and smallest follow-ups

No fundamental OS exclusion was found **in these inspected boundaries**. This is
a source-audit conclusion, not a whole-program compile/correctness guarantee.
Classify existing restrictions as follows; do not remove cfg gates to claim support.

1. **Contained experimental harnesses:** `core.rs` is Linux-only, using UnixStream,
   SO_PEERCRED PID/UID, Linux path limits and PR_SET_PDEATHSIG. Its non-Linux
   `DevelopmentCore::start/verify_for` return PlatformNotAudited. Supply native
   implementations preserving `start/verify_for/stop` and the fresh run/session/
   source/endpoint identity contract. Domain panes/drafts do not need Unix sockets.
   macOS shares Unix sockets but not Linux peer credentials or parent-death API;
   core `macos_launchd::prepare_core` may hand `serve` off to launchd and exit, so
   a direct owned Child cannot simply be assumed to be the serving PID. Inspect
   the pinned launch contract before choosing an isolated harness; never start
   an unowned job/service or reuse its default `dev.herdr.core` launch labels.
2. **Replaceable safety/platform services:** Windows guard `check_ancestor`,
   `private` and `prepare` fail closed. Supply handle-based ACL/reparse/hard-link/
   file-identity and canonical-path checks, native live-root exclusion and
   owned profile preparation at `isolation.rs`; keep GuardedRun authority intact.
   Core Windows has named-pipe ACLs and ConPTY/job handling, but its file helpers
   inherit directory permissions; they do not implement this probe's ownership
   guard. Native named-pipe identity/deadline/teardown checks remain to be written.
3. **Replaceable clipboard service:** Linux-target wl-clipboard-rs and helper
   limits are contained behind `clipboard_native::read_image` returning DraftImage
   or PasteError; the non-Linux path returns NativeUnavailable. Implement native
   format acquisition/conversion and bounded text reads there/at the input
   boundary, retaining off-render decode, limits and stale-reply invalidation.
   This demonstrates a replacement location, not a working replacement.
4. **PTY cancellation gap requiring implementation follow-up:**
   `terminal.rs::owned_writer` makes the Unix fd nonblocking but non-Unix uses
   `take_writer`. portable-pty's ConPTY writer exposes a pipe Write; the probe's
   outer two-second input loop cannot enforce a deadline while a write blocks.
   Supply cancellable Windows writes and owned termination/reap, then exercise
   the same stalled-reader/Stop/Drop contract natively. Linux-only fixture
   parent-death setup also needs macOS/Windows owned cleanup equivalents.
   These are current capability gaps, not evidence that portable-pty excludes
   Windows, and not a newly reproduced Windows runtime failure.

Underlying core portability is not inferred from the probe. At the pinned C
source, Windows uses a native named-pipe namespace, rejects remote pipe clients,
constructs user/elevation-scoped pipe security, launches ConPTY and maps interrupt
versus termination to console input/job termination. Unix sockets and POSIX signals
are not its Windows implementation. macOS has additional coalition/session/bundle
behavior; Linux systemd/login environment paths are separate. None was executed
here on Windows/macOS. No WSL requirement is introduced.

Build feasibility remains separate from dependency availability. GPUI's Windows
resource build requires its manifest toolchain, and its release shader build
locates fxc.exe on a Windows host. The selected Apple runtime_shaders feature
stitches shader source instead of precompiling Metal with xcrun; this does not
remove Apple SDK/framework/link or runtime Metal requirements. Core's Ghostty
build maps Cargo targets to Zig (including MSVC static-library naming) and disables
Apple xcframework output. These paths need native debug/release verification;
Linux-host dependency queries do not run them.

## Terms and excluded boundaries

Native Kit/base/component/macros and inspected GPUI snapshot crates declare
Apache-2.0. portable-pty and vt100 retain their own MIT notices; wl-clipboard-rs
offers MIT OR Apache-2.0 with the MIT text retained. Kit assets carry Lucide ISC
and Feather-derived MIT notices. Independent core is Apache-2.0; Ghostty root
is MIT with separate nested obligations. Preserve [all retained notices](../prototypes/native-ui/NOTICE.md).
The 614-entry declared-license/root-terms inventory is not complete per-file,
system-library, font, shader-tool or final-package clearance; unresolved terms
remain a distribution gate, not permission for unrestricted redistribution.

Optional gpui-shell/JavaScript runtime, Fast and webview are excluded from the
native graph. Their defaults/terms and any future versioned plugin boundary
need separate decisions under [the plugin plan](PLUGINS.md); process isolation
is not a security sandbox. GPL Zed terminal/terminal_view was not copied.
The earlier narrow Qt floating-dock source comparison remains evidence only;
no measured structural blocker warrants Iced/Slint/Qt implementation here.

## Carried evidence, new checks and review limits

YURH-3 at **d43d64d5a61e01a103d9e5df1d2cd1c31b1f449d** recorded six exit-0
groups: fmt, clippy (`ui-tests`, all targets on Linux), native contracts, native
build, mock UI contracts and diff check. Native contracts passed 21 checks;
mock UI passed 5, counting one pipe check in both groups. Wayland-observed smoke
exited 0; deliberate headless rejection exited 2 promptly; each left its owned
core socket absent. No presentation screenshot/interaction, IME/clipboard
interoperability, performance or Windows/macOS execution was proven. The prior
stalled-writer interruption **130**, headless wrapper timeout **124**, earlier
compile/assertion failures and backend-unverified exit-0 attempt remain historical
failures/limits, not overwritten passes.

YURH-4 rechecked archive hashes/extracted bytes for Kit, native GPUI-family crates,
PTY/parser/Wayland, core and Ghostty; compared retained notice bytes; repeated
locked offline metadata/tree queries and compared the inventory (all successful,
exit 0). Initial verification mistakenly included inactive iOS source absent
from the cache (exit 1); restricting it to the native inventory succeeded without
fetching or changing pins. Initial default-cache offline query failed (Cargo 101,
wrapper 1); explicit verified cache succeeded. Web reader could not fetch the
exact source endpoints; a raw Windows-core retrieval failed sandbox DNS (curl 6),
then permitted retrieval exited 0 and matched the verified archive bytes.
These access failures are not platform defects.

Final diff, local-link, pin/claim consistency and Git-exclusion results, exact
commands/exits/environment, full final documentation commit and sanitized proof
are retained in ignored local YURH-4 evidence and its local recipe artifact.
This is limited **author documentation self-review**. Implementation structural
and correctness/security audits are **not newly run: no implementation change**.
Prior separate prototype reviews are attributed to the implementing YURH-3 author
and the source commit above, not independent fresh eyes or audits of this change.
Native adapters, product architecture, plugin host and distribution clearance
remain future scoped work. Remote-ready, push, PR, merge and Feature acceptance
are explicitly withheld.
