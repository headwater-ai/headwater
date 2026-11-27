- **Voice regime** — the voice in which a document is written, with the machine-checkable part of it: e.g. *declarative present-state* (no future intent, no narration of change, no phased-rollout language), *narrative* (time-boxed exploration, changelogs), or *unconstrained*.
- **Lifecycle regime** — a state machine over the state facet: states, legal transitions, terminal states, and what each state implies (e.g. superseded documents are retained and delinked from live dependency paths).

`headwater check --fix` writes the corrections the engine can derive without judgment and prints what it wrote on standard error. It leaves every finding whose remedy is a rewrite, and it refuses a file rather than half-writing one. Read the diff before you commit it.
