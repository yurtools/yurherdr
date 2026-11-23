# Yurherdr coding and interface rules

All contributors, Feature Assistants, workers and reviewers use this document with [AGENTS.md](../AGENTS.md), the [review policy](REVIEW_POLICY.md) and the [test strategy](TEST_STRATEGY.md). Put the roadmap ID and acceptance criteria in each brief so a worker can work from its own checkout.

## Code and architecture

- Keep one owner for each behavior. Domain code owns state and policy, adapters translate external protocols, and the UI presents state and dispatches commands. Keep file and network I/O, process launch and expensive parsing off the render thread.
- Use explicit types for identities, requests, state transitions and errors. Use existing contracts before adding a new abstraction. Introduce interfaces where there is a real boundary, such as another provider, an OS service or a versioned plugin API.
- Keep modules small enough to understand together. Write for the normal path and make failures explicit. Prefer a focused change and meaningful names over speculative configurability and hidden fallback behavior.
- Put platform and provider differences behind their owning adapters. Centralize defaults, limits and feature flags in validated configuration or named domain policy. Keep product terms and component variants consistent.
- Define ownership, cancellation and timeouts for asynchronous work. Bound queues, cache and event lifetimes. Make retries idempotent where they can be, and give uncertain operations an explicit state. Persist related changes atomically and preserve existing identities during migration.
- Scope operations to the selected space, repository, session, account and window. Validate external inputs, keep credentials in scoped storage, and preserve approval and acceptance checks through plugins and remote interfaces. Use structured command arguments and redact diagnostics.

## A consistent interface

- Use shared components and theme tokens for type, spacing, color, icons and interaction states. Start with the app shell’s established controls for panes, tabs, menus, dialogs, settings and notifications. Extend the shared component where a new feature needs a reusable variant.
- Give the same action the same name, icon, shortcut convention and effect wherever it appears. Use a clear primary action, put related actions together and show their target before a consequential change. Follow platform conventions for system dialogs, keyboard modifiers and accessibility APIs.
- Reuse patterns for loading, empty, offline, disabled, success and error states. Give a useful next action when one exists. Keep drafts, selection, scroll position and focus through refresh, resize, failure, detaching and re-docking.
- Support keyboard navigation, visible focus, accessible names and state, screen readers, text scaling, high contrast and reduced motion. Pair color with another cue. Verify labels, logical tab order and focus return from dialogs with realistic content.
- Use shared pane and window commands for moving, detaching, closing and restoring. Preserve the underlying agent or task when a view closes. Test normal and narrow panes, detached windows, light and dark themes and the relevant desktop scale factors.
- Show user intent and outcomes in the interface. Use concise labels and actionable errors. Keep process names and internal identifiers in diagnostics unless the user needs them to make a decision.

## Tests, review and handoff

- Start with the brief’s prerequisites and acceptance criteria. Preserve existing behavior across affected entry points. Use synthetic data and the isolated development core for tests. Legacy Herdr App mod installation, app checks and shared-copy sync instructions apply only to legacy mod work.
- For Rust changes, run the repository’s formatter, lint and scoped tests once configured. Cover normal workflow, failure and recovery. For integration boundaries, exercise real contracts with controlled fixtures. For a fix, reproduce the regression when feasible and verify it through the affected entry point.
- For visible changes, exercise the affected workflow, compare its shared controls and states with adjacent features, and record what was observed on each tested OS. Keep compile, headless and real desktop evidence distinct.
- Apply both reviews in the [review policy](REVIEW_POLICY.md). Herdr’s optional reviewer setting controls the worker workflow. The project’s review evidence applies to implementation even when that setting is off.
- Report scope, reviewed commit, tests with actual exit codes, observed UI and outstanding limits. Keep the complete roadmap and orchestration records in the checkout’s ignored `local/` files. Keep source, shared rules and reusable tests available in worker worktrees. Start dependents once their prerequisites are verified and accepted where required.
