---
id: HW-NOTE-untyped-carrier
title: "A note that carries an identifier and sits on no shelf"
---

# A note that carries an identifier and sits on no shelf

Planted at `docs/doctrine/`, which no shelf pattern claims, so the census gives
this file a row and no kind. The identifier above is therefore carried by a
document the graph index holds among its untyped nodes.

That is the only near miss this engine computes. `headwater explain` refuses
the identifier through the path matcher and says nothing useful about it, and
`resolve_identifier` is what separates this state from an identifier nothing
carries at all. The checker makes both calls, and the case beside this file is
what holds the second one.
