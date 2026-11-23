# Plugin system plan

Status: requested architecture and backlog only; no SDK, plugin host or integration is implemented. Track implementation in YUR-048 through YUR-052 in the checkout’s local task list (`local/REWRITE_PLAN.md`).

## Purpose and extension surfaces

Allow users and third-party authors to add capabilities without rebuilding Yurherdr or adding brand-specific branches to the central UI. Built-in optional feature switches remain distinct from independent plugin packages. Reference integrations must use the same supported extension surface as other plugins.

| Surface | Planned contract |
| --- | --- |
| Commands and events | Namespaced actions and typed, versioned subscriptions with cancellation and backpressure |
| Agent adapters | Register provider capabilities and supported roles through the existing adapter contract |
| Panes and views | Host-managed layout, focus, themes, accessibility and detach/re-dock; plugins own their view state |
| Domain operations | Scoped space/Feature/task queries and actions through existing permission and acceptance policy |
| Background integrations | Explicit lifecycle, selected destinations, bounded jobs and no dependency on a pane staying open |
| Configuration and data | Validated settings, per-plugin state/migrations and scoped secret references |

YUR-048 records an ADR before choosing the executable format and rendering model. Evaluate a versioned process protocol or constrained runtime against real plugin needs; do not assume all plugins must be Rust, JavaScript or part of the app binary. Keep native rendering behind a host-owned contract; do not expose arbitrary toolkit pointers or promise a stable Rust dynamic-library ABI. Plugin packaging must declare host/protocol ranges and supported OS/architectures.

## Trust and lifecycle

YUR-049 must distinguish crash containment from security confinement. A same-user helper process can access the host outside its API unless a real sandbox prevents it. Define and test restrictions for the supported runtime/platforms; executable helpers without confinement are explicitly full-trust installations. Denying a host capability must also deny all equivalent host entry points.

YUR-050 covers local development links, pinned repository/package installation, discovery, explicit update, enable/disable, rollback and removal. Show the resolved source/revision, compatibility, permissions and commands/hooks before execution. Discovery only reads metadata. Newly requested grants need approval; disabled plugins cannot retain host access, event handlers or owned workers. Shared agents remain alive. Removing data or credentials is a separate explicit choice.

Use configured sources and dependency/version declarations rather than hardcoded plugin names, service URLs or transport limits. A catalog listing is not a security review. This scope does not publish a marketplace, install old Herdr packages or enable network integrations automatically.

## Reference integrations

**YUR-051: Yurherdr Blueprint.** Pin and assess both [herdr-blueprint](https://github.com/chappse6/herdr-blueprint), the terminal plugin, and [herdr-blueprint-app](https://github.com/chappse6/herdr-blueprint-app), the desktop mod, before defining the port inventory. They are distinct products. The new plugin should provide agent-linked diagrams/documents and targeted feedback while using native Yurherdr panes and the shared agent/domain contracts. Preserve provenance; assess platform/provider differences individually. Packaging, state and new commands use the Yurherdr namespace.

**YUR-052: Messaging.** Select Telegram or Signal for the first independently installed channel plugin; record the alternative as a follow-up. Notifications, replies and attachments are opt-in and bound to selected spaces/Features and paired sender/conversation identities. Incoming messages are external input, never trusted instructions that bypass normal task, secret or approval policy. Sensitive actions retain the existing authorization requirements and replay protection; outbound content is scoped explicitly.

[Telegram’s Bot API](https://core.telegram.org/bots/api) supports long polling or webhooks; select one inbound mode and handle retries/deduplication. If webhooks are used, validate the configured webhook secret. A Signal prototype may evaluate [signal-cli](https://github.com/AsamK/signal-cli), an unofficial project with a [JSON-RPC interface](https://github.com/AsamK/signal-cli/blob/master/man/signal-cli-jsonrpc.5.adoc); do not describe it as an official Signal bot API. Assess linking, identity changes, dependency maintenance and each target OS before claiming support. Credentials belong in scoped secret storage; logs must not leak message content or tokens.

Neither integration may depend on private central-app shortcuts. Success means a separate package can be installed, configured, exercised, disabled and removed through public APIs, with real Windows/macOS/Linux evidence and declared provider limits. [Provenance](PROVENANCE.md), [testing](TEST_STRATEGY.md) and [review](REVIEW_POLICY.md) gates apply to the host, SDK and both reference plugin types.
