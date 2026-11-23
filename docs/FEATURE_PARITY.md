# Rewrite feature parity inventory

All rows are **planned for the new native app**. This is an inventory of the current workspace and a coverage map, not a statement that the rewrite implements them. Existing limitations remain explicit until a task proves expanded support.

Baseline evidence: the existing repository’s lib/features.json, lib/addons.json, README.md and docs/workspace.md, settings.md, approvals.md, addons.md, install.md, web.md and insights.md. The inventory references the 3.1.35-derived legacy workspace at `9d92deccc01d1328a656e5a712a6732a3f4715aa`; newer upstream 3.2.x behavior is not silently substituted. See the [verified baseline distinctions and reuse gates](PROVENANCE.md). This pins the inventory’s source, not an executed parity or implementation audit.

## Registered baseline features

Every registered feature ID appears exactly once below. Add-on switches and dependencies are additionally covered by YUR-043. Registration does not certify equal native/web behavior or cross-platform runtime support.

| Baseline ID | Existing capability | Rewrite tasks |
| --- | --- | --- |
| spaces | Spaces sidebar | YUR-012 |
| space-actions | Space actions | YUR-012 |
| new-space | New space | YUR-012 |
| tabs | Tabs in a space | YUR-012 |
| panes | Panes and pane header | YUR-012, YUR-013 |
| status | Status bar | YUR-038 |
| notifications | Notifications | YUR-038 |
| menus | Menus and dialogs | YUR-012, YUR-038 |
| settings | Shepherdr settings | YUR-027 |
| frame | App frame | YUR-012 |
| theme | Theme | YUR-012 |
| terminals | Live terminals | YUR-014 |
| tasks-list | Tasks list | YUR-024 |
| task-view | Task view | YUR-024 |
| accept | Accept card | YUR-029 |
| space-files | Files side panel | YUR-031, YUR-032 |
| space-changes | Changes side panel | YUR-034 |
| task-pr | Pull request | YUR-029 |
| task-artifacts | A task's Artifacts | YUR-035 |
| task-menu | Pin from a row | YUR-024 |
| assistant | Assistant per space | YUR-025 |
| news | Task news | YUR-025, YUR-038 |
| timeline | Chat timeline | YUR-020, YUR-021 |
| composer | Composer | YUR-022 |
| message-actions | Message actions | YUR-021, YUR-023, YUR-038 |
| attachments | Attachments and images | YUR-022 |
| models | Model menu | YUR-015, YUR-019 |
| subagents | Subagents above the box | YUR-021, YUR-025 |
| background-jobs | Background shells above the box | YUR-021 |
| find | Find in chat | YUR-021 |
| cards | Cards in the chat | YUR-028, YUR-030 |
| secrets | Secret cards and vault | YUR-030 |
| jump | Jump | YUR-038 |
| keys | Keys list | YUR-038 |
| artifacts | Artifacts list and viewer | YUR-035 |
| markup | Mark up and answers | YUR-035 |
| dictation | Dictation | YUR-039 |
| call | Voice call | YUR-039 |
| reactions | Reactions | YUR-038 |
| browser | Live browser | YUR-037 |
| autopilot | Autopilot | YUR-040 |
| machines | Machines | YUR-041 |
| web | Web on/off, link, QR | YUR-042 |
| serve-from | Serve from: the web app from any linked machine | YUR-042 |
| insights | Usage insights: a private local log of how the person uses Shepherdr, for their own agents' tips | YUR-043 |

## Fork extensions, semantics and known limits

| Capability | Baseline scope / acceptance boundary | Rewrite tasks |
| --- | --- | --- |
| Named Feature Assistants and persistent worker ownership | Native extension; worktree isolation remains a task setting. | YUR-025 |
| Named Codex accounts and account-preserving launch/resume/fork/restart | Codex-only named profiles in the baseline; new provider support needs capability evidence. | YUR-019, YUR-023 |
| Model/thinking/permissions controls, skills and commands | Full current Chat/role integration covers Codex and Claude Code. | YUR-015, YUR-016, YUR-019 |
| Per-role space instructions appended to global prompts | Fresh starts use additions; resumed contexts are retained. | YUR-019, YUR-027 |
| Persistent space-rooted Files, project-folder selection and references | Native project tree and separate document panes. | YUR-012, YUR-031 |
| Document drafts, external changes, per-file tabs and text editing | Bounded files; explicit save/reload/conflict behavior. | YUR-032 |
| Markdown and CSV preview, source highlighting | Preserve existing language coverage and preview limits; fix demonstrated rendering regressions. | YUR-021, YUR-032 |
| Bounded DOCX and Excel OOXML content | Read-only content, not full Office layout, macros or formula calculation. | YUR-033 |
| Multiple repositories, worktrees/submodules and selected-repository commits | Native supports multiple repositories; current web panel is single-repository. | YUR-034, YUR-042 |
| Space-scoped closed conversations and previews | Local Codex history across configured accounts; broader remote/provider history is limited. | YUR-020, YUR-023 |
| Approval rules, delegated Feature review and native handoff history | Account/folder/scope/expiry binding; unknown native outcomes remain unknown. | YUR-028 |
| Persistent browser pages, web-chat shortcuts and profile isolation | Native extension; website account is separate from CLI accounts. | YUR-036 |
| Configurable launchers and clipboard/context actions | A launcher alone is not structured agent-role integration. | YUR-018, YUR-036, YUR-038 |
| Wide/comfortable Chat, Beside and extra views | All views retain the original session; no duplicate launch or accidental stop. | YUR-013, YUR-021, YUR-023 |
| Inherited resource/concurrency queues, archiving and branch/worktree settings | Restore operational controls through the canonical task engine. | YUR-024, YUR-027 |
| Remote secret ownership and private web hosting/Serve from | Explicit linked targets and scoped delivery; no implicit public exposure. | YUR-030, YUR-041, YUR-042 |
| Reliable native reader/render lifecycle and bounded event loops | Known unresolved visible Chat stalls and prior quiet-loop retention are regression fixtures, not behavior to preserve. | YUR-010, YUR-020, YUR-021, YUR-045 |

## Requested enhancements

| Enhancement | Rewrite tasks | Acceptance emphasis |
| --- | --- | --- |
| Default Kimi, Goose and OpenCode role integrations | YUR-017 | All role bindings tested with real adapters; no terminal-only substitution. |
| Extensible additional agents | YUR-015, YUR-018 | Registry/adapter extension without central UI brand switches. |
| Space work list creates one Feature per selected work item | YUR-026 | Explicit batch start, durable exactly-once mapping, dependencies/concurrency and policy-bound completion. |
| Paste clipboard images into Chat | YUR-022 | Native preview/draft/send on all three OSes, with provider/model capability checks. |
| Detach any pane to another monitor and re-dock | YUR-013 | Every pane type, stable sessions, mixed DPI/focus and all window-close orders. |
| Independently installed plugins and public extension SDK | YUR-048, YUR-049, YUR-050 | Versioned contracts, scoped capabilities, honest sandbox claims, lifecycle/rollback and plugin pane detachment. |
| Yurherdr Blueprint rewrite as a reference plugin | YUR-051 | Separate optional plugin; pin the TUI plugin and desktop mod references before porting. |
| Telegram or Signal messaging as a reference plugin | YUR-052 | Select first transport explicitly; opt-in notifications/replies/media, paired identities, scoped routing and delivery recovery. |
| Standalone Windows/macOS/Linux native client | YUR-005, YUR-008, YUR-009, YUR-045, YUR-046 | Independent app/core test state and native validation on every declared platform. |

The Blueprint and messaging plugins are new rewrite work, not bundled baseline capabilities. Existing Herdr TUI plugins and Herdr App mods need an explicit port or compatibility assessment; their manifests do not establish Yurherdr compatibility. See [the plugin plan](PLUGINS.md).

## Parity sign-off

For each row, record implementation commit, native/web/provider/platform scope, positive and negative tests, observed UI proof and unresolved limitations before marking it complete. A capability may be deferred only by an explicit scope decision; it must not vanish from the inventory.

Retain exact session/account/repository identity, explicit permission boundaries, user acceptance semantics and non-destructive data migration across the rewrite. Improved rendering and additional providers must not silently weaken those guarantees. Preserve the source language list in the document task’s detailed implementation inventory, including HTML, CSS, TypeScript, JavaScript, Java, SQL, Bash, XML, JSON, YAML, Helm, Liquibase, Python and Terraform.
