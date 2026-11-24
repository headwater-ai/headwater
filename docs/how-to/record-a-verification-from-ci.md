---
id: HW-HOW-record-a-verification-from-ci
status: current
status_since: 2026-11-24
summary: "After the pytest or JUnit step passes, one headwater observe call with only the verification identifiers and --commit records them, and the job calculates no digest."
last_verified: 2026-11-24
title: "Record a verification from CI"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5-5
  activity: draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-IFACE-headwater-observe
---

# Record a verification from CI

**Audience:** an adopter of Headwater whose corpus holds `verification` documents, and whose CI job runs the tests that each one names.

## Before you start

Your corpus must hold each verification as a document of kind `verification`. Exactly one acceptance criterion must reach each verification through a relation to `verification`, for example `proven_by` or `proves`. Run `headwater explain <identifier>` to see the edges of one verification. The snapshot holds one criterion digest in each entry, so `headwater observe` refuses a verification that two criteria reach.

The CI job needs the `headwater` binary on its `PATH`, and a checkout of the corpus with its `.headwater/taxonomy.lock`. The verb loads the corpus in the same way as `headwater check`, so a job that can run `headwater check` can run this verb.

You do not calculate a digest in the job. The verb takes the digest of each criterion from the census that `headwater check` compares it with.

## Steps

1. Run the tests that prove the verifications, in a step of their own. The next step runs only when this step passes.

2. Call `headwater observe` with the identifiers of the verifications that the step proved, and with the commit that the job checked out. Give nothing else.

   A pytest job on GitHub Actions:

   ```yaml
   - name: Run the tests
     run: pytest tests/test_document_paths.py
   - name: Record the verifications
     run: headwater observe HW-VER-0002 --commit "$GITHUB_SHA"
   ```

   A JUnit job that Maven runs:

   ```yaml
   - name: Run the tests
     run: mvn --batch-mode test -Dtest=DocumentPathTest
   - name: Record the verifications
     run: headwater observe HW-VER-0002 HW-VER-0003 --commit "$GITHUB_SHA"
   ```

   GitHub Actions skips a step after a step that failed, so the verb runs only when the tests passed. On another CI system, give the step the same condition.

3. Commit `.headwater/observations.yml` back to the repository in the way that your repository commits a file that a job writes, for example in a pull request that the job opens. `headwater check` reads the committed file.

## How to know it worked

The verb exits 0 and prints one line for each verification, in the form `<verification> observed at <commit>, <criterion> at <digest>`.

Run `headwater check` over the corpus with the new snapshot. The verification block shows each verification as `observed at <commit>`, and no finding of `relation.target.verification.suspect` names it.

Run the same step again with the same commit. The verb prints the same lines and says on standard error that nothing was written, and the file keeps its bytes.

When somebody later edits the criterion, `headwater check` shows the verification as `suspect`. The next passing run of the job records it again, and it then shows as `observed`.

When the verb exits 1, standard error names each identifier that it refused, with the reason. It still writes the entries of the other identifiers. [`headwater observe`](../interfaces/headwater-observe.md) lists each refusal.
