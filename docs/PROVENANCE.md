# Source provenance and reuse gates

YUR-004 · Feature `71116fa009c34f8cacd62f2992e4c6bd` · verified 2026-11-23.

The YUR-004 baseline was documentation-only and adopted no application dependencies. YUR-005 adds an experimental dependency graph under `prototypes/native-ui/`. YURH-4 records a provisional choice of that exact Rust/native GPUI Kit path under the owner's 2026-11-23 testing deferral; it changes no dependency, pin, notice or implementation. A source reference establishes origin, not ownership, compatibility, legal clearance or native platform support. Review-skill access and hashes are in the [review policy](REVIEW_POLICY.md).

## Frozen legacy baseline

Read-only Git inspection of the locally available legacy checkout established the following. Remote-tracking refs are local observations, not claims about current remote tips. No fetch, checkout, install or mutation was performed in that checkout.

| Role | Immutable object / observed ref | Meaning |
| --- | --- | --- |
| Installed-workspace baseline supplied by the brief | [yurtools/shepherdr commit `9d92deccc01d1328a656e5a712a6732a3f4715aa`](https://github.com/yurtools/shepherdr/commit/9d92deccc01d1328a656e5a712a6732a3f4715aa) | Canonical feature/source inventory baseline. Commit exists locally; tree `6f7429620e5b2cf7b7e9a00719b4720200b95037`; local `origin/main` and the diagnostic baseline branch resolve here. This does not prove every installed runtime byte matches Git. |
| Original upstream release | [eliasstravik/shepherdr commit `abd5a0f1aa6faf2d0ee8d6559dc7f064fdae488e`](https://github.com/eliasstravik/shepherdr/commit/abd5a0f1aa6faf2d0ee8d6559dc7f064fdae488e) | Local annotated tag `v3.1.35` (tag object `5bea5e83a21c431afcc4437a8b1ccdea7691bc70`) peels here. This commit is an ancestor of the fork baseline (`merge-base --is-ancestor` exit 0). Tag signature authenticity was not verified. |
| Upstream tracking snapshot | Local `upstream/main` resolves to `abd5a0f1aa6faf2d0ee8d6559dc7f064fdae488e` | Configured upstream is `https://github.com/eliasstravik/shepherdr.git`; fork origin is `https://github.com/yurtools/shepherdr.git`. No substitution of current upstream `main` or later releases is authorized. |
| Other committed local work | Legacy HEAD observed at `3b6ff2eac4a343ffbb3b122ec56bb08b91b2fbef` | Not the baseline and not a descendant of it (ancestry exit 1); common ancestor `5fc13f4858ae4b3837dfe2f168476f910db4e161`. Excluded. Any proposed port from another commit needs its own approved source/diff/license record. |
| Uncommitted local work | A modified tracked file was present | Excluded; contents were not read or copied. A dirty checkout is never an immutable source. |

The fork baseline’s `package.json` identifies version 3.1.35; that version label does not make the fork identical to the upstream release. The pinned `lib/features.json` and `lib/addons.json` differ from the upstream release blobs. The [parity inventory](FEATURE_PARITY.md) uses the fork’s committed evidence, not unrelated pending work or newer upstream behavior.

### Reproduce local verification without touching the legacy worktree

Set `LEGACY_REPO` to an existing authorized checkout. These commands read objects and refs only; do not fetch into it, reset it, run its helpers, or copy its working tree. Use `GIT_OPTIONAL_LOCKS=0` for status inspection. Ref observations can change; compare exact objects rather than assuming the table’s mutable refs remain current.

```sh
export GIT_OPTIONAL_LOCKS=0
LEGACY_REPO=/path/to/read-only-legacy-checkout
BASELINE=9d92deccc01d1328a656e5a712a6732a3f4715aa
UPSTREAM=abd5a0f1aa6faf2d0ee8d6559dc7f064fdae488e
git -C "$LEGACY_REPO" cat-file -t "$BASELINE"
git -C "$LEGACY_REPO" rev-parse "$BASELINE^{tree}"
git -C "$LEGACY_REPO" rev-parse v3.1.35 'v3.1.35^{commit}'
git -C "$LEGACY_REPO" merge-base --is-ancestor "$UPSTREAM" "$BASELINE"
git -C "$LEGACY_REPO" rev-parse HEAD refs/remotes/origin/main refs/remotes/upstream/main
git -C "$LEGACY_REPO" status --porcelain=v1 --untracked-files=all
git -C "$LEGACY_REPO" ls-tree -r --name-only "$BASELINE"
git -C "$LEGACY_REPO" show "$BASELINE:LICENSE"
git -C "$LEGACY_REPO" show "$BASELINE:LICENSE-herdr"
git -C "$LEGACY_REPO" show "$BASELINE:NOTICE"
```

Record each exit code separately; a pipeline’s final exit alone does not prove `git show` succeeded. For byte verification, save each successful `git show REV:PATH` to a disposable file and run SHA-256 on that file. Windows users can use Git Bash plus `sha256sum`, or PowerShell `Get-FileHash -Algorithm SHA256` on an exact binary download/export; avoid text re-encoding and newline conversion.

### Verified license and notice evidence

Hashes below are SHA-256 of exact `git show` blob bytes at the fork baseline. Root LICENSE, LICENSE-herdr and NOTICE have identical bytes at the upstream release. The upstream LICENSE and fork NOTICE were also fetched from their exact raw GitHub commit URLs and matched the local bytes.

| Pinned path | Bytes | SHA-256 |
| --- | ---: | --- |
| [LICENSE](https://github.com/yurtools/shepherdr/blob/9d92deccc01d1328a656e5a712a6732a3f4715aa/LICENSE) | 1070 | `34d04d3fd8fa65074d86e9dbfa2717d9c8d0237b1218cde0ece5d80eeae6fe2d` |
| [LICENSE-herdr](https://github.com/yurtools/shepherdr/blob/9d92deccc01d1328a656e5a712a6732a3f4715aa/LICENSE-herdr) | 11357 | `c71d239df91726fc519c6eb72d318ec65820627232b2f796219e87dcf35d0ab4` |
| [NOTICE](https://github.com/yurtools/shepherdr/blob/9d92deccc01d1328a656e5a712a6732a3f4715aa/NOTICE) | 498 | `25438e0542a4334edbaa1065b0f5e6cd8b862ee933e74a646e133f9175e3ffbc` |
| [Geist OFL.txt](https://github.com/yurtools/shepherdr/blob/9d92deccc01d1328a656e5a712a6732a3f4715aa/patches/herdr-app/0.1.16/fonts/OFL.txt) | 4382 | `e5e56b793839774266243b8dd89c618f2962f098a2076757a5f2839c27eaaabd` |
| [LICENSE-LobeHub.txt](https://github.com/yurtools/shepherdr/blob/9d92deccc01d1328a656e5a712a6732a3f4715aa/web/public/agents/LICENSE-LobeHub.txt) | 1064 | `add9d7531d1b21646317a8958e38fc727506fa39d24bdecb44154d943c82753a` |

Responsibilities below reflect the inspected texts; they are not a file-by-file license clearance or a declaration of Yurherdr’s own license.

| Material / original attribution | Observed terms and gate before any reuse |
| --- | --- |
| Shepherdr source and documentation; Copyright (c) 2026 Elias Stravik | Root MIT license requires retention of its copyright and permission notice in copies/substantial portions. Preserve upstream authorship, headers and disclaimers; record fork modifications separately. A repository-level MIT label does not override nested terms. |
| Herdr-derived `mods/machine-add-chat` and `mods/toasts` | NOTICE identifies these as derived from Herdr App mods and retained under Apache-2.0 (`LICENSE-herdr`). Map exact reused files/headers to their original copyright holders; the inspected root NOTICE is not a complete per-file attribution list. Preserve applicable notices, provide the license and mark modified files as required by the license. Inspect any further Herdr/runtime patch origins separately. |
| Lucide icons in mods | Root NOTICE identifies Lucide and ISC, but does not provide the complete icon provenance/ISC text. Owner of the importing UI task must locate exact icon versions, original copyright and license before copying. Keep required notices; do not assume all icons have the same source. |
| Agent SVG assets; Copyright (c) 2023 LobeHub | Nested MIT license requires its own copyright/permission notice. Check each intended asset and any brand/trademark constraints; root Shepherdr attribution alone is insufficient. |
| Geist font files; Copyright 2024 The Geist Project Authors | Nested SIL OFL 1.1 requires accompanying copyright/license, retains OFL for font derivatives and limits standalone sales and modified reserved names. Confirm the exact font’s metadata and any reserved names before reuse. |
| `service/vendor/qrcode.cjs`; Copyright (c) 2009 Kazuhiko Arase | Inspected inline header identifies MIT and a QR Code trademark notice for DENSO WAVE INCORPORATED. Preserve the header and obtain the applicable full license text and exact upstream version before reuse; do not treat the enclosing project’s MIT notice as the vendor’s notice. |
| Vendored code, runtime patches, dependency packages, fixtures, screenshots/media and generated files | Inventory exact files, inline notices, origin revisions and dependency/license chain in the owning task. Root license filenames alone do not find inline or transitive obligations. Synthetic test content still needs provenance for borrowed code/data; never copy live conversations/accounts. Unknown provenance blocks copying until resolved or the material is replaced. |

## Proposed sources and later owners

The YUR-005 **experimental** pins, exact archive digests, attribution and retained
terms are now in [the prototype notice](../prototypes/native-ui/NOTICE.md), with
the locked target-specific [dependency inventory](../prototypes/native-ui/DEPENDENCIES.csv)
and [conditional decision ADR](adr/0001-native-ui-toolkit.md). This includes native Kit/GPUI,
independent PTY/parser and Linux clipboard fixtures, plus a separate independently
built core/Ghostty/local Zig tool. No JavaScript host, Zed terminal crate or Qt
dependency was adopted. Raw source/review/runtime evidence remains ignored.

The [portability audit](NATIVE_UI_PORTABILITY.md) distinguishes source/manifest
support, Linux-host target resolution and actual execution. YURH-4 reverified
the cached Kit/core/Ghostty archives and native GPUI-family/PTY/parser/Wayland
crate bytes before relying on extracted source, compared retained notices and
repeated the locked target queries. The 614 package/version entries are a native
graph inventory, not three builds or distribution clearance. Root license labels
and root-notice hashes do not clear nested files, native libraries, shader tools,
fonts or the optional JavaScript runtime. The decision grants no unrestricted
redistribution right; resolve those obligations before shipping.

Sources below without an explicit experimental/conditional decision remain **unselected and unpinned here**. These discovery URLs may move. Before evaluation/adoption, the owner must resolve the exact source repository/package, full commit or immutable artifact plus digest, version, license/notice files (including nested/transitive material), original attribution, intended files/use and platform evidence at the applicable gate. An evaluation pin does not authorize shipping it. Record rejection or selection in that task’s evidence/ADR; update this inventory when a dependency is actually adopted.

| Source category / discovery references | Later owner and gate |
| --- | --- |
| [herdr-core](https://github.com/herdrdev/herdr-core), including hc / HC_SESSION protocol identifiers | YUR-010 pins the core/protocol version and any client library/generated schema source before client implementation; YUR-005/YUR-008 pin any core used for an isolated prototype/fixture run first. Review compatibility and licenses; do not rename external protocol identifiers. |
| [GPUI Kit](https://github.com/longbridge/gpui-kit), [GPUI in Zed](https://github.com/zed-industries/zed/tree/main/crates/gpui), native component path and separately optional JavaScript extension host | YUR-005 provisionally selects the exact native Kit/GPUI pins in the notice/ADR, with Windows/macOS/Linux retained targets and deferred native checks. Resolve any changed dependency's exact source/terms first. The JavaScript host is excluded and unselected; a Zed repository license is not automatically every crate's license. |
| [Iced](https://github.com/iced-rs/iced), [Slint](https://slint.dev/), [Qt](https://doc.qt.io/qt-6/supported-platforms.html) | Compare only against an evidenced blocker under ADR 0001. Pin the sources/packages actually compared, versions/editions and license options first. The notice records a narrow pinned Qt source comparison only. Unavailable hosts alone do not require an alternative; website support claims do not pass native validation. |
| [herdr-blueprint](https://github.com/chappse6/herdr-blueprint) (TUI plugin) and [herdr-blueprint-app](https://github.com/chappse6/herdr-blueprint-app) (desktop mod) | YUR-051 pins BOTH distinct sources and their license/notice chains before a port inventory. YUR-048–050 own host/SDK/runtime dependencies. Assess each reference independently; no unchanged plugin compatibility or privileged central-app shortcut is implied. |
| [Telegram Bot API](https://core.telegram.org/bots/api), [signal-cli](https://github.com/AsamK/signal-cli) and its [JSON-RPC interface](https://github.com/AsamK/signal-cli/blob/master/man/signal-cli-jsonrpc.5.adoc) | YUR-052 chooses the first transport, pins client/protocol/runtime sources and captures documentation/version dates and provider terms before implementation. signal-cli is unofficial. Keep credentials and live accounts out of fixtures. |
| Codex, Claude Code, Kimi, Goose, OpenCode and additional provider adapters/launchers | YUR-015–019 pin actual CLI/SDK/protocol/fixture sources and versions before integration. Names in README are capability goals, not adopted packages or redistribution permission. YUR-039 owns any voice/model/service sources. |
| Terminal, browser, document/Office/Markdown/code/media parsers, clipboard/OS services, icons/fonts, storage and import/migration sources | YUR-014, YUR-021–022, YUR-031–039, YUR-044 and the relevant implementing tasks pin their chosen libraries/assets/fixtures before reuse. Legacy dependency manifests are evidence to assess, not a dependency list inherited by the native app. |
| [Docker](https://docs.docker.com/learn/content/what-is-a-container/), [GitHub runners](https://docs.github.com/en/actions/concepts/runners/github-hosted-runners), [EC2 Mac](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ec2-mac-instances.html) and other CI/test images/tools | YUR-009 pins actual actions/toolchains/images (commit/digest, OS/architecture, licenses) and captures the documentation access date/version. YUR-045–046 own later acceptance/package evidence. These are testing proposals, not provisioned services; cloud expense still needs authorization. |

This covers the source categories referenced by README, the [test strategy](TEST_STRATEGY.md), [plugin plan](PLUGINS.md) and [parity inventory](FEATURE_PARITY.md). It does not assert that unavailable package licenses were inspected. New source categories must receive an owner and a pre-use gate before they enter an implementation.

## Reuse record required in the owning change

For each reused file/asset/fixture or dependency, record source URL, exact revision/subpath or artifact digest, upstream author/copyright, applicable license text and NOTICE/header locations, destination, modifications and required distribution notices. Retain original identifiers in attribution and explicit compatibility fixtures; new implementation uses the confirmed Yurherdr namespace (spelling confirmed 2026-11-23; YUR-007 platform/data-location acceptance remains open). Keep dependencies’ identifiers intact.

Resolve unknown license scope, missing notices and conflicting terms before copying or shipping; record the responsible owner’s decision and evidence. Do not infer authorization from public access, a manifest label or this inventory. Preserve required notices alongside any future reused material and include them in source/package attribution; do not silently vendor skill bodies. The full roadmap and local verification/proof assets stay under ignored `local/`, outside commits. YUR-004 verifies documentation, source bytes and workflow access only; no executed implementation audit, runtime parity or three-platform GUI proof is claimed.
