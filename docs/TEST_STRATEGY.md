# Cross-platform testing strategy

Target Windows, macOS and Linux from the first prototype. Declare exact supported OS versions, architectures and provider versions only after evidence; target coverage is not current certification.

## Current implementation decision and deferred checks

For YUR-005, the owner's 2026-11-23 instruction permits a provisional implementation
choice using this box or already configured Docker Desktop. [ADR 0001](adr/0001-native-ui-toolkit.md)
selects the pinned Rust/native GPUI Kit path with [explicit portability conditions](NATIVE_UI_PORTABILITY.md).
Do not repeatedly require unavailable Windows/macOS/desktop/hardware hosts before
continuing that decision. Label those checks **deferred / not run**, with the
affected capability, source path, required adapter and revisit trigger. Neither
an absent host nor a successful target dependency query establishes a platform
defect or a platform pass.

This deferral does not waive available regression failures, safe local native
startup/cleanup, malformed-input bounds, cancellation or view/session ownership.
Keep unimplemented guards fail closed. Cross-target queries, cross-compilation,
mock/offscreen rendering and Linux containers each establish only their named
layer. Linux-container results are Linux evidence; an already configured Windows
container engine may establish specifically tested Windows build/non-GUI contracts
within its recorded image, kernel and isolation limits. No container result
establishes native desktop, IME or monitor behavior. No Windows engine is currently
exposed/verified here; this clarification requests no mode switch, installation,
host probing or new tests. Before
announcing or releasing OS support, execute the applicable native build/package,
desktop and hardware checks below and record exact versions and limits.

The YURH-4 follow-up is documentation-only: carry forward YURH-3's exact
`d43d64d5a61e01a103d9e5df1d2cd1c31b1f449d` results explicitly; do not rerun the
unchanged prototype suite gratuitously or label the carry-forward a new run.
Use diff/link/pin/exclusion checks and meaningful read-only target/feature
inspection. Follow the documentation-only review procedure, without claiming
new implementation structural or correctness/security audits.

## Test layers

| Layer | Environment | What it establishes |
| --- | --- | --- |
| Domain/unit/property tests | Native OS matrix; Linux Docker for reproducibility | State, policy, parsing, bounds, serialization and adapter logic |
| Core/agent contract tests | Isolated herdr-core with fake/recorded providers on each OS | Protocol, reconnect, lifecycle, path/argument handling and capabilities |
| Plugin contract/lifecycle tests | Synthetic extensions on each OS and every supported plugin runtime | Version negotiation, grants/revocation, sandbox/trust behavior, install/update rollback, crash containment and cleanup |
| Native build/package smoke tests | Real Windows/macOS/Linux runners | Compilation, dependencies, startup and packaging for the declared target |
| Native UI integration tests | Toolkit-supported interactive or virtual desktops on the actual OS | Input, rendering, clipboard, dialogs and view lifecycles within the tested environment |
| Hardware/desktop acceptance | Real desktop sessions on each OS | IME/accessibility, image paste, GPU/driver behavior, mixed-DPI monitors, pane detach/re-dock and monitor removal |
| Long-session/recovery tests | Disposable environments with bounded synthetic workloads | Memory/subscription growth, crash recovery, cancellation and duplicate-work prevention |

A headless or virtual display result must be labeled accordingly. Native clipboard image formats, accessibility permissions, focus, browser embedding and mixed-DPI behavior need platform-specific tests. GPU/software-rendered CI is not a desktop performance benchmark.

## Docker, hosted CI and AWS

Docker is useful for reproducible build and non-GUI integration workloads. Linux
containers provide Linux evidence, including when Docker Desktop runs them in a
Linux VM on Windows/macOS. Docker Desktop can also run Windows containers in
appropriate Windows configurations; it is not synonymous with Linux containers.
See [Docker's Windows installation requirements](https://docs.docker.com/desktop/setup/install/windows-install/)
(consulted 2026-11-23).

An already configured Windows-container engine could establish the particular
Windows build/non-GUI contracts actually tested, within the recorded image/kernel/
isolation limits. Process isolation shares the host kernel; Hyper-V isolation uses
a utility VM. Microsoft's [Windows container guidance](https://learn.microsoft.com/en-us/virtualization/windowscontainers/quick-start/lift-shift-to-containers)
distinguishes these modes and non-interactive workloads from desktop GUI support
(consulted 2026-11-23). Neither Windows nor Linux container results establish
native desktop/IME/monitor behavior; cross-compilation does not execute a target
GUI either. No Windows-container engine is exposed/verified for this task, and no
mode switch, installation, host probing or additional tests are requested.

When native validation is scheduled, use Linux/Windows/macOS jobs plus suitable desktop acceptance hosts. Their absence is currently deferred under the scope above; no host provisioning is required to continue implementation. Run security/review checks on the same reviewed changes. Keep runner/toolchain versions explicit, monitor updates and retain actual run metadata.

Use EC2 or other dedicated/self-hosted machines when persistent reproductions, remote desktop sessions, controlled images or special hardware are needed. EC2 Mac uses Dedicated Hosts with a minimum 24-hour allocation; budget/availability must be reviewed before provisioning. Cloud VMs do not substitute for every physical-monitor/GPU/accessibility test. This plan provisions nothing.

## Isolation and fixtures

Each test run owns its application data root, core session/state/socket, synthetic projects and helper processes. Preserve the normal user HOME for the existing IDE; do not invoke its helpers with altered homes. Do not reuse production account stores, tokens, cores, agent terminals, listeners or layouts. Teardown only owned resources and confirm it cannot match the real IDE or the calling shell.

Use provider fixture recordings with credentials/content removed, plus explicitly authorized live tests in independent test profiles. Capability tests must prove all advertised role bindings; a terminal launcher is not full role integration. Image-input tests must prove the selected provider/model received the intended attachment, or a precise unsupported result appears before sending.

## Required regression families

- Main versus guardian/subagent logs; large/truncated/rotated transcripts; LF/CRLF/CR; exact visible-view readiness.
- Quiet event loops, cancellation/abandonment, bounded subscriptions, stale callbacks, multi-window ownership and all close orders.
- Accounts and role inheritance across delayed launch, resume, fork, reviewer recovery and profile changes.
- Approval identity/scope/expiry, exact versus prefix semantics, raw locked persistence and unknown native outcomes.
- Work-item-to-Feature deduplication, task/worktree/review ownership, pause/concurrency and restart reconciliation.
- Native image clipboard, MIME/size/decoder limits, draft persistence, unsupported providers, cleanup and remote delivery.
- Multi-repository identity, selected-repository staging/commit, late responses and independent drafts.
- Versioned import/rollback, unchanged legacy stores, schema failures, interrupted writes and keychain/secret boundaries.

## Plugin and messaging fixtures

Use small third-party-style plugins to prove registration without central code edits, incompatible manifests, denied/revoked grants, malformed/oversized events, crashes, stalled shutdown and update failures. Verify the actual supported sandbox; process separation alone proves no filesystem/network restriction. Test plugin panes through every detach/re-dock/close order and confirm disabling one leaves shared agents and other plugins running.

For Blueprint, exercise source mapping, selection/feedback targeting, stale document updates and project/worktree separation. For the selected messaging transport, use local protocol fakes for duplicate/out-of-order updates, sender/conversation mismatches, echo loops, retry/rate-limit behavior, media limits and unknown send outcomes. Never blindly retry a consequential action when delivery or execution is uncertain. Live tests require separate test bot/account credentials and explicitly selected recipients; mocks do not establish live provider support.

## Official references checked during planning

- Docker kernel/container model: https://docs.docker.com/learn/content/what-is-a-container/
- GitHub OS runners: https://docs.github.com/en/actions/concepts/runners/github-hosted-runners
- EC2 Mac constraints: https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ec2-mac-instances.html
- Iced platform support: https://github.com/iced-rs/iced
- Slint platform evaluation: https://slint.dev/
- GPUI Kit first-candidate evaluation: https://github.com/longbridge/gpui-kit
- GPUI platform evaluation: https://github.com/zed-industries/zed/tree/main/crates/gpui
- Qt platform evaluation: https://doc.qt.io/qt-6/supported-platforms.html

These planning links alone are not evidence for selection. The conditional native GPUI Kit decision uses the exact pins and primary source paths in [ADR 0001](adr/0001-native-ui-toolkit.md) and the [portability matrix](NATIVE_UI_PORTABILITY.md). The [provenance inventory](PROVENANCE.md) retains pin/license gates for changes and distribution. Terminal, rich documents, accessibility, platform APIs and multi-window native verification remain required before support claims.
