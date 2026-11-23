# Guarded native UI evaluation

YUR-005 · Feature `36be795f4590407eafd32028de2fc56e` · experimental slice, 2026-11-23.

This single Rust crate evaluates GPUI Kit through its re-exports. It is not the
Yurherdr product, a full herdr-core client, a plugin host or a platform support
commitment. See the [Proposed ADR](../../docs/adr/0001-native-ui-toolkit.md) and
[dependency/notice record](NOTICE.md). The person confirmed Yurherdr / yurherdr
on 2026-11-23; YUR-007 and YUR-008 acceptance remain open.

## Implemented experimental slice

- A read-only guard rejects inherited HC_SESSION (including empty), malformed
  manifests, aliases/traversal, insecure or foreign ownership, hard-linked
  manifest files, directory collisions and legacy profile roots. All runtime
  entry points validate before creating a client, window or fixture child.
- A Linux core harness stages only an explicitly hashed independent build.
  It starts `serve` with separate session/state/config/socket, checks Linux peer
  PID/UID, protocol generation, version and on-disk core identity, then retains
  its owned child handle. No installed hc, discovery, autostart or live agents.
- A native workspace has Kit text inputs, bounded synthetic Markdown with code,
  table, disabled links and in-memory images, and an independently written PTY
  grid. Actual native behavior needs the host evidence below.
- All pane types share reserve/readiness/commit/rollback. Readiness uses the
  destination frame callback and a three-second timeout. Source views retire
  after commit; closing a view/window does not stop the fixture. **Quit
  evaluation** ends the app; **Stop fixture** is separate and idempotent.
- Draft text, UTF-8 selection/direction and scroll are reconstructed with fresh
  input/focus state. Active IME composition blocks moving until committed.
  The rich document retains its domain object and external scroll handle.
  Its logical source selection and Copy text live in the session; public Kit
  range highlights restore visible selection after window gesture state clears.
- Linux image reads use a separate pinned Wayland pipe adapter, not GPUI's
  uncapped native clipboard read. One owned helper has a three-second parent
  deadline, a two-second pipe deadline, four-MiB encoded limit and per-process
  heap/CPU limits. PNG/JPEG signature, dimensions, pixel/allocation limits and
  JPEG orientation are validated off rendering. Remove/Close invalidate late
  replies without allowing another physical worker until completion.

Stock input **keyboard and context-menu Paste are blocked before their native
read**. Typing and platform IME still use Kit input handling. Use **Paste image**
for the bounded Wayland path or **Use fixture image** for an explicitly synthetic
preview. X11, Windows and macOS image paste are unsupported in this increment;
Wayland requires a compositor exposing a compatible data-control protocol.
Process separation and resource limits are not a security sandbox.

## Build and checks

Use rustc/cargo **1.100.0**. `rust-toolchain.toml` declares it and `build.rs`
rejects another compiler release. On a system without rustup, invoke the
verified installed binaries directly. No global toolchain or package install is
performed by these sources. Keep HOME unchanged and use owned disposable caches:

```sh
YURHERDR_REPO=/absolute/path/to/this/checkout
export CARGO_HOME="$YURHERDR_REPO/local/native-ui-cargo"
export CARGO_TARGET_DIR="$YURHERDR_REPO/local/native-ui-target"
cargo build --manifest-path prototypes/native-ui/Cargo.toml --locked --features native
cargo fmt --manifest-path prototypes/native-ui/Cargo.toml -- --check
cargo clippy --manifest-path prototypes/native-ui/Cargo.toml --locked --all-targets --features native -- -D warnings
env -u HC_SESSION cargo test --manifest-path prototypes/native-ui/Cargo.toml --locked
```

The scoped `env -u` is for this synthetic test process, not a change to the
calling shell or an old-app helper. Private Unix socket tests need a context
permitted to bind their own socket. A sandbox denial is not a core/toolkit defect.
The independently built core test is ignored unless explicitly requested:

```sh
env -u HC_SESSION YURHERDR_TEST_CORE=/absolute/owned/build/herdr-core \
  cargo test --manifest-path prototypes/native-ui/Cargo.toml --locked --test core -- --ignored
env -u HC_SESSION YURHERDR_TEST_PROBE="$CARGO_TARGET_DIR/debug/yurherdr-native-ui-probe" \
  cargo test --manifest-path prototypes/native-ui/Cargo.toml --locked --features ui-tests --lib
```

`ui-tests` uses Kit's mock platform, real component layout/paint commands and a
real guarded synthetic PTY. It checks both window-close orders, response after
reopen, source retirement, cancelled destination/readiness timeout, scroll through
detach/re-dock, painted selection quads, Copy and captured Paste. These are not
native desktop, IME, accessibility or pixel/presentation tests.

Regenerate the target inventory with Python 3.10+ and the same Cargo/cache:

```sh
python3 prototypes/native-ui/dependency_inventory.py \
  --evidence local/native-ui-metadata \
  --output prototypes/native-ui/DEPENDENCIES.csv
```

The inventory uses target-specific Cargo **build trees**; metadata includes
inactive optional edges and cannot alone establish the compiled graph. The CSV
includes host build dependencies, feature unions and root notice digests. It is
resolved on the observed Linux host; it is not a build on the other hosts or
complete per-file/native-library license clearance.

## Independent core preparation

Do this only in an owned ignored/disposable checkout. Resolve these exact inputs:

- herdr-core `300a455b8cfa91f8f068b49bb9d78b94d3bd4f0e`, version 0.1.22.
- Its `vendor/ghostty` gitlink `1e5cdb865807abc990181efb8ebfa9122955cce2`.
- Official Linux x86_64 Zig 0.16.0 archive and published SHA-256 in [NOTICE](NOTICE.md).

Read their manifests/licenses/build scripts first. Build the core with its own
Cargo.lock and an explicit local Zig path. Scope `ZIG_GLOBAL_CACHE_DIR`,
`ZIG_LOCAL_CACHE_DIR`, CARGO_HOME and CARGO_TARGET_DIR to owned directories.
Keep HOME unchanged. Do not run upstream install/service/update scripts. The
inspected `serve` contract uses HC_SESSION, HC_SOCKET, XDG_CONFIG_HOME,
XDG_STATE_HOME and XDG_RUNTIME_DIR; a custom runtime directory avoids its Linux
systemd-user-manager/linger path. This is not an invented core CLI interface.

The receipt binds an audited build hash to the source pins; it is not a signed
source attestation. Never stage an installed/live binary or trust an arbitrary
self-declared receipt. The independently built core is not redistributed here.

## Guard and native launch

Launch from a terminal outside a live hc session. The guard intentionally rejects
any inherited HC_SESSION. Each launch needs a fresh private run; used state is
never silently reused. Example for Linux (the only implemented core harness):

```sh
YURHERDR_PROBE="$CARGO_TARGET_DIR/debug/yurherdr-native-ui-probe"
YURHERDR_NONCE=$(python3 -c 'import secrets; print(secrets.token_hex(32))')
"$YURHERDR_PROBE" --prepare-run /tmp --nonce "$YURHERDR_NONCE"
YURHERDR_RUN="/tmp/yurherdr-run-$(printf '%.16s' "$YURHERDR_NONCE")"
"$YURHERDR_PROBE" --guard-only "$YURHERDR_RUN" --nonce "$YURHERDR_NONCE"
python3 prototypes/native-ui/stage_core.py --probe "$YURHERDR_PROBE" \
  --core /absolute/owned/build/herdr-core --sha256 "$YURHERDR_CORE_SHA256" \
  --run-root "$YURHERDR_RUN" --nonce "$YURHERDR_NONCE"
python3 prototypes/native-ui/run_probe.py --probe "$YURHERDR_PROBE" \
  --run-root "$YURHERDR_RUN" --nonce "$YURHERDR_NONCE" --smoke
```

Set `YURHERDR_CORE_SHA256` from the independently verified local build record
before staging; an arbitrary receipt is not source verification.
The nonce is a run marker, not an authentication secret. The run directory suffix
is its first sixteen hex digits. `--prepare-run` provisions files without
starting runtime; `--guard-only` reads and emits a sanitized result with zero
runtime effects. To inspect the core alone, use `--core-check` on a separately
fresh staged run; it checks/stops/reaps and does not open a window.

The launcher sets config/data/cache/state and Yurherdr log paths **only for the
new probe process**, preserves normal HOME and native display endpoints, and
passes an allowlisted environment. Direct `--native` without those owned profile
paths fails closed. `--smoke` explicitly quits after five seconds; omit it for
interactive evaluation. Closing all windows keeps the session alive; reopen from
a remaining window or OS application activation where supported. Use **Quit
evaluation** to stop. No unknown path is removed automatically; failed or partial
run roots remain marked for inspection.

Linux native mode rejects a headless GPUI connection before opening windows.
Smoke diagnostics report the public connected-platform compositor/backend name.
An earlier exit-zero attempt without that observation remains backend-unverified;
window construction success alone is not OS presentation evidence.
An initial headless rejection printed an error but hung until the twenty-second
wrapper deadline (exit 124). Startup now runs as a foreground task after the
event loop starts, so both headless rejection and initial-window failure can
quit after calloop's stop-signal reset. A real independent-core regression test
requires headless exit two and completed core cleanup within ten seconds;
external timeout intervention fails that test. Actual OS window-open failure
has not been forced; its shared scheduled startup path is source-reviewed.
The later container attempt reported **Wayland**, exited zero and removed its
owned core socket. No presentation screenshot, native interaction or compositor/
GPU/IME version was observed; those acceptance cells remain pending.

The private root assumes one trusted same-user owner. It rejects static aliases
and revalidates before effects; it does not confine a malicious same-user actor
able to mutate owned files during a run. Windows ACL/reparse validation and
macOS/Windows independent-core harnesses are unimplemented and fail closed.

## Slice and evidence limits

The PTY runs this executable's synthetic child with argument vectors, explicit
cwd and an allowlisted environment. No shell/agent/provider runs. The grid is
at most 48×160, with 256 scrollback rows and an eight-MiB lifetime output budget;
the native surface starts at 24×80 and exposes a controlled resize. UTF-8 splits,
wide/combining cells, basic color/cursor, alternate screen, bracketed paste and
SGR mouse input have fixture tests. OSC clipboard/title actions, sixel/graphics,
full keyboard encoding, native scrollback UI, glyph/DPI hit geometry and a full
terminal emulator are outside the slice. Flood reports a limit; Stop reaps the
owned fixture. Unix writes use a nonblocking owned descriptor, a two-second
deadline and an independent stop handle; teardown bypasses the input queue.
Stalled-reader integration tests cover Stop and worker Drop within two seconds.
Queued commits keep the input draft because a timeout can follow a partial write;
retrying may repeat an already written prefix. Window lifetimes do not own the child.

Input undo-history migration, geometric/keyboard selection extension and
accessibility selection across window transfer remain unproven; the preserved
document highlight/Copy is session-owned logical state, not restored native
gesture state. Actual platform composition/candidate windows and native clipboard
interoperability need the matrix below. No remote image, browser or external link
is loaded, and no image is sent to a provider.

| Evidence | Linux container | Actual Linux desktop | Windows host | macOS host |
| --- | --- | --- | --- | --- |
| Rust native dependency/build path | Observed; exact head/exits in local proof | Pending | Pending | Pending |
| Guard/domain/owned PTY/core identity | Observed fixture contracts | Pending host repeat | ACL/core harness pending | Core harness pending |
| Mock-platform component/window tests | Observed; not desktop proof | Not a substitute | Pending | Pending |
| Native windows/text/IME/image/terminal/rich text/detach | See dated local run evidence; no acceptance | Pending | Pending | Pending |
| Accessibility/mixed DPI/monitors/GPU/long session | Not proven | Pending | Pending | Pending |

For every host capture exact commit/lock/toolchain, OS/architecture, GPU/driver,
compositor, scale/refresh, IME and screen reader versions. Record actual exits and
failures. Startup (cold/warm repeated launches), input/render latency, CPU/memory,
handle/child counts and long-session lifecycle workloads are **pending**. Test
durations and a compile are not performance measurements. Research thresholds
remain proposals, not product requirements. No remote host or cloud CI was used.
