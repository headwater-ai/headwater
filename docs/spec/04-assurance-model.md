# 4 — Assurance model

How the system knows it is working, and admits where it does not.

## Assurance, not enforcement

Enforcement implies a gate that stops bad things. Real documentation systems need
four different postures, because no single gate catches enough:

| Class | Acts | Example |
|---|---|---|
| **Preventive** | Before the mistake lands | Scaffolding, editor validation, agent behaviour, commit hooks |
| **Detective** | After it lands | CI checks, scheduled drift scans, staleness sweeps |
| **Corrective** | Repairs what was found | Auto-fix, generated remediation tasks, agent-raised change proposals |
| **Adaptive** | Retunes the mechanisms | Efficacy probes, false-positive tracking, promotion decisions |

Most systems build the first two and claim the set. The adaptive layer is the one
that decides whether the other three are worth their cost, and it is specified here
as a first-class obligation rather than an aspiration.

## Obligations are data

An obligation is a stable, identified invariant the corpus commits to, recorded in
a machine-readable register — not a bullet in a strategy document that no tool can
read.

```yaml
obligations:
  - id: OB-001
    statement: Behaviour-changing code updates its governing specification in the same change
    rationale: A stale specification actively misleads humans, agents, and auditors
    severity: high
  - id: OB-014
    statement: Every live decision is reachable from at least one artefact it constrains
    rationale: Rationale nobody can find from the thing it explains is rationale nobody reads
    severity: medium
```

## Controls are data

A control declares what discharges an obligation, when it runs, and how hard it
bites.

```yaml
controls:
  - id: CT-007
    mechanism: check:relation-reciprocity
    discharges: [OB-014]
    trigger: pull_request
    posture: blocking
  - id: CT-021
    mechanism: scheduled:staleness-sweep
    discharges: [OB-003]
    trigger: weekly
    posture: detective
    handoff: task-per-finding
```

Controls are validated like anything else: a control naming a mechanism the engine
does not implement, or a pipeline that does not exist, is a finding. A register
that claims coverage it does not have is worse than no register.

## Every obligation has exactly one disposition

This is the rule that keeps the register honest:

| Disposition | Meaning |
|---|---|
| **Verified** | One or more controls discharge it |
| **Gap** | No control yet; wanted; tracked with an owner and, ideally, a target |
| **Unverifiable** | No mechanism can exist — accepted, with the reasoning recorded |

There is no fourth state and no silence. An obligation absent from the register is
itself a finding. This is the check that stops an assurance model from decaying
into a list of good intentions: the register is complete by construction or the
build fails.

The **coverage report** — what fraction of obligations are verified, by severity,
with the gap list — is generated from the register. It is never written by hand,
because a hand-written coverage claim is a marketing document.

## Feedback targets

Whether a control is *useful* depends on whether its signal reaches someone who can
act. The register records the target, and the gaps become visible:

| Target | Timing | Typical mechanism |
|---|---|---|
| Author | Pre-commit | Scaffolding, editor, agent, hooks |
| Author + reviewer | Pull request | CI checks, review annotations |
| Maintainer | Scheduled | Drift scans, efficacy probes, coverage reports |
| Team | Periodic | Conformance audits |
| Next reader | Continuous | *Usually absent* — reader feedback loops |
| Downstream consumer | On release | *Usually absent* — deprecation and breaking-change notice |

The last two rows are where documentation systems consistently fail, and naming
them in the model is how they stop being invisible. A reader who cannot tell the
maintainer that a document did not answer their question is a control that does not
exist.

## Promotion: advisory to blocking

A new check ships **advisory**. It becomes blocking only against evidence:

- a stated observation window;
- a false-positive rate under a declared threshold;
- an unambiguous, mechanical remediation path;
- no unresolved escape-hatch concentration on one shelf.

The criteria are recorded with the control, so promotion is a decision with a
paper trail rather than an argument about someone's tolerance for red builds. The
inverse is also specified: a blocking check whose false-positive rate rises past
the threshold is demoted, not endured.

## Absence is a finding class of its own

Every mechanism above validates something that exists. None of them can see the
document that should exist and does not — the accepted proposal nobody implemented,
the incident with no postmortem, the decision that never reached a specification.

**Sequence expectations** ([spec 2](02-taxonomy-model.md#sequence-expectations)) close
that gap. A sequence declares that a document of one kind, in a given state, is
expected to acquire a relation to a document of another kind within a window; the
engine reports the ones that did not.

They are constrained deliberately:

- **detective only, never blocking** — the work may legitimately be in flight,
  deferred, or abandoned for good reason, and none of those are defects;
- **windowed** — an expectation with no time bound is a wish, not a control;
- **reported against the originating document** — that is where the person who can
  act will look.

This is the drift readers complain about most, and it is invisible to link and
front-matter validation because there is nothing malformed to find.

## Findings

Every finding, from every mechanism, has one shape:

```json
{
  "rule": "relation.reciprocity.missing",
  "severity": "error",
  "obligation": "OB-014",
  "path": "docs/decisions/dr-0042.md",
  "line": 7,
  "message": "DR-ACME-0042 declares supersedes: DR-ACME-0031, which does not link back",
  "remediation": "add 'superseded_by: DR-ACME-0042' to docs/decisions/dr-0031.md",
  "fixable": true
}
```

Uniform findings are what make the rest cheap: one renderer per output format
(human, Markdown for review comments, JSON, SARIF), one severity model, one
suppression mechanism, one path from finding to fix. Every finding names the
obligation it serves — a check that cannot say which invariant it protects has not
earned its place.

## Suppression

Suppression is allowed, bounded, and observable: scoped to a file or block, it must
state a reason, and it may carry an expiry. Suppressions are inventoried in the
coverage report, because a rule with fifty suppressions is not a rule — it is a
finding about the taxonomy.

## Conformance audit

The controls above verify *form*. Whether a specification still describes the
system is a semantic question, and periodically a human (or a supervised agent)
must sample the corpus and answer it. The audit produces evidence records with
their own kind, lifecycle, and identifiers — inside the corpus, governed like
everything else. An audit whose findings are not addressable is theatre; an audit
whose output is a typed, tracked document is a control.

## The system's own assurance

docgov's obligations, controls, and gaps live in docgov's own corpus and are
checked in docgov's own CI. Coverage numbers published for the project are
generated by the same command an adopter runs. If we exempt ourselves from
something, that exemption is visible in the register — which is exactly the
property we are asking adopters to accept.
