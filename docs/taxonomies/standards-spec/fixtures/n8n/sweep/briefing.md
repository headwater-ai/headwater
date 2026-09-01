A coherence sweep over .agents/review-rules, 7 of the 7 classified documents of this corpus.
taxonomy: sha256:284f896b921745a85dc2f756d6b263b9c8f63f52de7bc67ab53e6c3a92d326ed

Read every document below. Report only what no linter can see, in one of the 5 classes named at the end. Your output is a proposal a person reads and accepts, and it is never a verdict: nothing in this repository gates on it.

## The slice

- .agents/review-rules/README.md (standard)
    id: N8N-STD-review-rules
    summary: How n8n files the rules its AI reviewer loads, which agent each directory maps to, and the limits that silently drop a rule.
- .agents/review-rules/db-migrations/conventions-and-tests.md (standard)
    id: N8N-STD-conventions-and-tests
    summary: The conventions a database migration answers to, and when a missing integration test is worth a comment.
- .agents/review-rules/db-migrations/data-safety.md (standard)
    id: N8N-STD-data-safety
    summary: The data hazards a database migration meets on an instance that holds dirty rows, and what a reviewer flags when it does not handle them.
- .agents/review-rules/frontend/design-system.md (standard)
    id: N8N-STD-design-system
    summary: The enforcement level a reviewer applies to n8n's frontend design tokens, which the linter validates by name and never by value.
- .agents/review-rules/qa-dx/workflow-safety.md (standard)
    id: N8N-STD-workflow-safety
    summary: What a reviewer reads in a GitHub Actions change that the workflow scanners cannot see, which is a silenced scanner, a gate that cannot fail and a widened permission.
- .agents/review-rules/security/credentials-and-secrets.md (standard)
    id: N8N-STD-credentials-and-secrets
    summary: The ways credential material reaches a log, an error message or a reader without the scope to see it, and what a reviewer flags.
- .agents/review-rules/testing/coverage.md (standard)
    id: N8N-STD-coverage
    summary: The bar n8n's reviewer applies to new behavior that ships with no test, and the four things it does not flag.

## What the graph already declares

Nothing, between two members of this slice. Every relation you propose is new.

## What to write back

One YAML file, then `headwater sweep report <path>`. The intake verifies every quotation against the file it names and refuses a finding it cannot find, so quote and never paraphrase.

```yaml
taxonomy: sha256:284f896b921745a85dc2f756d6b263b9c8f63f52de7bc67ab53e6c3a92d326ed
slice: .agents/review-rules
findings:
  - class: undeclared_conflict
    documents:
      - <a path from the slice above>
      - <a second path, where the class compares two>
    evidence:
      - path: <one of the paths above>
        quote: <the passage, copied>
    message: <what you believe, in one sentence>
    proposal:            # optional, and the best outcome
      relation: conflicts_with
      from: <identifier>
      to: <identifier>
```

## The classes

- `undeclared_conflict`: two documents contradict each other, both are current, and neither says so
- `quiet_supersession`: a newer document has quietly overtaken an older claim
- `undefined_concept`: a term is used across the slice and defined in none of it
- `audience_mismatch`: the audience the document declares could not act on what it says
- `unwritten_section`: a heading the kind requires, over prose that says nothing about it
