# Working on Yurherdr

This repository is the planned standalone native client over herdr-core. Read README.md and the [coding and interface rules](docs/CODING_RULES.md) before working. Consult `local/REWRITE_PLAN.md` when present. Workers in worktrees use the acceptance criteria in their briefs. Planned features must remain distinguishable from implemented behavior.

## Scope and isolation

- Target Windows, macOS and Linux. Under the owner's 2026-11-23 testing deferral, Rust + the pinned native GPUI Kit path is provisionally selected for continued implementation; follow [ADR 0001](docs/adr/0001-native-ui-toolkit.md) and its [portability conditions](docs/NATIVE_UI_PORTABILITY.md). Unavailable OS/desktop/hardware checks are deferred, never passed; do not block this implementation decision solely on missing hosts or announce native support before verification.
- Develop against a distinct application identity, configuration/data/cache/log directories and isolated herdr-core session/state/socket. Keep the normal HOME; do not run the old herdr-app or herdr-task under a changed HOME/HERDR_APP_HOME.
- The current IDE repository and live runtime are separate. Do not install into them, change their files/settings/services, restart their cores/windows, connect test workers to live spaces or synchronize helpers to linked machines for a test.
- Do not start a second Herdr App window. A future standalone prototype must pass its isolation guard before running and use its own development core.

## Naming and provenance

- Use the confirmed Yurherdr / yurherdr namespace throughout new code, commands, crates, manifests, storage and environment variables. The person confirmed spelling on 2026-11-23; YUR-007 platform/data-location acceptance remains open.
- Keep dependency/protocol identifiers such as herdr-core, hc and HC_SESSION intact. Legacy names belong only in upstream attribution, compatibility/import mappings and fixtures that test those mappings.
- Preserve upstream copyright, licenses and applicable notices for reused code, assets and test data. Never imply ownership of inherited work.

## Architecture and quality

- Follow typed contracts between domain state, core transport, provider adapters, storage, platform services and UI. Keep network/process/file I/O and heavy parsing outside rendering.
- Put each behavior in one canonical layer. Use capability-based agent adapters; configurable executable/argument definitions are distinct from structured provider integration.
- Validate configuration and external inputs. Pass commands as executable plus argument vectors, with explicit environment/cwd; handle Windows paths and quoting deliberately.
- Keep data updates atomic and operations idempotent. Bound subscriptions, history, queues, caches, image decoding and diagnostics. Cancellation and window lifetimes must not accidentally kill agents or retain views.
- Favor earned abstractions and readable modules over speculative frameworks, pass-through wrappers or scattered brand/platform checks.
- Plugins use the versioned extension boundary in docs/PLUGINS.md, including provider adapters, panes and service integrations. Reference plugins must remain independently installable and work without central brand switches. Do not treat process isolation as a security sandbox or claim existing Herdr plugins run unchanged.
- Every pane type, including plugin panes, must participate in detach/re-dock through the shared pane/window contract. Preserve agent/session identity and distinguish closing a view from stopping an agent.

## Verification and review

- Follow docs/REVIEW_POLICY.md, including the user-requested Thermo-Nuclear Code Quality Review and separate correctness/security checks for implementation changes.
- Add meaningful tests with behavior changes. Cover actual failure paths and ownership/race boundaries; avoid tests that only mirror the implementation.
- Follow docs/TEST_STRATEGY.md. Linux containers do not verify macOS/Windows GUI behavior. A successful compile or offscreen render is not multi-monitor, clipboard, accessibility or native lifecycle proof.
- Use this box or already configured Docker Desktop where useful; record unavailable checks and concrete revisit triggers. Keep local native safety, malformed-input bounds, view/session ownership and available regression failures as gates. Never bypass fail-closed platform guards to manufacture evidence or require WSL in place of native Windows.
- Use synthetic conversations/accounts/projects. Never copy live credentials or publish raw logs, private rule data, account identifiers or personal paths in proof.
- Record actual exit codes, the tested commit, OS/provider/tool versions and limits. Preserve the distinction between observed results and hypotheses.

## Changes and delivery

- Develop the standalone rewrite in this repository, starting work in roadmap dependency order. Keep the full task list local and excluded from Git. The current IDE is available for coordination and baseline reading. Use separate Yurherdr test environments for development and verification.
- For later implementation, use focused branches and obey the task’s review/merge settings. Do not force-push shared branches.
- A release, live cutover, production migration or new cloud expense requires the applicable explicit authorization. Never mark a user acceptance gate complete merely because a worker finished.
