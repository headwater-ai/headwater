---
"headwater:generated": "probe_result. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: HW-RESULT-campaign-of-2026-10-05-campaign-tier-no-skills-arm-discovery
title: Probe result for campaign-of-2026-10-05-campaign-tier-no-skills-arm-discovery
status: current
status_since: 2026-10-08
summary: "The grade of the transcript `campaign-of-2026-10-05-campaign-tier-no-skills-arm-discovery`, taken over the probes this corpus declares and the version of the grader that evaluated them."
last_verified: 2026-10-08
---

# The result of docs/probe-runs/campaign-of-2026-10-05-campaign-tier-no-skills-arm-discovery.md

A probe result is a function of committed inputs and of nothing else: the transcript at `docs/probe-runs/campaign-of-2026-10-05-campaign-tier-no-skills-arm-discovery.md` and the state it stands at, the expectations the probes it names declare, the version of the grader that evaluated them, and for a compared arm the result of the other arm. Fetch those and this file comes back.

## The run this transcript recorded

A campaign run in the no-skills arm, on claude-sonnet-5 at 2026-10-05.
served version claude-sonnet-5, tree sha256:81d5f62acb289022175d192df0a61faf5f71eb4b79e8dd1f4efae4532ebdd916, selection sha256:c84af3cc500bdabda0be3c6dd5e76b38223d5e9950b0fc012f9fc68e7c9a1e32, read set sha256:0934c679a536b4e44258201b27ca9ee9bea11fa827b636e241fed118610860c7, seed 0, harness 0.5.0.
realized cost $140.27, which the adaptive layer reads as the cost of its own instrument.

354 events over 3 probes, in 354 sessions and 4587 tool calls.

The engine confirmed the taxonomy, that every member of the run identity is present, the membership of every probe named, that no key outside the closed set appears, and that a realized cost was recorded. Present is not confirmed. This page names the lock, the selection and the read set the transcript recorded, and it compares none of them with the tree in front of a reader. `headwater generate` and `headwater probe stale` name what moved since the recording, and neither one fails for it. It graded nothing: a verdict is a function of this transcript, the expectations these probes declare and a grader version, and `headwater probe grade` is the verb that holds all three.

The selection this transcript names is `sha256:c84af3cc500bdabda0be3c6dd5e76b38223d5e9950b0fc012f9fc68e7c9a1e32`, and the probes graded below are the probes it was planned over, as this corpus declares them now. A probe added to this corpus after the recording is not a session this run owed. The tree, the seed and the harness above are provenance and nothing compares them.

The `read_set` digest above covers every probe of the selection and every document one of them examines, by path and content. This file names it as provenance and compares it with nothing, and the same holds for the lock and the selection: an edit to a document, a new probe or a moved lock leaves these bytes alone. `headwater generate` names each result whose recorded lock, read set or selection moved, and `headwater probe stale` takes the digest and reports which recorded results a change voided. Neither one fails for it.

Graded by grader 0.5.0.
A campaign run in the no-skills arm, on claude-sonnet-5 at served version claude-sonnet-5.

## The verdicts

- HW-PROBE-a-cold-agent-reaches-the-governing-document-through-the-corpus-descriptor (discovery, expects opened)
    session L4-campaign-no-skills-p1-r1: satisfied — event 1, call 14: `Bash {"command":"grep -n \"^##\\|^###\\|severity\\|rule id\\|\\`[a-z_]*\\.[a-z_]*\\.[a-z_]*\\`\" docs/spec/12-check-layer.md | head -120"}`
    session L4-campaign-no-skills-p1-r10: not satisfied — 19 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r100: not satisfied — 34 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r101: not satisfied — 19 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r102: not satisfied — 27 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r103: not satisfied — 31 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r104: not satisfied — 51 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r105: not satisfied — 15 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r106: satisfied — event 9, call 24: `Bash {"command":"grep -rn \"summary\" docs/spec/12-check-layer.md 2>/dev/null | head -30","description":"Search spec 12 for mentions of summary"}`
    session L4-campaign-no-skills-p1-r107: not satisfied — 28 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r108: satisfied — event 11, call 28: `Grep /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p1-r108/docs/spec/12-check-layer.md`
    session L4-campaign-no-skills-p1-r109: not satisfied — 15 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r11: not satisfied — 19 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r110: not satisfied — 13 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r111: not satisfied — 10 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r112: not satisfied — 19 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r113: satisfied — event 17, call 16: `Grep /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p1-r113/docs/spec/12-check-layer.md`
    session L4-campaign-no-skills-p1-r114: not satisfied — 13 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r115: not satisfied — 17 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r116: not satisfied — 17 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r117: not satisfied — 13 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r118: not satisfied — 17 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r12: not satisfied — 11 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r13: satisfied — event 24, call 7: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p1-r13/docs/spec/12-check-layer.md`
    session L4-campaign-no-skills-p1-r14: not satisfied — 9 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r15: not satisfied — 31 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r16: not satisfied — 15 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r17: not satisfied — 11 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r18: not satisfied — 9 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r19: not satisfied — 19 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r2: not satisfied — 34 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r20: satisfied — event 32, call 16: `Bash {"command":"grep -n \"posture\" docs/spec/12-check-layer.md 2>/dev/null | head -20; echo ---; grep -rn \"enum Severity\" engine/crates/check/src/finding.rs","description":"Check meaning of posture field and Severity enum"}`
    session L4-campaign-no-skills-p1-r21: not satisfied — 16 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r22: not satisfied — 14 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r23: not satisfied — 15 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r24: satisfied — event 36, call 8: `Grep /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p1-r24/docs/spec/12-check-layer.md`
    session L4-campaign-no-skills-p1-r25: not satisfied — 20 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r26: not satisfied — 11 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r27: satisfied — event 39, call 24: `Grep /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p1-r27/docs/spec/12-check-layer.md`
    session L4-campaign-no-skills-p1-r28: not satisfied — 17 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r29: not satisfied — 10 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r3: satisfied — event 42, call 21: `Bash {"command":"cd /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p1-r3\ngrep -n \"severity\\|posture\\|error\\|advisory\\|mechanical\\|total\" docs/spec/12-check-layer.md | head -80"}`
    session L4-campaign-no-skills-p1-r30: not satisfied — 12 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r31: not satisfied — 19 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r32: not satisfied — 11 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r33: satisfied — event 46, call 9: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p1-r33/docs/spec/12-check-layer.md`
    session L4-campaign-no-skills-p1-r34: not satisfied — 10 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r35: not satisfied — 11 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r36: not satisfied — 20 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r37: satisfied — event 50, call 32: `Grep /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p1-r37/docs/spec/12-check-layer.md`
    session L4-campaign-no-skills-p1-r38: not satisfied — 23 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r39: satisfied — event 52, call 27: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p1-r39/docs/spec/12-check-layer.md`
    session L4-campaign-no-skills-p1-r4: not satisfied — 14 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r40: satisfied — event 54, call 37: `Bash {"command":"grep -n \"severity-is-the-checks-posture-is-the-controls\\|## Severity is the check\" -A 40 docs/spec/12-check-layer.md | head -80","description":"Read spec 12 section on severity vs posture"}`
    session L4-campaign-no-skills-p1-r41: not satisfied — 13 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r42: not satisfied — 15 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r43: not satisfied — 16 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r44: not satisfied — 13 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r45: not satisfied — 14 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r46: not satisfied — 17 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r47: not satisfied — 12 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r48: not satisfied — 12 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r49: not satisfied — 19 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r5: satisfied — event 64, call 12: `Bash {"command":"grep -n -i \"severity\\|fixability\\|advisory\\|mechanical\" docs/spec/12-check-layer.md | head -60"}`
    session L4-campaign-no-skills-p1-r50: not satisfied — 24 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r51: not satisfied — 20 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r52: not satisfied — 21 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r53: satisfied — event 68, call 23: `Bash {"command":"cd /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p1-r53\necho \"=== docs/spec/12-check-layer.md: severity-is-the-checks-posture-is-the-controls section ===\"\ngrep -n \"^#\" docs/spec/12-check-layer.md | head -60\n"}`
    session L4-campaign-no-skills-p1-r54: not satisfied — 19 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r55: satisfied — event 70, call 37: `Bash {"command":"cd /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p1-r55\ngrep -n \"severity-is-the-checks-posture-is-the-controls\" -A 30 docs/spec/12-check-layer.md | head -60"}`
    session L4-campaign-no-skills-p1-r56: not satisfied — 17 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r57: satisfied — event 72, call 35: `Grep /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p1-r57/docs/spec/12-check-layer.md`
    session L4-campaign-no-skills-p1-r58: not satisfied — 12 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r59: not satisfied — 41 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r6: satisfied — event 75, call 27: `Bash {"command":"ROOT=/var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p1-r6\ngrep -n \"new rule\\|add a rule\\|adding a rule\\|new check\\|generated, not written\\|five origins\" $ROOT/docs/spec/12-check-layer.md | head -40\n"}`
    session L4-campaign-no-skills-p1-r60: not satisfied — 13 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r61: not satisfied — 16 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r62: satisfied — event 78, call 9: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p1-r62/docs/spec/12-check-layer.md`
    session L4-campaign-no-skills-p1-r63: not satisfied — 11 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r64: satisfied — event 80, call 26: `Bash {"command":"cd /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p1-r64\nsed -n '1,400p' docs/spec/12-check-layer.md | grep -n \"origin\\|Shape\\|generated\\|Severity\\|severity\" | head -80","description":"Search check-layer spec for origin and severity discussion"}`
    session L4-campaign-no-skills-p1-r65: not satisfied — 13 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r66: satisfied — event 82, call 19: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p1-r66/docs/spec/12-check-layer.md`
    session L4-campaign-no-skills-p1-r67: not satisfied — 18 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r68: satisfied — event 84, call 37: `Bash {"command":"cd /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p1-r68\necho \"=== 12-check-layer.md severity section ===\"\ngrep -n \"^#.*[Ss]everity\\|error.*warn.*info\\|^### \" docs/spec/12-check-layer.md | head -20\necho \"=== 02-taxonomy-model.md facets section header ===\"\ngrep -n \"^#.*[Ff]acet\" docs/spec/02-taxonomy-model.md | head -10","description":"Find severity section in spec 12 and facets section in spec 02"}`
    session L4-campaign-no-skills-p1-r69: not satisfied — 13 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r7: satisfied — event 86, call 15: `Bash {"command":"sed -n '1,120p' /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p1-r7/docs/spec/12-check-layer.md 2>/dev/null | head -150","description":"Read spec 12 check layer doc intro"}`
    session L4-campaign-no-skills-p1-r70: not satisfied — 25 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r71: not satisfied — 19 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r72: not satisfied — 12 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r73: not satisfied — 24 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r74: not satisfied — 16 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r75: not satisfied — 10 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r76: not satisfied — 16 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r77: not satisfied — 16 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r78: not satisfied — 20 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r79: satisfied — event 96, call 20: `Grep /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p1-r79/docs/spec/12-check-layer.md`
    session L4-campaign-no-skills-p1-r8: not satisfied — 18 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r80: not satisfied — 26 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r81: not satisfied — 15 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r82: not satisfied — 22 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r83: satisfied — event 101, call 12: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p1-r83/docs/spec/12-check-layer.md`
    session L4-campaign-no-skills-p1-r84: satisfied — event 102, call 17: `Grep /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p1-r84/docs/spec/12-check-layer.md`
    session L4-campaign-no-skills-p1-r85: not satisfied — 38 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r86: satisfied — event 104, call 26: `Bash {"command":"grep -n \"^#\\|dotted\\|naming\" /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p1-r86/docs/spec/12-check-layer.md | head -60"}`
    session L4-campaign-no-skills-p1-r87: satisfied — event 105, call 30: `Bash {"command":"cd /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p1-r87\ngrep -n \"severity-is-the-checks-posture-is-the-controls\" -A 15 docs/spec/12-check-layer.md | head -25","description":"Read spec 12 severity-vs-posture section heading context"}`
    session L4-campaign-no-skills-p1-r88: satisfied — event 106, call 32: `Bash {"command":"grep -n \"^#\\|^##\\|five origins\\|Shape-origin\\|fixability\\|Fixability\" /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p1-r88/docs/spec/12-check-layer.md | head -60","description":"View headings in check-layer spec"}`
    session L4-campaign-no-skills-p1-r89: not satisfied — 33 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r9: not satisfied — 13 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r90: not satisfied — 15 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r91: satisfied — event 110, call 10: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p1-r91/docs/spec/12-check-layer.md`
    session L4-campaign-no-skills-p1-r92: not satisfied — 13 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r93: satisfied — event 112, call 17: `Bash {"command":"cd /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p1-r93\ngrep -n \"fixability\\|mechanical\\|five origins\\|rule id\\|rule_id\\|dotted\" docs/spec/12-check-layer.md | head -30"}`
    session L4-campaign-no-skills-p1-r94: not satisfied — 16 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r95: not satisfied — 26 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r96: not satisfied — 26 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r97: satisfied — event 116, call 13: `Bash {"command":"grep -n \"posture\" -r docs/spec/12-check-layer.md docs/spec/*.md 2>/dev/null | head -20","description":"Find definition of 'posture' in spec docs"}`
    session L4-campaign-no-skills-p1-r98: not satisfied — 34 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p1-r99: satisfied — event 118, call 24: `Grep docs/spec/12-check-layer.md`
- HW-PROBE-a-session-finds-a-ruling-from-a-task-in-plain-words (discovery, expects opened)
    session L4-campaign-no-skills-p2-r1: satisfied — event 119, call 4: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r1/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r10: satisfied — event 120, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r10/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r100: satisfied — event 121, call 2: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r100/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r101: satisfied — event 122, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r101/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r102: satisfied — event 123, call 2: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r102/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r103: satisfied — event 124, call 4: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r103/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r104: satisfied — event 125, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r104/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r105: satisfied — event 126, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r105/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r106: satisfied — event 127, call 4: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r106/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r107: satisfied — event 128, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r107/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r108: not satisfied — 2 recorded calls, and none named any of the 2 documents
    session L4-campaign-no-skills-p2-r109: satisfied — event 130, call 4: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r109/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r11: satisfied — event 131, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r11/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r110: satisfied — event 132, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r110/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r111: satisfied — event 133, call 2: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r111/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r112: satisfied — event 134, call 2: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r112/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r113: satisfied — event 135, call 4: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r113/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r114: satisfied — event 136, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r114/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r115: satisfied — event 137, call 5: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r115/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r116: satisfied — event 138, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r116/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r117: satisfied — event 139, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r117/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r118: satisfied — event 140, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r118/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r12: satisfied — event 141, call 2: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r12/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r13: satisfied — event 142, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r13/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r14: satisfied — event 143, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r14/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r15: satisfied — event 144, call 2: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r15/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r16: satisfied — event 145, call 5: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r16/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r17: satisfied — event 146, call 4: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r17/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r18: satisfied — event 147, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r18/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r19: satisfied — event 148, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r19/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r2: satisfied — event 149, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r2/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r20: satisfied — event 150, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r20/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r21: not satisfied — 2 recorded calls, and none named any of the 2 documents
    session L4-campaign-no-skills-p2-r22: satisfied — event 152, call 4: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r22/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r23: satisfied — event 153, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r23/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r24: satisfied — event 154, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r24/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r25: satisfied — event 155, call 2: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r25/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r26: satisfied — event 156, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r26/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r27: satisfied — event 157, call 2: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r27/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r28: satisfied — event 158, call 4: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r28/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r29: satisfied — event 159, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r29/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r3: satisfied — event 160, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r3/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r30: satisfied — event 161, call 4: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r30/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r31: satisfied — event 162, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r31/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r32: satisfied — event 163, call 4: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r32/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r33: not satisfied — 2 recorded calls, and none named any of the 2 documents
    session L4-campaign-no-skills-p2-r34: satisfied — event 165, call 4: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r34/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r35: satisfied — event 166, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r35/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r36: satisfied — event 167, call 5: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r36/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r37: satisfied — event 168, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r37/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r38: satisfied — event 169, call 2: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r38/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r39: satisfied — event 170, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r39/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r4: satisfied — event 171, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r4/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r40: satisfied — event 172, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r40/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r41: satisfied — event 173, call 2: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r41/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r42: satisfied — event 174, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r42/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r43: satisfied — event 175, call 2: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r43/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r44: satisfied — event 176, call 4: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r44/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r45: satisfied — event 177, call 5: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r45/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r46: satisfied — event 178, call 4: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r46/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r47: satisfied — event 179, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r47/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r48: satisfied — event 180, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r48/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r49: not satisfied — 1 recorded call, and none named any of the 2 documents
    session L4-campaign-no-skills-p2-r5: satisfied — event 182, call 4: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r5/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r50: satisfied — event 183, call 4: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r50/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r51: satisfied — event 184, call 4: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r51/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r52: satisfied — event 185, call 4: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r52/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r53: satisfied — event 186, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r53/docs/evaluations/why-corpus-counts-are-derived-not-stored.md`
    session L4-campaign-no-skills-p2-r54: satisfied — event 187, call 4: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r54/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r55: satisfied — event 188, call 2: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r55/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r56: satisfied — event 189, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r56/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r57: satisfied — event 190, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r57/docs/evaluations/why-corpus-counts-are-derived-not-stored.md`
    session L4-campaign-no-skills-p2-r58: satisfied — event 191, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r58/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r59: satisfied — event 192, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r59/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r6: satisfied — event 193, call 5: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r6/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r60: satisfied — event 194, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r60/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r61: not satisfied — 1 recorded call, and none named any of the 2 documents
    session L4-campaign-no-skills-p2-r62: satisfied — event 196, call 4: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r62/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r63: satisfied — event 197, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r63/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r64: satisfied — event 198, call 4: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r64/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r65: satisfied — event 199, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r65/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r66: satisfied — event 200, call 2: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r66/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r67: satisfied — event 201, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r67/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r68: satisfied — event 202, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r68/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r69: satisfied — event 203, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r69/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r7: satisfied — event 204, call 4: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r7/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r70: satisfied — event 205, call 4: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r70/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r71: satisfied — event 206, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r71/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r72: satisfied — event 207, call 6: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r72/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r73: satisfied — event 208, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r73/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r74: satisfied — event 209, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r74/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r75: satisfied — event 210, call 4: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r75/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r76: satisfied — event 211, call 4: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r76/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r77: satisfied — event 212, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r77/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r78: satisfied — event 213, call 2: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r78/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r79: satisfied — event 214, call 2: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r79/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r8: satisfied — event 215, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r8/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r80: satisfied — event 216, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r80/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r81: satisfied — event 217, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r81/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r82: satisfied — event 218, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r82/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r83: satisfied — event 219, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r83/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r84: satisfied — event 220, call 4: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r84/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r85: satisfied — event 221, call 2: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r85/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r86: satisfied — event 222, call 2: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r86/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r87: satisfied — event 223, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r87/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r88: satisfied — event 224, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r88/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r89: satisfied — event 225, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r89/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r9: satisfied — event 226, call 4: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r9/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r90: satisfied — event 227, call 4: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r90/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r91: satisfied — event 228, call 4: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r91/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r92: satisfied — event 229, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r92/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r93: satisfied — event 230, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r93/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r94: satisfied — event 231, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r94/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r95: satisfied — event 232, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r95/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r96: satisfied — event 233, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r96/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r97: satisfied — event 234, call 4: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r97/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r98: satisfied — event 235, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p2-r98/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`
    session L4-campaign-no-skills-p2-r99: not satisfied — 2 recorded calls, and none named any of the 2 documents
- HW-PROBE-the-pointer-this-corpus-offers-for-a-task-is-the-document-a-session-opens (discovery, expects opened)
    session L4-campaign-no-skills-p3-r1: not satisfied — 10 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r10: not satisfied — 7 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r100: not satisfied — 4 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r101: satisfied — event 240, call 5: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p3-r101/docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md`
    session L4-campaign-no-skills-p3-r102: not satisfied — 9 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r103: not satisfied — 7 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r104: not satisfied — 17 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r105: not satisfied — 10 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r106: not satisfied — 5 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r107: satisfied — event 246, call 12: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p3-r107/docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md`
    session L4-campaign-no-skills-p3-r108: not satisfied — 10 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r109: satisfied — event 248, call 9: `Read docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md`
    session L4-campaign-no-skills-p3-r11: not satisfied — 11 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r110: not satisfied — 19 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r111: not satisfied — 11 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r112: not satisfied — 9 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r113: not satisfied — 17 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r114: not satisfied — 18 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r115: not satisfied — 15 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r116: satisfied — event 256, call 9: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p3-r116/docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md`
    session L4-campaign-no-skills-p3-r117: not satisfied — 19 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r118: satisfied — event 258, call 6: `Bash {"command":"for f in docs/decisions/0025-q25-where-the-namespace-goes-in-an-identifier-and-who-declares-it.md docs/decisions/0027-q27-whether-a-decision-record-is-governed-prose.md docs/decisions/0028-q28-whether-an-evaluation-is-governed-prose.md docs/process/decisions/0024-a-document-that-moves-to-a-process-shelf-keeps-its-identifier.md docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md; do echo \"=== $f ===\"; cat \"$f\" 2>/dev/null | head -100; echo; done"}`
    session L4-campaign-no-skills-p3-r12: not satisfied — 22 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r13: satisfied — event 260, call 11: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p3-r13/docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md`
    session L4-campaign-no-skills-p3-r14: satisfied — event 261, call 10: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p3-r14/docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md`
    session L4-campaign-no-skills-p3-r15: satisfied — event 262, call 10: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p3-r15/docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md`
    session L4-campaign-no-skills-p3-r16: not satisfied — 11 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r17: not satisfied — 17 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r18: not satisfied — 10 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r19: not satisfied — 18 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r2: not satisfied — 16 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r20: not satisfied — 13 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r21: not satisfied — 15 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r22: satisfied — event 270, call 12: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p3-r22/docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md`
    session L4-campaign-no-skills-p3-r23: not satisfied — 9 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r24: satisfied — event 272, call 12: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p3-r24/docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md`
    session L4-campaign-no-skills-p3-r25: not satisfied — 10 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r26: not satisfied — 13 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r27: not satisfied — 9 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r28: not satisfied — 15 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r29: not satisfied — 8 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r3: not satisfied — 13 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r30: not satisfied — 14 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r31: not satisfied — 18 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r32: not satisfied — 9 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r33: not satisfied — 27 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r34: not satisfied — 17 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r35: not satisfied — 9 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r36: not satisfied — 9 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r37: not satisfied — 9 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r38: not satisfied — 15 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r39: satisfied — event 288, call 25: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p3-r39/docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md`
    session L4-campaign-no-skills-p3-r4: satisfied — event 289, call 19: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p3-r4/docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md`
    session L4-campaign-no-skills-p3-r40: satisfied — event 290, call 9: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p3-r40/docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md`
    session L4-campaign-no-skills-p3-r41: not satisfied — 11 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r42: not satisfied — 17 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r43: satisfied — event 293, call 7: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p3-r43/docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md`
    session L4-campaign-no-skills-p3-r44: not satisfied — 10 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r45: not satisfied — 13 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r46: not satisfied — 19 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r47: satisfied — event 297, call 16: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p3-r47/docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md`
    session L4-campaign-no-skills-p3-r48: satisfied — event 298, call 8: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p3-r48/docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md`
    session L4-campaign-no-skills-p3-r49: not satisfied — 9 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r5: not satisfied — 13 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r50: not satisfied — 16 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r51: satisfied — event 302, call 5: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p3-r51/docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md`
    session L4-campaign-no-skills-p3-r52: not satisfied — 12 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r53: not satisfied — 6 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r54: satisfied — event 305, call 11: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p3-r54/docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md`
    session L4-campaign-no-skills-p3-r55: satisfied — event 306, call 9: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p3-r55/docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md`
    session L4-campaign-no-skills-p3-r56: not satisfied — 15 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r57: not satisfied — 15 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r58: not satisfied — 18 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r59: not satisfied — 13 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r6: not satisfied — 18 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r60: not satisfied — 18 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r61: not satisfied — 8 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r62: not satisfied — 19 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r63: satisfied — event 315, call 19: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p3-r63/docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md`
    session L4-campaign-no-skills-p3-r64: not satisfied — 7 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r65: satisfied — event 317, call 5: `Bash {"command":"cat /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p3-r65/docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md","description":"Read obligation 0107 referenced by router"}`
    session L4-campaign-no-skills-p3-r66: not satisfied — 9 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r67: not satisfied — 8 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r68: not satisfied — 12 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r69: not satisfied — 8 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r7: not satisfied — 8 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r70: not satisfied — 10 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r71: satisfied — event 324, call 8: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p3-r71/docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md`
    session L4-campaign-no-skills-p3-r72: not satisfied — 10 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r73: not satisfied — 8 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r74: satisfied — event 327, call 16: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p3-r74/docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md`
    session L4-campaign-no-skills-p3-r75: satisfied — event 328, call 9: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p3-r75/docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md`
    session L4-campaign-no-skills-p3-r76: satisfied — event 329, call 12: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p3-r76/docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md`
    session L4-campaign-no-skills-p3-r77: not satisfied — 7 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r78: satisfied — event 331, call 3: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p3-r78/docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md`
    session L4-campaign-no-skills-p3-r79: not satisfied — 22 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r8: not satisfied — 14 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r80: not satisfied — 13 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r81: not satisfied — 12 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r82: not satisfied — 12 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r83: satisfied — event 337, call 9: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p3-r83/docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md`
    session L4-campaign-no-skills-p3-r84: not satisfied — 8 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r85: not satisfied — 8 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r86: not satisfied — 6 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r87: not satisfied — 14 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r88: satisfied — event 342, call 5: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p3-r88/docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md`
    session L4-campaign-no-skills-p3-r89: not satisfied — 12 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r9: satisfied — event 344, call 7: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p3-r9/docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md`
    session L4-campaign-no-skills-p3-r90: not satisfied — 15 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r91: not satisfied — 12 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r92: not satisfied — 9 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r93: satisfied — event 348, call 12: `Read /var/tmp/hw-1659/b/ws/L4-campaign-no-skills-p3-r93/docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md`
    session L4-campaign-no-skills-p3-r94: not satisfied — 16 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r95: not satisfied — 9 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r96: not satisfied — 9 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r97: not satisfied — 10 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r98: not satisfied — 11 recorded calls, and none named any of the 1 document
    session L4-campaign-no-skills-p3-r99: not satisfied — 12 recorded calls, and none named any of the 1 document

## The rate, and the denominator it is over

175 of 354 graded sessions satisfied their expectation: 49.4%, in a 95% interval of 44.3% to 54.6%.
The denominator is the graded sessions and never the selected probes. 0 sessions reached no verdict, and a session with no verdict is outside both halves of that fraction.

An interval that overlaps the previous run's is variance and one that does not is drift. This is one arm, so it estimates no effect: an efficacy claim is a comparison of two results, and the arm each one recorded is on it.

## The comparisons this arm takes part in

What the skills change. The treated arm is the `campaign` present arm and the control is the `campaign` `no-skills` arm, which removed the paths the `no-skills` component's delta names. The documents are in both arms (spec 5).

- treated, `docs/probe-runs/campaign-of-2026-10-05-campaign-tier-present-arm-discovery.md`: 137 of 354 graded sessions satisfied their expectation, 38.7%, in a 95% interval of 33.8% to 43.9%. 0 sessions refused by the session itself.
- control, `docs/probe-runs/campaign-of-2026-10-05-campaign-tier-no-skills-arm-discovery.md`: 175 of 354 graded sessions satisfied their expectation, 49.4%, in a 95% interval of 44.3% to 54.6%. 0 sessions refused by the session itself.

The difference is -10.7 points, in a 95% Newcombe interval of -17.9 points to -3.4 points. The interval is below zero, so the treated arm satisfied less often at the 5% level.
