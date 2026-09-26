---
id: SPEC-FIX-paragraph-sentences
doc_type: design_spec
status: current
status_since: 2026-01-05
title: Paragraph sentences
summary: A paragraph past six sentences is the one defect in this document.
---

# Paragraph sentences

The failing fixture for the paragraph rule. Each block below holds seven sentences except one. Only the first block is a finding.

The gate reads the lock. The lock names each package. Each package holds a kind. Each kind names a shelf. Each shelf holds documents. Each document has an owner. Each owner reads the report.

The gate reads the lock. The lock names each package. Each package holds a kind. Each kind names a shelf. Each shelf holds documents. Each document has an owner.

- The gate reads the lock. The lock names each package. Each package holds a kind. Each kind names a shelf. Each shelf holds documents. Each document has an owner. Each owner reads the report.
- A second item keeps the list tight.

The next list is loose, so its items are paragraphs to the parser.

- The gate reads the lock. The lock names each package. Each package holds a kind. Each kind names a shelf. Each shelf holds documents. Each document has an owner. Each owner reads the report.

- A second item keeps the list loose.

> The gate reads the lock. The lock names each package. Each package holds a kind. Each kind names a shelf. Each shelf holds documents. Each document has an owner. Each owner reads the report.
