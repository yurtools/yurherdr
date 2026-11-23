# Review evidence template

Copy into an authorized evidence location such as ignored `local/<task>/review.md`. Fill every applicable field, using **not run**, **unavailable** or **not applicable with reason** when appropriate. Do not invent exit codes. Keep private roadmap/orchestration content and raw sensitive output out of commits and shared proof. See [the review policy](REVIEW_POLICY.md) and [provenance gates](PROVENANCE.md).

## Change identity and frozen inputs

- Task / Feature / parity IDs and acceptance criteria:
- Scope (implementation or documentation only), behavior and excluded work:
- Exact base commit (40 hex), intended base branch and comparison semantics:
- Exact final head commit (40 hex) and merge base:
- Changed files, full-content inputs and affected contracts/callers/trust boundaries:
- Worktree/index state; excluded uncommitted content:
- Reused material / dependency provenance records and unresolved gates:
- Date; OS/version/architecture; tools/toolchain versions:
- Provider/CLI version, model/effort and fixture versions (or not exercised):
- Both skill source revision, path, exact-byte hashes, license verification and access results:
- Skill delivery (verified file instructions or supported invocation); anything unavailable:

## Structural review record

- Reviewer identity/role and author relationship; review date:
- Exact reviewed base/head and inspected files/contracts:
- Applied skill: thermo-nuclear-code-quality-review at the verified pin:
- Findings or explicit none within inspected scope (do not infer whole-repo clearance):

| ID / level | Location / contract | Evidence and impact | Remedy / simplification | Disposition and evidence / owner |
| --- | --- | --- | --- | --- |
| Fill or state none | | | | |

- Structural simplification considered; layers/branches/duplication removed or justified:
- Canonical ownership, boundaries/types, orchestration and atomicity:
- File sizes before/after; any 1,000-line crossing and decomposition/exception rationale:
- Justified exceptions; open risks, owner and next action:
- Final reviewed head; pass / blocked and precise scope/limits:

## Correctness/security review record

- Reviewer identity/role and author relationship; same reviewer as structural pass?:
- Review date; exact reviewed base/head and inspected callers/boundaries:
- Applied skill: thermo-nuclear-review at the verified pin:
- Modified normal/failure paths traced, including ownership/races and input/authorization boundaries:
- Configuration/devex/feature gates; platform/provider impact; plugin-specific coverage where relevant:
- Findings or explicit none within inspected scope:

| ID / level | Location / contract | Reproduction / proof and impact | Remedy | Disposition and evidence / owner |
| --- | --- | --- | --- | --- |
| Fill or state none | | | | |

- PR/MR discussion inspected AFTER audit if required; attributed findings or reason unavailable/not applicable:
- Open risks, missing evidence, owner and next action:
- Final reviewed head; pass / blocked and precise scope/limits:

## Verification tied to the final commit

| Exact command / check | Tested head | Environment / tool / provider | Actual exit code or not run | Observed result / evidence | Limits |
| --- | --- | --- | --- | --- | --- |
| Fill, including failed attempts | | | | | |

- Regression repro before/after where applicable; positive and negative evidence:
- Native versus container/headless evidence; Windows/macOS/Linux coverage separately:
- Live provider versus fixture coverage; untested versions and capabilities:
- Final changed-file scope, links/inventory and Git exclusions checked:
- Fix commits and affected follow-up review/checks; any evidence carried forward and why still applicable:
- Unresolved findings and authorized risk acceptance (owner, date, rationale, follow-up), or none:
- Evidence location / local artifact ID; redaction checked:
- Not proven / remaining gates:

## Documentation-only use

For a documentation-only change, replace the two implementation reports with a clearly labeled documentation review: scope/planned-status consistency, inventory coverage and immutable source/license evidence, retrieval/workflow verification, links and Git exclusions. Identify the self-review and its limits. Explicitly mark implementation structural and correctness/security audits **not run: no implementation change**. Record exact base/head, actual commands/exit codes and environment above; no runtime or native-platform proof is implied.
