# Yurherdr

A planned standalone native development workspace for coding agents, powered by [herdr-core](https://github.com/herdrdev/herdr-core).

Yurherdr will own its desktop interface and workflow while herdr-core runs agent terminals, messaging and machine connections. The target platforms are **Windows, macOS and Linux**. Rust is the preferred application-core language; Rust + [GPUI Kit](https://github.com/longbridge/gpui-kit) is the first UI prototype candidate, with toolkit selection gated on cross-platform evidence.

**Status: planning and documentation only. No application implementation or runnable native build is included yet.** The current Herdr/Shepherdr-based IDE remains a separate installation.

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
- [Required review policy](docs/REVIEW_POLICY.md)
- [Cross-platform testing strategy](docs/TEST_STRATEGY.md)
- [Contributor and agent rules](AGENTS.md)

The plan separates existing behavior to preserve, known limitations to resolve and requested new features. A checked planning task does not mean its application feature is implemented.

## Naming and development isolation

The planning namespace is **Yurherdr / yurherdr**, matching this repository. The request also used the spelling “Yurherder”; confirm that spelling in YUR-007 before implementation. New app identifiers and storage will use the confirmed new name. Legacy names are reserved for attribution and explicit import/compatibility mappings; external herdr-core and hc names remain unchanged.

Development will use a separate executable, app identity, state directories and development core/session. It must remain independent of the current IDE and its live agents, data and services.

## Origins

The feature baseline is the existing [yurtools/shepherdr](https://github.com/yurtools/shepherdr) workspace, derived from Elias Stravik’s [Shepherdr](https://github.com/eliasstravik/shepherdr) 3.1.35, including the fork’s documented extensions. This is a new application project, not a claim to have authored upstream work. License and notice preservation is required before reusing source, assets or fixtures; see YUR-004.
