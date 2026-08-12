---
id: EVAL-FIX-eta
status: current
status_since: 2026-08-05
summary: An evaluation at the source end of a relation that permits only a design spec there.
relations:
  assesses:
    - EVAL-FIX-alpha
---

# Eta

The failing fixture for the endpoint rule. `assesses` declares
`from: [design_spec]`, and this document is an `evaluation`, so the source end
is a kind the relation does not permit. The target end is fine: `alpha.md` is an
`evaluation`, and `to: [governed_document]` admits it through `is_a`.

So one instance reports one finding here and says nothing about the other end,
which is the half a check that compared kind names directly would get wrong.
