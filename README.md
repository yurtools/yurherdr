# Yurherdr

A planned standalone native development workspace for coding agents, powered by [herdr-core](https://github.com/herdrdev/herdr-core).

Yurherdr will own its desktop interface and workflow while herdr-core runs agent terminals, messaging and machine connections. The target platforms are **Windows, macOS and Linux**. Rust is the preferred application-core language; Rust + [GPUI Kit](https://github.com/longbridge/gpui-kit) is the first UI prototype candidate, with toolkit selection gated on cross-platform evidence.

**Status: planned product with an experimental, guarded [native evaluation probe](prototypes/native-ui/README.md).** The probe is a YUR-005 experiment, not a supported client. Toolkit selection and three-platform acceptance remain open. The current Herdr/Shepherdr-based IDE remains a separate installation.

## Planned experience

- Work with agents through Chat and the real terminal, with durable spaces, sessions, tasks, reviews, accounts and approvals.
- Use Codex, Claude Code, Kimi, Goose and OpenCode through an extensible adapter registry; add other agents without editing central UI logic.
- Give a space Assistant a work list: it creates one persistent Feature per selected item and coordinates the work under the configured policies.
- Paste images into Chat with a preview and capability-aware delivery.
- Detach any pane into its own window, move it across monitors and re-dock without restarting agents or losing state.
- Extend the workspace through independently installed plugins, including a planned Yurherdr Blueprint rewrite and Telegram or Signal messaging integration.
- Retain files, documents, multi-repository Changes, artifacts, browsers, voice, automation, linked machines and a web/mobile companion as tracked parity work.

## Rewrite plan

- The ordered task list and acceptance criteria are kept locally in `local/REWRITE_PLAN.md`.
- [Current-feature parity inventory](docs/FEATURE_PARITY.md)
- [Plugin system and reference integrations](docs/PLUGINS.md)
- [Source provenance and dependency inventory](docs/PROVENANCE.md)
- [Required review policy](docs/REVIEW_POLICY.md)
- [Cross-platform testing strategy](docs/TEST_STRATEGY.md)
- [Contributor and agent rules](AGENTS.md)

The plan separates existing behavior to preserve, known limitations to resolve and requested new features. A checked planning task does not mean its application feature is implemented.

## Naming and development isolation

The person confirmed **Yurherdr / yurherdr** in the YUR-005 Feature conversation on 2026-11-23. This confirms spelling for the probe; it does not complete YUR-007 platform/data-location acceptance or YUR-008 environments. Legacy names are reserved for attribution and explicit import/compatibility mappings; external herdr-core, hc and HC_SESSION names remain unchanged.

Development will use a separate executable, app identity, state directories and development core/session. It must remain independent of the current IDE and its live agents, data and services.

## Origins

Evaluation pins are experimental, not product adoption decisions. [ADR 0001](docs/adr/0001-native-ui-toolkit.md) remains Proposed.

The feature baseline is the [yurtools/shepherdr](https://github.com/yurtools/shepherdr) workspace at `9d92deccc01d1328a656e5a712a6732a3f4715aa`, derived from Elias Stravik’s [Shepherdr](https://github.com/eliasstravik/shepherdr) 3.1.35, including the fork’s documented extensions. This is a new application project, not a claim to have authored upstream work. The [provenance inventory](docs/PROVENANCE.md) distinguishes that baseline from the upstream release, other local refs and pending changes, and records license/notice gates before reuse. Toolkit and dependency adoption remain future work.
