# Cross-platform testing strategy

Target Windows, macOS and Linux from the first prototype. Declare exact supported OS versions, architectures and provider versions only after evidence; target coverage is not current certification.

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

Docker is useful for pinned Linux build/integration dependencies and multiple Linux distributions. Containers share a kernel; a Linux container does not provide a Windows or macOS native desktop. Docker Desktop on those hosts runs Linux containers in a Linux VM. Cross-compiling alone also does not execute a target GUI.

Start with GitHub Actions Linux/Windows/macOS jobs plus suitable desktop acceptance hosts. Run security/review checks on the same reviewed changes. Keep runner/toolchain versions explicit, monitor updates and retain actual run metadata.

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

Toolkit links are candidates, not a selection. The [provenance inventory](PROVENANCE.md) assigns the later pin/license gates for toolkit, core and test-infrastructure sources. Evaluate terminal embedding, rich documents, accessibility, licensing, platform APIs and multi-window support against YUR-005 before an architecture decision.
