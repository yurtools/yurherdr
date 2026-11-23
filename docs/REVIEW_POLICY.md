# Rewrite review policy

## Required skills

The requested review style is Cursor’s **thermo-nuclear-code-quality-review**:

https://github.com/cursor/plugins/blob/main/thermos/skills/thermo-nuclear-code-quality-review/SKILL.md

It was located and read from the official source on 2026-11-23. The complete skill is not installed globally or copied into this repository. YUR-004 must pin its source revision and establish reproducible access before implementation review. If a different “thermonuclear” skill was intended, record that correction before the first implementation change; do not pretend an unavailable skill was run.

Apply it to the complete changed files and their contracts. Prioritize structural regressions, unnecessary complexity, misplaced responsibilities, duplication and brittle type boundaries. Require behavior-preserving simplification when it removes substantial complexity. Changes that push a source file past roughly 1,000 lines need explicit decomposition review or a documented structural justification. Reject abstraction layers that add indirection without clarity.

Use Cursor’s separate **thermo-nuclear-review** for correctness/security coverage:

https://github.com/cursor/plugins/blob/main/thermos/skills/thermo-nuclear-review/SKILL.md

This companion audit traces modified behavior through callers, boundaries and failure paths. It covers security, breaking behavior, configuration/devex changes and feature-gate leaks. Resolve findings against the actual implementation rather than reporting speculative issues. Review external PR discussion after the independent audit when that discussion is relevant.

## Required evidence per implementation change

1. Scope: task/parity IDs, exact reviewed commit/base and changed contracts.
2. Structural report using the requested skill, including simplification opportunities and justified exceptions.
3. Correctness/security report covering affected callers and trust boundaries.
4. Actual test/build exit codes and applicable OS/provider/native UI evidence.
5. Finding disposition: fixed, disproven with evidence, or explicitly accepted by the authorized owner. Tests passing alone does not settle an architectural finding.

Plugin changes must additionally audit version negotiation, permission enforcement, secret boundaries, update/migration rollback and cancellation/resource ownership. Review reference integrations through the same public contracts available to third-party authors; reject hidden privileged shortcuts and unsupported sandbox claims.

Review policies are mandatory gates for the rewrite; they do not change merge authorization or license a reviewer to deploy. No implementation code exists in this initial planning change. Its review consists of scope, link, task-dependency, feature-coverage and planning-status checks; do not describe it as an executed code audit.
