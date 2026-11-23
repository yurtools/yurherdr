# Rewrite review policy

Every implementation change requires a **structural review** and a **separate correctness/security review**. These repository gates apply even when Herdr’s optional Review setting is off. That setting controls orchestration, not the evidence required here. This policy grants no permission to start agents, merge, publish, deploy or waive user acceptance.

## Pinned official review skills

Both skills were retrieved and read from official `cursor/plugins` revision **`7a583913eb1b8d1a0ee76753d1a2cf072e7b54f9`** on 2026-11-23. Use these immutable sources rather than `main`:

| Review record | Exact source | Bytes | SHA-256 of fetched bytes |
| --- | --- | ---: | --- |
| Structural | [thermos/skills/thermo-nuclear-code-quality-review/SKILL.md](https://github.com/cursor/plugins/blob/7a583913eb1b8d1a0ee76753d1a2cf072e7b54f9/thermos/skills/thermo-nuclear-code-quality-review/SKILL.md) | 12437 | `7faca08b51b643b2ddd0836f92af15574444024685dcc1e677dbbb39ae8c9e8f` |
| Correctness/security | [thermos/skills/thermo-nuclear-review/SKILL.md](https://github.com/cursor/plugins/blob/7a583913eb1b8d1a0ee76753d1a2cf072e7b54f9/thermos/skills/thermo-nuclear-review/SKILL.md) | 4547 | `5091011c4490932d40658ae958fb55b9aaca8e2bba5196860295ab945180e434` |
| Accompanying terms | [thermos/LICENSE](https://github.com/cursor/plugins/blob/7a583913eb1b8d1a0ee76753d1a2cf072e7b54f9/thermos/LICENSE) | 1063 | `702f5f331b56aff0e33d8c7826df5202559f894145eb70355c6477b55b5bb8a0` |

The complete, non-truncated GitHub tree at that revision was inspected for root/`thermos` license and notice paths. It lists `thermos/LICENSE` (MIT; Copyright (c) 2026 Cursor); no root license/NOTICE or additional `thermos` NOTICE was listed. The raw files’ Git blob IDs matched the tree: structural `ac76a2bc88bb2d895e83ab1788aa584a82346cfc`, correctness `1aa5dfd03bc75bc9d0b250fa9a31826cb2a32cb4`, license `ca2bba771cd39dbef6acf96b52481133983451f3`. This is byte/source verification, not signature verification or a legal clearance.

Keep the copyright and permission notice with copies/substantial portions as the MIT text requires. This repository links to the skills and records metadata; it neither vendors their bodies nor installs them globally. The [provenance inventory](PROVENANCE.md) governs future copying and dependency reuse. Updating a skill requires a new explicit revision, inspection of both bodies and terms, exact-byte hashes and a recorded policy change; do not silently follow a moving branch.

### Reproducible retrieval and verification

Run in a disposable POSIX shell with `curl`, `sha256sum` and Git available (for example Git Bash on Windows). This only downloads text to a fresh temporary directory; it does not install or execute the plugin. Keep the directory/license for the review record. Do not normalize line endings or hash a rendered HTML page.

```sh
set -eu
YURHERDR_REVIEW_DIR=$(mktemp -d)
YURHERDR_SKILL_REV=7a583913eb1b8d1a0ee76753d1a2cf072e7b54f9
YURHERDR_SKILL_ROOT="https://raw.githubusercontent.com/cursor/plugins/$YURHERDR_SKILL_REV/thermos"
curl --fail --location --silent --show-error \
  "$YURHERDR_SKILL_ROOT/skills/thermo-nuclear-code-quality-review/SKILL.md" \
  --output "$YURHERDR_REVIEW_DIR/structural-SKILL.md"
curl --fail --location --silent --show-error \
  "$YURHERDR_SKILL_ROOT/skills/thermo-nuclear-review/SKILL.md" \
  --output "$YURHERDR_REVIEW_DIR/correctness-SKILL.md"
curl --fail --location --silent --show-error \
  "$YURHERDR_SKILL_ROOT/LICENSE" --output "$YURHERDR_REVIEW_DIR/LICENSE"
cd "$YURHERDR_REVIEW_DIR"
cat > SHA256SUMS <<'SUMS'
7faca08b51b643b2ddd0836f92af15574444024685dcc1e677dbbb39ae8c9e8f  structural-SKILL.md
5091011c4490932d40658ae958fb55b9aaca8e2bba5196860295ab945180e434  correctness-SKILL.md
702f5f331b56aff0e33d8c7826df5202559f894145eb70355c6477b55b5bb8a0  LICENSE
SUMS
sha256sum --check SHA256SUMS
git hash-object structural-SKILL.md correctness-SKILL.md LICENSE
cat LICENSE
cat structural-SKILL.md
cat correctness-SKILL.md
```

The verification command must report all three `OK` results and exit 0; compare the three blob IDs with those above. Read both complete files and terms before using them as review instructions. Pass those verified files explicitly to the review session if its runtime cannot discover temporary skills. Record whether instructions were supplied manually or a supported skill invocation was used. A successful download alone is not an executed audit or a Cursor plugin installation.

For an independent source/tree cross-check, fetch this metadata to the disposable directory, confirm `sha` equals the revision and `truncated` is false, and compare each `tree` entry’s path/blob SHA with `git hash-object` output:

```sh
curl --fail --location --silent --show-error \
  "https://api.github.com/repos/cursor/plugins/git/trees/7a583913eb1b8d1a0ee76753d1a2cf072e7b54f9?recursive=1" \
  --output cursor-tree.json
```

Initial sandbox downloads in YUR-004 exited 6 (DNS); permitted network retries exited 0. GitHub HTML blob pages returned HTTP 503 during link checks, while the exact raw endpoints above succeeded and matched the pinned hashes. Record actual network/sandbox failures separately from source facts. If access, checksum verification or complete reading is unavailable, record the failed step, environment and exit code; obtain approved access or an already verified exact-byte copy before claiming the corresponding review. Never replace it with a similarly named skill, invent a hash or mark an unavailable audit complete. GitHub API rate limits or missing CLI/PR access must also be stated explicitly.

## Review procedure for every implementation change

1. **Freeze scope.** Record task/Feature/parity IDs, acceptance criteria, exact full base and head commit IDs, their merge base, changed files, full changed-file contents and the affected contracts/callers. Resolve symbolic refs once; record whether base is the intended comparison tip or a merge base. Worktree/index changes outside head must be committed or excluded explicitly. An audit of uncommitted content is not an audit of the final commit.
2. **Verify inputs.** Retrieve/read both pinned skills and terms as above. Record revision, hashes, reviewer identity/role, provider/model/effort, tool versions, date and access limits. Reviewers inspect without changing source; author fixes receive a new commit. Follow the task’s authorization for any helper/reviewer agent. Two records can come from separate passes by the same reviewer when allowed, but identify that fact; never describe author self-review as independent fresh eyes.
3. **Structural pass.** Apply `thermo-nuclear-code-quality-review` to the complete changed files and their contracts. Record structural regressions, opportunities to remove whole layers/branches, duplication, canonical ownership, type/boundary cleanliness, unjustified wrappers/casts/optionality, atomicity and avoidable orchestration complexity. A source file crossing from below 1,000 lines to above 1,000 needs decomposition or a compelling recorded structural justification. Passing tests do not settle structural findings.
4. **Correctness/security pass.** Apply `thermo-nuclear-review` separately, tracing modified behavior through callers, boundaries and real failure paths. Check security, breakage, devex/config/environment changes and feature-gate leaks caused by the diff. Investigate end-to-end before reporting; distinguish facts from hypotheses. Record identity/authorization/secret boundaries, input validation, executable/argument/cwd/environment handling, ownership/cancellation/races, bounds, persistence/rollback, platform/provider differences and affected negative paths where applicable. Explain non-applicable areas rather than asserting blanket coverage.
5. **External discussion after the audit.** Only after the independent correctness pass, when medium/high-risk findings exist and a PR/MR exists, inspect its discussion as the skill directs. Attribute imported findings and validate them. Record no PR, unavailable discussion access or other applicable limits honestly; do not create/publish a PR just to perform this step.
6. **Disposition and verify.** Give each finding an ID, level, location/contract, reproducible evidence, impact and remedy. Use blocker for unsafe behavior/work loss or broken essential behavior, major for a likely user failure or disproved claim, minor for a rare issue or style. Record fixed (commit and verification), disproven (concrete contrary evidence), or accepted (authorized owner, rationale, remaining risk and follow-up). Unresolved blocker/major findings block implementation acceptance unless the authorized owner explicitly accepts the documented risk; author/reviewer silence is not acceptance. Structural exceptions need evidence too.
7. **Bind evidence to final head.** Run appropriate checks from the [test strategy](TEST_STRATEGY.md) and record each command’s actual exit code, OS/architecture, toolchain/provider versions, fixture/environment, observed behavior and limits. Preserve failed attempts. A skipped, unavailable or unrun check is never exit 0. After fixes, update base/head, repeat affected review and verification, and explicitly carry forward only findings/evidence still applicable to the final head. If final content was never reviewed, say so and leave the gate open.

Use explicit comparisons, for example (after setting `YURHERDR_BASE` and `YURHERDR_HEAD` to the recorded full hashes):

```sh
git rev-parse "$YURHERDR_BASE^{commit}" "$YURHERDR_HEAD^{commit}"
git merge-base "$YURHERDR_BASE" "$YURHERDR_HEAD"
git diff --name-status "$YURHERDR_BASE" "$YURHERDR_HEAD"
git diff --check "$YURHERDR_BASE" "$YURHERDR_HEAD"
git diff "$YURHERDR_BASE" "$YURHERDR_HEAD" --
git status --porcelain=v1 --untracked-files=all
```

Read committed file contents with `git show "${YURHERDR_HEAD}:relative/path"`, using the recorded full head rather than a mutable working file or symbolic HEAD. A triple-dot comparison is acceptable only when its resolved merge-base scope is recorded. Review adjacent unchanged callers to understand changed behavior, but do not report unrelated pre-existing defects as regressions introduced by the change.

## Required records and limits

Use the [review record template](REVIEW_RECORD_TEMPLATE.md) for two distinctly labeled reports; they may share one evidence bundle but neither may be replaced by a combined “review passed.” Both records must identify scope, inputs, findings (or explicit none in inspected scope), dispositions, actual checks, final reviewed head and outstanding limits. Keep durable evidence accessible to the accepting owner; respect the task’s local-only publication boundary. Local records/proof and orchestration data live in ignored `local/`; the reusable template is tracked. Check exclusions and redact account identifiers, personal paths, private rules, raw logs and credentials before sharing proof.

Plugin changes additionally audit version negotiation, permissions including equivalent entry points, secret boundaries, install/update/migration rollback, cancellation/resource ownership and every pane’s detach/re-dock contract. Review reference integrations using the same public APIs available to third-party authors; reject privileged shortcuts, unchanged-plugin compatibility assumptions and unsupported sandbox claims.

Record native OS/provider evidence precisely. Linux compilation, containers or offscreen rendering do not prove Windows/macOS UI, native clipboard, accessibility, mixed DPI, multiple monitors or window lifecycle behavior. Fixture success is not live-provider support. Missing required evidence remains an open gate, with a responsible owner and next action; reviews do not certify untested platforms.

Documentation-only changes use scope, inventory coverage, immutable source/link/byte verification, workflow consistency and Git-exclusion checks. Record this limited review honestly; no executed implementation audit is implied. YUR-004 establishes access and the procedure above, and does not begin YUR-005 or claim either implementation review was run. Optional Herdr Review remains off for that task; its no-agent/no-ready/no-push boundaries remain in effect.
