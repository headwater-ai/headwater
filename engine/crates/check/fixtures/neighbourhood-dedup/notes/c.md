---
id: NOTE-FIX-c
relations:
  depends_on:
    - NOTE-FIX-d
    - NOTE-FIX-e
  constrains:
    - NOTE-FIX-d
---

# c

Reaches `NOTE-FIX-d` by two relations, and `NOTE-FIX-e` between them, so the second edge to `d` does not follow the first.
