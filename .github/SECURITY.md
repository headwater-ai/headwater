# Security policy

## What this project is, today

Headwater is in its design phase. No engine has been released, so there is no shipped artifact that can carry a vulnerability yet. This policy exists before the code does, because the design has already taken on the obligation.

[Q17](../docs/spec/09-decisions.md#q17--governed-access-and-the-solution-layer) rules that a filtered export is a security control rather than a convenience. A document withheld from an export that reaches it anyway is a leak, and a leak is the one class of defect in this system that cannot be fixed forward. The specification also fixes the one place where the project's usual "visibility before blocking" rule does not apply ([principle 4](../docs/spec/00-vision-and-scope.md#design-principles)): a control whose failure is unrecoverable ships at its final posture.

## Reporting a vulnerability

**Do not open a public issue, a pull request, or a discussion.**

Report privately through GitHub's private vulnerability reporting, on the Security tab of this repository. That channel is private to the maintainers and creates a record with a date.

Include what you have: the affected component and version or commit, what an attacker gains, and the steps that reproduce it. A report that is incomplete is still worth sending.

## What to expect

We acknowledge a report within seven days. We aim to confirm or reject it within thirty days, and to state a remediation date when we confirm it. If a report goes quiet, treat the silence as a failure of this process rather than as a judgment about the report, and say so on the same thread.

Disclosure is coordinated. We ask for ninety days from acknowledgement before public detail, and we will publish sooner when a fix is available sooner. We will credit you by the name you choose, or not at all if you prefer.

We will not pursue anyone who reports a finding in good faith under this policy. There is no bounty.

## What counts

In scope: any way a reader obtains content that a declared export filter withheld from them, any way a check reports a clean or complete result over a corpus that is neither, and any way a published artifact carries content its loss set says it dropped.

Out of scope, and by design rather than by oversight: a reader who can clone a repository reading that repository ([Q17](../docs/spec/09-decisions.md#q17--governed-access-and-the-solution-layer) places enforcement at the export step, not at the file), and the structural information that a filtered view leaks by its shape, which the specification states as mitigation rather than as a guarantee.
