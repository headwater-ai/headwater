#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
#
# The tuning round of the additive campaign, from the plan to the committed
# record, as eleven verbs (#1821, #1822, #1838). Before this script, each try
# had a launch script, a wait condition, a canary reader and a keep decision
# typed by hand, the total spend of the round was prose that a person added
# to by hand, and one hand reader read `named 1 path outside` as a blank field
# because it matched the plural alone. Before #1838 it ran one try of 70
# sessions and kept it on that try's own figures, where the owner ruled on
# 2026-10-22 (#1777) for rounds of screened variants and a confirmation.
#
#     sh tools/probe/try.sh status   <root>
#     sh tools/probe/try.sh spent    <root>
#     sh tools/probe/try.sh plan     <root> [<stage>]
#     sh tools/probe/try.sh launch   <root> <stage>
#     sh tools/probe/try.sh rank     <root>
#     sh tools/probe/try.sh relaunch <root> <try>
#     sh tools/probe/try.sh canary   <root> <try>
#     sh tools/probe/try.sh wait     <root> <try>
#     sh tools/probe/try.sh result   <root> <try>
#     sh tools/probe/try.sh ledger   <root>
#     sh tools/probe/try.sh accept   <root> <ruling-url>
#
# `<root>` is the root of the tuning round: one directory that holds every
# try root of the round and the two files of the round: `<root>/env`, which
# a person writes once, and `<root>/base-pin`, which `result` of a base
# rewrites. A try root is the `--root` of
# one `tools/probe/campaign-drive.sh` driver, and its batch directories are
# `<try>/<batch>/sessions/<job>`. `<try>` names a try root under `<root>`.
#
# ## Rounds, stages and try roots
#
# The open layer and its round are `rounds.tuning.open` in
# `.headwater/probe.yml`: its `layer`, and its `try`, which is the round `n`.
# The figures of a round are the seven fields of the tuning round that
# `tools/probe/declared.py <declaration> tuning` prints, with the count of
# each stage computed there and nowhere else: over the five tuning probes, a
# screened variant runs 25 sessions and a confirmation 145. A round has three
# stages, a fourth stage measures the base outside the rounds, and each
# `<stage>` names one launch and its try root:
#
# - `screen <k>` runs variant `k` of the round at the screen's count, in
#   `<root>/<layer>-r<n>-v<k>`, on a fixed draw (#1875): with `screen: 22`
#   over 22 tuning probes, that is one session for each probe. `k` is 1 to
#   `variants`, so a fifth variant
#   of four is refused. The variant is the commit of the checkout, and its
#   scenario tree is built with `campaign.sh --scenario-tree` at `HEAD`. A
#   variant whose tree has the digest of another variant's of the round is
#   refused, because two variants that share one tree screen one change.
#   `launch` writes the digest in `<try>/tree`.
# - `best` measures the best so far of the layer at the confirmation's count,
#   in `<root>/<layer>-r<n>-best`, at the checkout's `HEAD`. It runs only
#   where the layer has no measurement at that count: no `best` event and no
#   kept confirmation. Otherwise it is refused.
# - `confirm <variant>` runs the variant named, a try root of a screen of
#   the round, at the confirmation's count, in `<root>/<layer>-r<n>-confirm`,
#   from a checkout at the commit that variant ran at.
# - `base` measures the base scenario of the open layer, scenario 3 for the
#   skills, the hook and MCP, at the confirmation's count, in
#   `<root>/base-<k>`, at the checkout's `HEAD`, with `k` the first free
#   number from 1. One base serves every layer of that base, so the try root
#   names no layer. It is no stage of a round: it reads no rule of rounds
#   and order, and it builds no tree, because it is the measurement the
#   base check asks for. It is refused where the ledger holds a `base` event
#   at `HEAD`'s commit already. A confirmation waits for one (#1838).
#
# `launch` writes the stage in `<try>/stage`, and each driver it starts gets
# `HEADWATER_TUNING_STAGE`, `screen` or `confirm`, which the dry run reads
# for the count of the batch. A best so far and a base run as `confirm`.
#
# **The fixed draw of a screen (#1875).** A probe's `## Task` holds two or
# three `### Phrasing <n>` subsections (HW-DR-0112), and a task with none is
# one phrasing. `plan screen <k>` prints one `draw: <probe> <n>` line for
# each session of the screen, in the order of the set, and a probe at more
# than one repetition has a line for each. The phrasing is
# `tools/probe/phrasing.sh draw` of `TRY_SEED`, the open layer, the round
# `n`, the probe and the repetition, over the probe's count of phrasings in
# the checkout. So every variant of a round gets the same draw, and `plan`
# writes nothing. `launch` writes the lines to `<try>/draw`, and `launch` and
# `relaunch` give the driver `HEADWATER_CAMPAIGN_DRAW=<try>/draw`, from which
# `campaign.sh` takes each session's phrasing. A best so far, a
# confirmation and a base draw nothing: `campaign.sh` spreads the
# phrasings of each probe evenly over its sessions.
#
# **A hook variant runs its moment plus row 11 (#1875).** The hooks layer
# names its hook moment in `rounds.tuning.open`, as `moment`. A variant of
# `read`, `write` or `review` runs the tuning probes that
# `headwater probe plan --moment <m>` selects, together with those that
# `--action ungoverned_control` selects, which is row 11 of HW-DR-0112. They
# are two calls, because the two flags of one call give the probes that pass
# both. A moment that no probe passes is an empty part of the union, and a
# union of no probe is refused. An `intent` variant runs every probe.
# `plan` prints one `selected: <probe>` line for each probe of a hooks layer
# that the stage runs, and `launch` writes each other probe as an exclude on
# `<try>/spec`. A stage runs its selection at the stage's repetitions, the
# rule that a line which leaves probes out runs the others at the same
# count (`tools/probe/additive-dry-run.sh`). So a `read` variant over 8 of 22
# probes confirms at 7 x 8 = 56 sessions and not 145. HW-DR-0112 does not
# say which count a subset holds, and that is the owner's question. A base
# runs the whole set, because one base serves every layer of it.
#
# **Rank on use, then pass rate.** `result` counts, for each try, the
# sessions that used the open layer with `tools/probe/usage.py`, from the
# session files the batch already holds: a `Skill` call that names a shipped
# skill, a path the hook injected that the session then opened, or a call to
# an MCP tool. `rank` orders the screened variants of the round by the share
# of their sessions that used the layer, and then by their pass rate.
#
# **No gain in use, no confirmation.** A variant is confirmed only when it is
# the top of the rank, and only when its share of sessions that used the
# layer is above the best so far's, read at the best so far's own
# measurement at the confirmation's count. A variant that does not raise the
# use is refused, whatever its pass rate. When the refused variant is the top
# of the rank, no variant of the round raises the use, so `launch` closes the
# round with nothing kept.
#
# **The keep rule reads the confirmation alone.** `result` of a confirmation
# compares its transcript with the best so far's, both at the confirmation's
# count, and keeps the variant only when the whole 95% interval of the
# difference is above zero. A screen or a best so far is never kept, and a
# best so far whose sessions are not the confirmation's count is refused.
#
# **The oracle's verdict is the pass of a probe that has one** (#1875). Each
# `result.md`, of every stage, holds one line for each probe of the try, in
# one of two forms:
#
#     probe <id> (<action>): <p> of <n> by the oracle <name> (grader <x> of <g>)
#     probe <id> (<action>): <x> of <g> by the grader
#
# The first form is for a probe whose plan line is `oracle: command:<name>`.
# `<n>` is the number of the probe's sessions under `<try>/a/sessions/` that
# recorded, with `status` 0, and hold an `oracles` file, which
# `tools/probe/campaign.sh` writes. The probe of a session is the one its
# `record.argv` names. `<p>` is the number of those whose line for `<name>`
# reads `pass`. `<x>/<g>` is the probe's own row of `headwater probe compare`
# of the transcript against itself, which is the grader's count. The line
# ends ` disagree` when the two counts differ: the recorder writes each
# oracle's verdict from the same findings the grader reads, so a
# disagreement is a defect to look at. The second form is for every other
# probe, including the probes that grade against a rule. The action and the
# oracle come from `headwater probe plan` of the round's set in the try's
# scenario, and never from a probe's YAML. A try whose sessions hold no
# `oracles` file at all was recorded before #1875, prints
# `oracle: not recorded (the batch predates #1875)`, and shows the grader's
# count for every probe. These lines are not in `ledger.jsonl`.
#
# **Rounds and order.** A layer takes at most `tries` rounds, and it stops
# after `stop_after_empty` rounds in a row with nothing kept. `plan` refuses
# a round of a layer that `order` does not name, a round of a later layer
# while an earlier one is open (neither stopped nor at its most rounds), a
# round of a layer that stopped or took its most rounds, and a round that
# the ledger has closed.
#
# Every figure is read from the declaration through `tools/probe/declared.py`
# and from the files under `<root>`, and this script stores no total.
#
# ## The two files of the round
#
# - `<root>/env` holds the facts of the host, one `KEY=value` per line:
#   `HEADWATER_DRIVE_CLAUDE_VERSIONS`, `CARGO_TARGET_DIR` and `TMPDIR`, which
#   `launch` and `relaunch` need and export to the driver, and optionally
#   `TRY_FLOOR_ROOT_KB` and `TRY_FLOOR_TRY_KB`, the disk floors of `wait`. A
#   line that sets any other key, or a line that is not a comment, a blank
#   or `KEY=value`, exits 2. The file is read and never run.
# - `<root>/base-pin` is one line, `<scenario> <sha>`: the base scenario of
#   the open layer and the commit at which that base was measured. The
#   declaration cannot carry it, because `declared.py` refuses a seventh
#   field of `open`. `result` of a `base` try writes it. A pin that a person
#   writes by hand still names the commit of the base check, but no `base`
#   event backs it, so it does not clear the gate of a confirmation.
#
# A try root that `launch` makes also holds `stage`, which `result` reads.
#
# The launch constants (model, harness version, session cents, parallel
# count, overshoot margin, seed) are in `tools/probe/additive-launch.env`,
# which this script sources.
#
# ## The verbs
#
# - `spent <root>` prints the sum of every `cost` and every `spent-before` at
#   `<root>/<try>/<batch>/sessions/<job>/`, over every try and every batch,
#   dot-named directories included, at that depth and no other. A file there
#   that does not hold a whole number of cents exits 2, because a total that
#   skips it is too low. It then prints the two parts of that total, each
#   against its ceiling: the bases, `base-1` and the earliest
#   `<layer>-r<n>-best` of each layer, against `base_ceiling_cents`, and
#   every other try root against `ceiling_cents`. The part of a try root is
#   read from its name, so no total is stored (#1838).
# - `status <root>` prints each try root with what it spent, its markers and
#   whether its driver runs, and then the total, the approval, the two parts
#   and the open layer.
# - `plan <root> [<stage>]` exits 0 when the stage of the open layer's round
#   may launch, or with no stage when the round may start, and prints the
#   figures it read. It refuses with 5 when no layer is open, before it
#   builds anything. It refuses with 6 a round the rules of rounds and order
#   above refuse, and a stage those of the stages refuse, before it builds
#   anything. It refuses with 5 when `spent <root>` plus the open layer's
#   `cap_cents` is above the tuning round's `approved_cents`, when the tries'
#   part of it plus the price of what is left of the round is above
#   `ceiling_cents`, when the bases' part plus the price of the base left is
#   above `base_ceiling_cents`, and when the price of the stage is above
#   `cap_cents`, which the driver would stop it at. Equal passes. The price
#   of a stage is its sessions times the additive tier's
#   `session_cost_cents`. What is left of the round is the variants from `k`
#   on at the screen's count, the confirmation, and a best so far that is
#   due and is not the layer's first, or the whole round with no stage. The
#   base left is the layer's first best so far where it is due, or `base-1`
#   for a `base`. A `base` past `base-1` is a try. A `base` stops here: it
#   builds no tree. Otherwise `plan` then builds the tree of the open scenario's base at the pin of
#   `<root>/base-pin` and at `HEAD`, each with `campaign.sh --scenario-tree`,
#   prints every path by which the two differ, and exits 4 when one matches
#   a pattern of `base_invalidators` and no `accept` event of
#   `<root>/ledger.jsonl` covers it. In a pattern, `*` and `**` both match
#   any string, `/` included. An `accept` event covers a path when its
#   `base` and `pin` are the two fields of `<root>/base-pin` and it lists the
#   path with the digest the path has now in the tree at `HEAD`, and `plan`
#   prints `accepted: <path> (<ruling>)` for it. A path that changes again
#   after its acceptance has another digest, so the base is stale again. For
#   a confirmation, it exits 4 unless a `base` event of the ledger at the
#   confirmation's count has the scenario and the pin of `<root>/base-pin`,
#   and no acceptance covers a path under `engine/crates/*/src/**`, because a
#   change to the engine can change what `headwater route` prints to every
#   session of the base scenario. Nothing here reads which change does, so
#   the gate errs toward stale: `launch <root> base` at the variant's commit
#   measures the base again, and its `result` writes the new pin. Before
#   either exit 4, `plan` prices that re-measure under the ceiling it is
#   charged to, and exits 5 instead when the spend refuses it. A refusal of
#   spend is never 4, so a wrapper can tell a stale base from a refusal. A screen then builds the open scenario's tree at `HEAD` and
#   refuses with 6 a tree another variant of the round has.
# - `launch <root> <stage>` runs `plan` with the stage, and exits with its
#   status when it is not 0. It then makes the try root of the stage, writes
#   the spec `additive scenario-<n> tuning` there, outside the checkout, and
#   the stage, and starts
#   `campaign-drive.sh` detached, from this checkout and pinned at its
#   `HEAD`, with the constants of `additive-launch.env`, the open layer's
#   `cap_cents` as `--cap-cents`, and `--on-canary` running `canary` on the
#   try. It writes `driver.argv`, `driver.pid` and `launched` in the try
#   root. A try root that exists already exits 6, and nothing is started. A
#   confirmation from a checkout that is not at the `--pin` of its variant's
#   `driver.argv` exits 6.
#   Before it makes the try root, it refuses with 6 the four states of the
#   checkout that the driver's pin check refuses: `HEAD` on a branch, a pin
#   that is not on `origin/main` or on a branch under `origin/tune/`, a
#   dirty tree, and, for `relaunch`, a
#   `HEAD` that is no longer the pin of `driver.argv`. The driver checks the
#   harness and the token itself.
# - `rank <root>` prints the best so far of the open layer, with its use, and
#   then each screened variant of the open round that `result` recorded, in
#   rank order: its use, its pass rate, and `gain` or `no gain` in use over
#   the best so far, or `-` where no best so far is measured.
# - `relaunch <root> <try>` starts the driver again on a try root that
#   `launch` made, with the same arguments, read from `driver.argv`. It
#   exits 6 when the try root does not exist, when its driver runs, or when
#   `done`, `capped`, `canary-failed` or `stop` is present. It never writes
#   `a.done`, so a capped batch stays capped until a person rules on it.
#   `usage-gated` does not refuse a relaunch (#1876): the driver reads the
#   usage gate first, removes the marker below the threshold and goes on,
#   with `campaign.sh` skipping every job already recorded, and above it
#   writes the marker again and exits 13 with no invocation. Each launch
#   and relaunch hands the driver the three `TRY_USAGE_*` values of the
#   current `additive-launch.env`.
# - `canary <root> <try>` reads the eight records of the canary, the jobs
#   of `<try>/a/jobs` that have a `status`, and writes `<try>/canary-passed`
#   only when there are eight and each one has `status` 0, a cost of at most
#   `TRY_SESSION_CENTS + TRY_OVERSHOOT_CENTS`, and the sentences of
#   `tools/probe/probe-record.sh` that state 0 calls to a web tool, 0 paths
#   outside its workspace and no refused provider fetch, and that the intent
#   hook was live when the scenario of `<try>/spec` holds the `hooks` layer
#   and was not live otherwise. A count of 1 is written in the singular
#   (`1 path`, `1 call`), so the reader matches the sentence of 0 and treats
#   every other sentence, the "not counted" one included, as offending.
#   Otherwise it writes `<try>/canary-needs-reader` with one line for each
#   offence, and exits 7. With `canary-passed` present it reads nothing and
#   exits 0, because a person who read the records may have written it.
# - `wait <root> <try>` is the condition for `tools/run/wait-for.sh`. It
#   exits 0 and prints one line when a person must read something or the
#   driver has stopped: `disk-floor`, `done`, `canary-failed`, `capped`,
#   `stop`, `canary-needs-reader`, `canary-recorded`, `usage-gated` or
#   `driver-exited`. It exits 1 while the driver runs and needs nobody, and
#   so while a gated invocation lets its sessions in flight finish. A driver
#   that stopped on the usage gate prints `usage-gated:` and the line of
#   `<try>/usage-gated`, and the first `wait` to read that marker appends
#   one `gate` event to the ledger. When the free space
#   under `/` is below `TRY_FLOOR_ROOT_KB`, or on the file system of `<root>`
#   below `TRY_FLOOR_TRY_KB`, it writes `<try>/stop` first, which the driver
#   reads within a minute.
# - `result <root> <try>` turns a try that the driver finished into its
#   record, in seven steps:
#   1. It reads the stage of the try from `<try>/stage`, and a try root with
#      no stage, or with a name that is not `<layer>-r<n>-v<k>`,
#      `<layer>-r<n>-best`, `<layer>-r<n>-confirm` or `base-<k>`, exits 2. A try that has
#      a `result.md` exits 6, so no try is recorded twice.
#   2. When `<try>/done` is absent, it appends a `halt` event with the try's
#      spend and markers to `<root>/ledger.jsonl` and exits 7. It prints no
#      `keep:` line and writes no `result.md` and no transcript. A try that is
#      `capped`, stopped or whose canary failed has no `done`.
#   3. It sums the try's session files as `status` does, and exits 2 when the
#      sum is not the cents of the last `all=<n>/<cap>` line that the driver
#      wrote to `<try>/drive.out`, and names both figures. It exits 2 when the
#      jobs of `<try>/a/jobs` are not the stage's count, so a batch of a
#      screen is never recorded as a confirmation. It counts the use of the
#      layer with `tools/probe/usage.py`, except for a base, which adds no
#      layer.
#   4. It runs `campaign.sh --out <try>/a --assemble`, and writes the
#      transcript with `headwater new probe_transcript --facet tier=additive
#      --facet arm=scenario-<n> --facet round=tuning` in the checkout, with
#      the assembled identity and events blocks as its two sections. It
#      writes the path in `<try>/transcript`, and a later run that finds that
#      file reuses it. The narrative paragraphs stay a person's job.
#   5. It runs `headwater probe compare <new> <control>` and reads the
#      `pooled:` line. The base row's control is the transcript of the `base`
#      event at the confirmation's count that `<root>/base-pin` names, and
#      where no event backs the pin, the one committed transcript with
#      `tier: additive`, `round: screening` and the arm of the base scenario.
#      It is reported, and the keep rule does not read it. A base compares
#      with the base it measures again, takes its pass rate from the treated
#      side, appends a `base` event with its scenario, the `--pin` of its
#      `driver.argv`, its count, its pass rate, its spend and its transcript,
#      writes `<root>/base-pin` as `<scenario> <pin>`, and writes `result.md`
#      with `keep: never`. Zero or
#      more than one match exits 2. A screen takes its pass rate from the
#      treated side of the base row, and it and a best so far stop here: each
#      appends its event, `screen` or `best`, with its use, and writes
#      `result.md` with `keep: never`. A confirmation compares again for the
#      keep row, whose control is the best so far: the transcript of the
#      latest event of the layer that is a `best` or a `result` of a
#      confirmation with `kept` true. None, or one whose batch did not plan
#      the confirmation's count, exits 2.
#   6. The keep rule is the owner's of 2026-10-20 on #1777 (ruling 7): keep a
#      tuning change only when the whole uncertainty range of the difference
#      against the best so far sits above zero. So the try is kept when the
#      printed low bound of the keep row is above zero, and a printed `+0.0`
#      or `-0.0` is not. The direction is `up`, `down` or `flat` by the sign
#      of the printed difference. A keep row with no interval, because a side
#      graded no session, exits 2 and keeps nothing. The rule has no branch
#      for a try that held while the text in front of the model shrank:
#      ruling 7 names the interval rule alone.
#   7. It writes `<try>/result.md` with the spend, the use, both pooled
#      lines, `keep: kept` or `keep: not kept`, the direction, the rule and
#      the transcript, and appends a `result` event and a `round` event that
#      closes the round to the ledger. It exits 0 whether the variant is kept
#      or not, because the verdict is in the record.
# - `ledger <root>` prints one line for each event of `<root>/ledger.jsonl`,
#   `<event>: <key>=<value> ...`, and then the round's total as `spent`
#   counts it. A line that does not read as JSON exits 2. With no ledger it
#   prints `no events`.
# - `accept <root> <ruling-url>` records that the owner accepted the paths
#   by which the base differs. It builds the two trees as `plan` does, at the
#   pin of `<root>/base-pin` and at `HEAD`, and appends an `accept` event with
#   the base, the pin, `HEAD`, the ruling and, for each path that
#   invalidates, the sha256 of the path in the tree at `HEAD`, or `absent`. It
#   prints each path. With no path that invalidates, it prints `nothing to
#   accept` and records nothing. The ruling is the URL of a comment on an
#   issue, `https://github.com/<owner>/<repo>/issues/<n>#issuecomment-<id>`,
#   or of a pull request, `.../pull/<n>`, and any other text exits 2. The
#   ledger is outside git, so the ruling's URL is the durable record, and a
#   committed transcript cites it.
#
# `<root>/ledger.jsonl` holds one JSON object for each event. `result`
# appends `screen`, `best`, `base`, `result`, `round` and `halt`, `accept` appends
# `accept`, `wait` appends `gate` for each stop of the usage gate, and `launch` appends a `round` with `kept` false when it refuses
# the top variant of a round for no gain in use.
#
# ## The base tree, and `campaign.sh`
#
# Both trees are built by this checkout's `campaign.sh`, with this
# checkout's engine and tools, from `git archive` of each commit. So the
# engine binary in a tree that holds the `engine` layer is the same file in
# both trees and never differs on its own: the engine's sources stand for
# it, under `engine/crates/*/src/**`.
#
# The owner ruled on 2026-10-20 (#1821) that an edit to `campaign.sh` makes a
# base stale only when it touches the code that builds, runs or grades a
# session, and that the gate and refusal checks are exempt. So when
# `tools/probe/campaign.sh` differs, `plan` reads `diff -U0` of the two
# copies. A removed line is neutral when it is a comment, a blank, or inside
# a region of the older copy marked `# base-neutral: begin <reason>` and
# `# base-neutral: end`, and an added line the same against the newer copy.
# `campaign.sh` makes the base stale only when one line is not neutral, and
# `plan` prints that line. A region must close before the line
# `for tool in git jq tar awk cargo claude bwrap`, where the batch's first
# check of its tools runs. A copy with a region that reaches it, or an
# unclosed region, exits 2. `tools/probe/seal.sh` and `tools/probe/leak.py`
# are not on the list (owner, #1819).
#
# `HEADWATER_TRY_DECLARATION`, `HEADWATER_TRY_DRIVER`,
# `HEADWATER_TRY_CAMPAIGN`, `HEADWATER_TRY_REPO` and `HEADWATER_TRY_ENGINE`
# name the declaration, the driver, the `campaign.sh` it runs, the checkout
# the driver runs from and the engine binary of `result`, by default
# `engine/target/dev-release/headwater` in that checkout, so that
# `tools/probe/try-fixtures.sh` runs every verb with stubs. It needs `git`,
# `awk`, `python3` with PyYAML, `diff`, `sha256sum` and `setsid`.
# `HEADWATER_TRY_USAGE` names the counter of use, by default
# `tools/probe/usage.py` beside this script.
#
# ## Exit status
#
#   0   the verb did what it states: `canary-passed` is written, the plan
#       passes, the driver is started, `rank` printed, `wait` met its condition, `result`
#       recorded a try kept or not kept, `ledger` printed, or `accept`
#       recorded or had nothing to accept
#   1   `wait` alone: the driver runs and needs no reader
#   2   a usage error, or a file it reads is missing or does not read:
#       `<root>`, `<root>/env`, `<root>/base-pin`, `driver.argv`, a cost, a
#       marked region of `campaign.sh`, the declaration, a line of
#       `ledger.jsonl`, a `drive.out` whose `all=` line is not the try's
#       spend, an assembly that refused, a control transcript that is not
#       exactly one, a keep row with no interval, or a ruling that is not a
#       GitHub comment or pull request URL
#   3   a tool is missing, a tree did not build, or the engine is missing or
#       wrote no transcript
#   4   the base is stale: a path by which the two trees differ matches
#       `base_invalidators`, and no `accept` event of `ledger.jsonl` covers
#       it at its current digest, or, for a confirmation, no `base` event at
#       its count backs the pin, or the engine's source differs from the
#       pin. Nothing else exits 4
#   5   the spend is refused: no layer is open, what the round spent plus
#       the open layer's cap is above the round's approval, what the tries
#       or the bases spent plus the price of what is left under that ceiling
#       is above it, or the price of the stage is above the cap
#   6   the state of the try refuses the verb: `launch` on a try root that
#       exists, `relaunch` on a try that is absent, running, done, capped,
#       stopped or whose canary failed, `launch` or `relaunch` on a
#       checkout the driver's pin check refuses, `canary` before
#       `canary-recorded`, or `result` on a try that has a `result.md`; or
#       the rules of rounds, order and stages refuse the round or the stage
#       of `plan` or `launch`
#   7   a person must read the try: `canary` wrote `canary-needs-reader`, or
#       `result` found no `done` and appended a `halt` event

set -u

here=$(cd "$(dirname "$0")" && pwd -P)
repo=$(cd "$here/../.." && pwd -P)
self=$here/try.sh
declaration=${HEADWATER_TRY_DECLARATION:-$repo/.headwater/probe.yml}
driver=${HEADWATER_TRY_DRIVER:-$here/campaign-drive.sh}
campaign=${HEADWATER_TRY_CAMPAIGN:-$here/campaign.sh}
checkout=${HEADWATER_TRY_REPO:-$repo}
engine=${HEADWATER_TRY_ENGINE:-$checkout/engine/target/dev-release/headwater}
usage_counter=${HEADWATER_TRY_USAGE:-$here/usage.py}

usage() {
    echo "usage: sh tools/probe/try.sh status|spent|rank|ledger <root>, plan <root> [<stage>], launch <root> <stage>, relaunch|canary|wait|result <root> <try>, or accept <root> <ruling-url>, where <stage> is screen <k>, best, confirm <variant> or base" >&2
    exit 2
}
say() { printf 'try: %s\n' "$1" >&2; }

[ $# -ge 2 ] || usage
verb=$1 root=$2
try_name=
ruling=
stage_kind=
stage_arg=
case $verb in
    status | spent | rank | ledger) [ $# = 2 ] || usage ;;
    plan) [ $# -le 4 ] || usage; stage_kind=${3:-} stage_arg=${4:-} ;;
    launch) [ $# -ge 3 ] && [ $# -le 4 ] || usage; stage_kind=$3 stage_arg=${4:-} ;;
    relaunch | canary | wait | result) [ $# -le 3 ] || usage; try_name=${3:-} ;;
    accept) [ $# = 3 ] || usage; ruling=$3 ;;
    *) usage ;;
esac
case $stage_kind in
    '') ;;
    screen)
        case $stage_arg in
            '' | *[!0-9]* | 0*) say "a screen names its variant, a whole number from 1: screen <k>."; exit 2 ;;
        esac
        ;;
    best | base) [ -z "$stage_arg" ] || usage ;;
    confirm)
        case $stage_arg in
            '' | */* | . | ..) say "a confirmation names the try root of a screened variant under the round root: confirm <variant>."; exit 2 ;;
        esac
        ;;
    *) say "\`$stage_kind\` is not a stage. A stage is screen <k>, best, confirm <variant> or base."; exit 2 ;;
esac
[ -d "$root" ] || { say "no round root at $root."; exit 2; }
root=$(cd "$root" && pwd -P)

# shellcheck source=tools/probe/additive-launch.env
. "$here/additive-launch.env"

# ------------------------------------------------------------ the inputs

# declared <mode>: the printer of the declaration. Its 3 is a missing
# PyYAML, and anything else it refuses is a declaration that does not read.
declared() {
    command -v python3 >/dev/null 2>&1 || { say "python3 is not on the path."; exit 3; }
    declared_out=$(python3 "$here/declared.py" "$declaration" "$1")
    declared_status=$?
    case $declared_status in
        0) printf '%s\n' "$declared_out" ;;
        3) exit 3 ;;
        *) exit 2 ;;
    esac
}

# read_env: the host facts of `<root>/env`, set and exported. `need` is 1
# where the three host facts are required.
read_env() {
    need=$1
    env_file=$root/env
    if [ ! -f "$env_file" ]; then
        [ "$need" = 1 ] || return 0
        say "no $env_file. Write HEADWATER_DRIVE_CLAUDE_VERSIONS, CARGO_TARGET_DIR and TMPDIR there, one KEY=value per line."
        exit 2
    fi
    env_lines=$(awk '
        /^[ \t]*(#.*)?$/ { next }
        /^(HEADWATER_DRIVE_CLAUDE_VERSIONS|CARGO_TARGET_DIR|TMPDIR|TRY_FLOOR_ROOT_KB|TRY_FLOOR_TRY_KB)=/ { print; next }
        { printf "bad %d %s\n", NR, $0 }
    ' "$env_file")
    if printf '%s\n' "$env_lines" | grep -q '^bad '; then
        printf '%s\n' "$env_lines" | awk -v f="$env_file" '/^bad / { n = $2; sub(/^bad [0-9]+ /, ""); print "try: line " n " of " f " is not KEY=value of a key it reads: " $0 }' >&2
        exit 2
    fi
    while IFS= read -r env_line; do
        [ -n "$env_line" ] || continue
        env_key=${env_line%%=*}
        env_value=${env_line#*=}
        case $env_key in
            TRY_FLOOR_ROOT_KB) TRY_FLOOR_ROOT_KB=$env_value ;;
            TRY_FLOOR_TRY_KB) TRY_FLOOR_TRY_KB=$env_value ;;
            HEADWATER_DRIVE_CLAUDE_VERSIONS) HEADWATER_DRIVE_CLAUDE_VERSIONS=$env_value; export HEADWATER_DRIVE_CLAUDE_VERSIONS ;;
            CARGO_TARGET_DIR) CARGO_TARGET_DIR=$env_value; export CARGO_TARGET_DIR ;;
            TMPDIR) TMPDIR=$env_value; export TMPDIR ;;
        esac
    done <<EOF
$env_lines
EOF
    if [ "$need" = 1 ]; then
        for env_key in HEADWATER_DRIVE_CLAUDE_VERSIONS CARGO_TARGET_DIR TMPDIR; do
            grep -q "^$env_key=." "$env_file" || { say "$env_file sets no $env_key."; exit 2; }
        done
    fi
}

# open_fields: `rounds.tuning.open`, as `layer scenario try cap`, with the
# hooks layer's `moment` after them, or `none`.
open_fields() {
    open_line=$(declared open) || exit $?
    if [ "$open_line" = none ]; then
        echo none
        return
    fi
    printf '%s\n' "$open_line" | awk '{ print $1, $2, $3, $4, $7 }'
}

# select_probes: the probes of the tuning set that the stage runs, in the
# order of the set, in `$selected`, their count in `$selected_n`, and the
# others in `$excluded`. Every probe, but for a hook variant of a moment
# other than intent, which runs the probes of its moment and those of row 11
# (#1875).
select_probes() {
    set_members=$(declared tuning_set) || exit $?
    selected=$set_members
    if [ "$open_layer" = hooks ] && [ "$open_moment" != intent ] && [ "$stage_kind" != base ]; then
        set_name=$(declared rounds | awk '$1 == "tuning" { print $2 }') || exit $?
        [ -x "$engine" ] || { say "no engine at $engine, which selects the probes of a hook variant. Build it, or name it with HEADWATER_TRY_ENGINE."; exit 3; }
        picked=
        for select_flag in "--moment $open_moment" "--action ungoverned_control"; do
            # shellcheck disable=SC2086
            if ! select_out=$("$engine" probe plan --root "$checkout" --tier additive --arm "scenario-$open_scenario" \
                --set "$set_name" $select_flag 2> /dev/null); then
                say "headwater probe plan $select_flag did not run, so the probes of the $open_moment variant are not known."
                exit 3
            fi
            picked="$picked $(printf '%s\n' "$select_out" | sed -n 's/^- \(HW-PROBE-[^ ]*\) (.*/\1/p' | tr '\n' ' ')"
        done
        selected=$(printf '%s\n' "$set_members" | while IFS= read -r member; do
            case " $picked " in *" $member "*) printf '%s\n' "$member" ;; esac
        done)
    fi
    selected_n=$(printf '%s\n' "$selected" | awk 'NF' | wc -l | tr -d ' ')
    excluded=$(printf '%s\n' "$set_members" | while IFS= read -r member; do
        case "
$selected
" in *"
$member
"*) ;; *) printf '%s ' "$member" ;; esac
    done)
    excluded=${excluded% }
    if [ "$selected_n" = 0 ]; then
        say "the $open_moment variant of the hooks layer selects no tuning probe: none passes --moment $open_moment, and none has the action ungoverned_control."
        exit 6
    fi
}

# draw_lines: the fixed draw of a screen, one `draw: <probe> <n>` line for
# each session, in the order of the set (#1875).
draw_lines() {
    printf '%s\n' "$selected" | while IFS= read -r member; do
        [ -n "$member" ] || continue
        member_count=$(sh "$here/phrasing.sh" count "$checkout" "$member") || exit 3
        repetition=1
        while [ "$repetition" -le "$t_screen_reps" ]; do
            printf 'draw: %s %s\n' "$member" "$(sh "$here/phrasing.sh" draw "$TRY_SEED" "$open_layer" "$open_try" "$member" "$repetition" "$member_count")"
            repetition=$((repetition + 1))
        done
    done
}

# resolve_try: the try root a verb acts on, in `$try`.
resolve_try() {
    if [ -n "$try_name" ]; then
        case $try_name in
            */* | . | ..) say "a try is the name of a directory under $root, not \`$try_name\`."; exit 2 ;;
        esac
        try=$root/$try_name
        return
    fi
    say "a round runs several tries, so name the try: sh tools/probe/try.sh $verb $root <try>."
    exit 2
}

# tuning_read: the figures of the tuning round's rounds (#1838), each from
# `declared.py tuning`, which computes the count of each stage.
tuning_read() {
    tuning_out=$(declared tuning) || exit $?
    if [ -z "$tuning_out" ]; then
        say "the tuning round declares none of screen, confirm, variants, stop_after_empty, order, ceiling_cents and base_ceiling_cents, so it has no rounds to run."
        exit 2
    fi
    t_screen=$(printf '%s\n' "$tuning_out" | awk '$1 == "screen" { print $4 }')
    t_screen_reps=$(printf '%s\n' "$tuning_out" | awk '$1 == "screen" { print $3 }')
    t_confirm=$(printf '%s\n' "$tuning_out" | awk '$1 == "confirm" { print $4 }')
    t_confirm_reps=$(printf '%s\n' "$tuning_out" | awk '$1 == "confirm" { print $3 }')
    t_variants=$(printf '%s\n' "$tuning_out" | awk '$1 == "variants" { print $2 }')
    t_rounds=$(printf '%s\n' "$tuning_out" | awk '$1 == "rounds" { print $2 }')
    t_stop=$(printf '%s\n' "$tuning_out" | awk '$1 == "stop_after_empty" { print $2 }')
    t_order=$(printf '%s\n' "$tuning_out" | awk '$1 == "order" { $1 = ""; sub(/^ /, ""); print }')
    t_ceiling=$(printf '%s\n' "$tuning_out" | awk '$1 == "ceiling_cents" { print $2 }')
    t_base_ceiling=$(printf '%s\n' "$tuning_out" | awk '$1 == "base_ceiling_cents" { print $2 }')
    t_rate=$(printf '%s\n' "$tuning_out" | awk '$1 == "rate" { print $2 }')
}

# ------------------------------------------------------------ the spend

# spent_round: every cost and spent-before at `<root>/<try>/<batch>/sessions/<job>/`.
spent_round() {
    spent_files=$(mktemp "${TMPDIR:-/tmp}/try-spent.XXXXXX") || exit 3
    for spent_try in "$root"/*/ "$root"/.[!.]*/ "$root"/..?*/; do
        [ -d "$spent_try" ] || continue
        for spent_batch in "$spent_try"*/ "$spent_try".[!.]*/ "$spent_try"..?*/; do
            [ -d "${spent_batch}sessions" ] || continue
            find "${spent_batch}sessions" -mindepth 2 -maxdepth 2 -type f \( -name cost -o -name spent-before \) -print >> "$spent_files"
        done
    done
    spent_sum=$(awk '
        {
            line = ""
            if ((getline line < $0) <= 0) line = ""
            close($0)
            if (line !~ /^[0-9]+$/) { bad = bad "\n  " $0 " holds \"" line "\""; next }
            s += line
        }
        END {
            if (bad != "") { printf "bad%s\n", bad; exit 1 }
            print s + 0
        }
    ' "$spent_files")
    spent_status=$?
    rm -f "$spent_files"
    if [ "$spent_status" != 0 ]; then
        say "a file of what a session spent does not hold a whole number of cents, so the round's total cannot be counted:"
        printf '%s\n' "$spent_sum" | sed 1d >&2
        exit 2
    fi
    printf '%s\n' "$spent_sum"
}

# try_spent <try>: the same sum, over one try root. awk reads each file
# as its own records, so a cost with no final newline does not join the
# next file's number, as cat into one awk did (30 and 40 read as 3040).
try_spent() {
    try_spent_sum=0
    for try_batch in "$1"/*/ "$1"/.[!.]*/ "$1"/..?*/; do
        [ -d "${try_batch}sessions" ] || continue
        try_spent_sum=$((try_spent_sum + $(find "${try_batch}sessions" -mindepth 2 -maxdepth 2 -type f \( -name cost -o -name spent-before \) -exec awk '{ s += $1 } END { print s + 0 }' {} + 2>/dev/null | awk '{ s += $1 } END { print s + 0 }')))
    done
    echo "$try_spent_sum"
}

# first_best <layer>: the try root of the earliest best so far of the layer
# under the round root, by its round, or nothing.
first_best() {
    for first_dir in "$root/$1"-r*-best/; do
        [ -d "$first_dir" ] || continue
        first_dir=${first_dir%/}
        printf '%s\n' "${first_dir##*/}"
    done | sed -n "s/^$1-r\([1-9][0-9]*\)-best\$/\1 &/p" | sort -n | awk 'NR == 1 { print $2 }'
}

# charge_of <try>: the ceiling a try root is charged to, by its name alone
# (#1838). `base` is the base ceiling, which pays for the bases of #1833:
# `base-1`, the first measurement of the base scenario at the
# confirmation's count, and the earliest best so far of each layer. Every
# other try root, `base-2` and later included, is `tries`, the tuning
# ceiling. A try root that does not exist yet is read as if it did.
charge_of() {
    charge_name=$1
    if [ "$charge_name" = base-1 ]; then
        echo base
        return
    fi
    charge_fields=$(printf '%s\n' "$charge_name" | sed -n 's/^\([a-z][a-z0-9-]*\)-r\([1-9][0-9]*\)-best$/\1 \2/p')
    if [ -n "$charge_fields" ]; then
        charge_first=$(first_best "${charge_fields% *}")
        charge_first_n=${charge_first%-best}
        charge_first_n=${charge_first_n##*-r}
        if [ -z "$charge_first" ] || [ "${charge_fields#* }" -le "$charge_first_n" ]; then
            echo base
            return
        fi
    fi
    echo tries
}

# spent_split: `<base> <tries>`, what the try roots charged to each ceiling
# spent. It reads the files `spent_round` reads, and it stores no total.
spent_split() {
    split_base=0
    split_tries=0
    for split_try in "$root"/*/ "$root"/.[!.]*/ "$root"/..?*/; do
        [ -d "$split_try" ] || continue
        split_try=${split_try%/}
        split_sum=$(try_spent "$split_try")
        if [ "$(charge_of "${split_try##*/}")" = base ]; then
            split_base=$((split_base + split_sum))
        else
            split_tries=$((split_tries + split_sum))
        fi
    done
    echo "$split_base $split_tries"
}

# ceilings: `<tries> <base>`, the two ceilings of the tuning round, or `- -`
# where it declares no rounds.
ceilings() {
    ceilings_out=$(declared tuning) || exit $?
    printf '%s\n' "$ceilings_out" | awk '
        $1 == "ceiling_cents" { t = $2 }
        $1 == "base_ceiling_cents" { b = $2 }
        END { print (t == "" ? "-" : t), (b == "" ? "-" : b) }'
}

# ------------------------------------------------------------ the ledger

# ledger_py <mode> <args...>: every read and write of `<root>/ledger.jsonl`.
# A line that does not read as a JSON object exits 2, in every mode.
#   append <key>=<type>:<value>...   append one event. The type is `s` for a
#                                    string, `i` for a whole number, `b` for
#                                    true or false, and `t` for the rows
#                                    `<path>\t<digest>` of standard input
#   print                            one line for each event
#   best <layer>                     the best so far of the layer, as
#                                    `<transcript>\t<used>\t<sessions>\t<event>\t<count>`:
#                                    the latest `best` event, or `result` of a
#                                    confirmation with `kept` true, or nothing
#   rank <layer> <round>             the `screen` events of the round, as
#                                    `<try>\t<used>\t<sessions>\t<passed>\t<graded>`,
#                                    by the share used, then the pass rate
#   layers <rounds> <stop> <layer>...  for each layer, `<layer> <state>
#                                    <closed rounds>`: `open`, `stopped` or
#                                    `exhausted`, from the `round` events
#   cover <base> <pin>               for each row `<path>\t<digest>` of
#                                    standard input, `covered\t<path>\t<ruling>`
#                                    or `uncovered\t<path>`
#   base <scenario> <pin> <count>    the latest `base` event of the scenario at
#                                    that pin and count, `-` for any, as
#                                    `<transcript>\t<pin>\t<count>\t<try>`
# shellcheck disable=SC2016
ledger_program='
import json, sys
mode, ledger = sys.argv[1], sys.argv[2]
args = sys.argv[3:]

def events():
    try:
        with open(ledger, encoding="utf-8") as handle:
            lines = handle.read().splitlines()
    except FileNotFoundError:
        return None
    out = []
    for number, line in enumerate(lines, 1):
        if not line.strip():
            continue
        try:
            event = json.loads(line)
        except ValueError:
            event = None
        if not isinstance(event, dict):
            print("try: line %d of %s does not read as a JSON object: %s" % (number, ledger, line), file=sys.stderr)
            sys.exit(2)
        out.append(event)
    return out

def rows():
    return [row.split("\t", 1) for row in sys.stdin.read().splitlines() if row]

if mode == "append":
    events()
    event = {}
    for arg in args:
        key, typed = arg.split("=", 1)
        kind, value = typed.split(":", 1)
        if kind == "i":
            event[key] = int(value)
        elif kind == "b":
            event[key] = value == "true"
        elif kind == "t":
            event[key] = {path: digest for path, digest in rows()}
        else:
            event[key] = value
    with open(ledger, "a", encoding="utf-8") as handle:
        handle.write(json.dumps(event) + "\n")
elif mode == "print":
    found = events()
    if not found:
        print("no events")
    for event in found or []:
        rest = " ".join("%s=%s" % (key, value if isinstance(value, str) else json.dumps(value))
                        for key, value in event.items() if key != "event")
        print("%s: %s" % (event.get("event", "unknown"), rest))
elif mode == "best":
    best = None
    for event in events() or []:
        if event.get("layer") != args[0]:
            continue
        if event.get("event") == "best" or (event.get("event") == "result" and event.get("stage") == "confirm"
                                            and event.get("kept") is True):
            best = event
    if best:
        print("\t".join(str(best.get(key, "")) for key in ("transcript", "used", "sessions", "event", "count")))
elif mode == "rank":
    from fractions import Fraction
    rows = [event for event in events() or [] if event.get("event") == "screen"
            and event.get("layer") == args[0] and str(event.get("round")) == args[1]]
    def share(top, bottom):
        return Fraction(int(top), int(bottom)) if int(bottom) > 0 else Fraction(0)
    rows.sort(key=lambda e: (-share(e.get("used", 0), e.get("sessions", 0)),
                             -share(e.get("passed", 0), e.get("graded", 0)), str(e.get("try"))))
    for event in rows:
        print("\t".join(str(event.get(key, "")) for key in ("try", "used", "sessions", "passed", "graded")))
elif mode == "layers":
    most, stop = int(args[0]), int(args[1])
    closed = {layer: {} for layer in args[2:]}
    for event in events() or []:
        if event.get("event") == "round" and event.get("layer") in closed:
            kept = closed[event["layer"]]
            number = int(event.get("round"))
            kept[number] = kept.get(number, False) or event.get("kept") is True
    for layer in args[2:]:
        kept = closed[layer]
        numbers = sorted(kept)
        recent = [kept[number] for number in numbers][-stop:]
        if len(recent) == stop and not any(recent):
            state = "stopped"
        elif len(numbers) >= most:
            state = "exhausted"
        else:
            state = "open"
        print(layer, state, ",".join(str(number) for number in numbers) or "-")
elif mode == "base":
    found = None
    for event in events() or []:
        if event.get("event") == "base" and str(event.get("scenario")) == args[0] \
                and args[1] in ("-", event.get("pin")) and args[2] in ("-", str(event.get("count"))):
            found = event
    if found:
        print("\t".join(str(found.get(key, "")) for key in ("transcript", "pin", "count", "try")))
elif mode == "cover":
    accepted = {}
    for event in events() or []:
        if event.get("event") == "accept" and str(event.get("base")) == args[0] \
                and event.get("pin") == args[1] and isinstance(event.get("paths"), dict):
            for path, digest in event["paths"].items():
                accepted.setdefault(path, {})[digest] = event.get("ruling", "")
    for path, digest in rows():
        ruling = accepted.get(path, {}).get(digest)
        print("covered\t%s\t%s" % (path, ruling) if ruling is not None else "uncovered\t%s" % path)
'
ledger_py() {
    command -v python3 >/dev/null 2>&1 || { say "python3 is not on the path."; exit 3; }
    ledger_mode=$1
    shift
    python3 -c "$ledger_program" "$ledger_mode" "$root/ledger.jsonl" "$@" || exit 2
}

now() { date -u +%Y-%m-%dT%H:%M:%SZ; }

# read_pin: the two fields of `<root>/base-pin`, in `pin_scenario` and
# `pin_sha`.
read_pin() {
    pin_file=$root/base-pin
    [ -f "$pin_file" ] || { say "no $pin_file. Write one line there: <scenario> <sha>, the base scenario and the commit it was measured at."; exit 2; }
    # shellcheck disable=SC2046
    set -- $(head -1 "$pin_file")
    [ $# = 2 ] || { say "$pin_file reads \`$(head -1 "$pin_file")\`, and it must be \`<scenario> <sha>\`."; exit 2; }
    pin_scenario=$1 pin_sha=$2
    case $pin_sha in
        *[!0-9a-f]*) say "the pin in $pin_file is not a commit: $pin_sha."; exit 2 ;;
    esac
    [ ${#pin_sha} -ge 7 ] || { say "the pin in $pin_file is not a commit: $pin_sha."; exit 2; }
}

# transcripts <tier> <round> <arm>: the committed transcripts of the checkout
# whose front matter holds the three, as paths under the checkout.
transcripts() {
    for transcript_file in "$checkout"/docs/probe-runs/*.md; do
        [ -f "$transcript_file" ] || continue
        awk -v t="tier: $1" -v r="round: $2" -v a="arm: $3" -v f="${transcript_file#"$checkout"/}" '
            NR == 1 { if ($0 != "---") exit; next }
            $0 == "---" { if (tt && rr && aa) print f; exit }
            $0 == t { tt = 1 }
            $0 == r { rr = 1 }
            $0 == a { aa = 1 }
        ' "$transcript_file"
    done
}

# one_transcript <what> <tier> <round> <arm>: the one transcript that
# matches, and exit 2 on none or on more than one.
one_transcript() {
    one_what=$1
    shift
    one_found=$(transcripts "$@")
    one_count=$(printf '%s' "$one_found" | awk 'END { print NR }')
    if [ "$one_count" != 1 ]; then
        say "the $one_what must be one committed transcript with tier: $1, round: $2 and arm: $3 under $checkout/docs/probe-runs, and $one_count match${one_found:+:}"
        [ -z "$one_found" ] || printf '%s\n' "$one_found" | sed 's/^/  /' >&2
        exit 2
    fi
    printf '%s\n' "$one_found"
}

# compare_row <treated> <control>: the `pooled:` line of `headwater probe
# compare`, both paths under the checkout.
compare_row() {
    compare_out=$("$engine" probe compare --root "$checkout" "$checkout/$1" "$checkout/$2" 2> "${TMPDIR:-/tmp}/try-compare.$$")
    compare_status=$?
    if [ "$compare_status" != 0 ]; then
        say "headwater probe compare $1 $2 exited $compare_status:"
        tail -5 "${TMPDIR:-/tmp}/try-compare.$$" >&2
        rm -f "${TMPDIR:-/tmp}/try-compare.$$"
        exit 2
    fi
    rm -f "${TMPDIR:-/tmp}/try-compare.$$"
    compare_line=$(printf '%s\n' "$compare_out" | grep '^pooled: ' | tail -1)
    [ -n "$compare_line" ] || { say "headwater probe compare $1 $2 printed no pooled: line."; exit 2; }
    printf '%s\n' "$compare_line"
}

# probe_lines <transcript> <scenario>: one line for each probe of the try,
# the oracle's verdict beside the grader's count, in the two forms the header
# states (#1875). The probes are those of the plan that the transcript's
# compare grades, in the plan's order. It reads `$try`.
probe_lines() {
    pl_set=$(declared rounds | awk '$1 == "tuning" { print $2 }') || exit $?
    if ! pl_plan=$("$engine" probe plan --root "$checkout" --tier additive --arm "scenario-$2" --set "$pl_set" 2> /dev/null); then
        say "headwater probe plan of the set $pl_set did not run, so the oracle of each probe is not known."
        exit 2
    fi
    if ! pl_compare=$("$engine" probe compare --root "$checkout" "$checkout/$1" "$checkout/$1" 2> /dev/null); then
        say "headwater probe compare $1 $1 did not run, so the grader's count of each probe is not known."
        exit 2
    fi
    pl_recorded=no
    for pl_file in "$try"/a/sessions/*/oracles; do
        [ -f "$pl_file" ] && pl_recorded=yes
    done
    [ "$pl_recorded" = yes ] || echo "oracle: not recorded (the batch predates #1875)"
    printf '%s\n' "$pl_plan" | awk '
        function flush() { if (id != "") print id, (action == "" ? "-" : action), (oracle == "" ? "-" : oracle) }
        /^- HW-PROBE-/ { flush(); id = $2; action = ""; oracle = ""; next }
        /^- / { flush(); id = ""; next }
        id != "" && /^ +action: / { action = $2 }
        id != "" && /^ +oracle: / { oracle = $2 }
        END { flush() }
    ' | while read -r pl_id pl_action pl_oracle; do
        pl_grader=$(printf '%s\n' "$pl_compare" | awk -v p="$pl_id:" '$1 == p && $3 == "against" { print $2; exit }')
        [ -n "$pl_grader" ] || continue
        pl_x=${pl_grader%%/*} pl_g=${pl_grader#*/}
        case $pl_recorded:$pl_oracle in
            yes:command:*)
                pl_name=${pl_oracle#command:}
                pl_p=0 pl_n=0
                for pl_dir in "$try"/a/sessions/*/; do
                    [ -f "${pl_dir}oracles" ] || continue
                    [ "$(head -1 "${pl_dir}status" 2>/dev/null)" = 0 ] || continue
                    [ "$(awk 'previous == "--probe" { print; exit } { previous = $0 }' "${pl_dir}record.argv" 2>/dev/null)" = "$pl_id" ] || continue
                    pl_n=$((pl_n + 1))
                    if awk -F '\t' -v n="$pl_name" '$1 == n && $2 == "pass" { found = 1 } END { exit !found }' "${pl_dir}oracles"; then
                        pl_p=$((pl_p + 1))
                    fi
                done
                pl_line="probe $pl_id ($pl_action): $pl_p of $pl_n by the oracle $pl_name (grader $pl_x of $pl_g)"
                [ "$pl_p/$pl_n" = "$pl_x/$pl_g" ] || pl_line="$pl_line disagree"
                printf '%s\n' "$pl_line"
                ;;
            *) printf 'probe %s (%s): %s of %s by the grader\n' "$pl_id" "$pl_action" "$pl_x" "$pl_g" ;;
        esac
    done
}

# ------------------------------------------------------------ the base

# manifest <tree>: `<path>\t<sha256>` for every file, and `<path>\t-> <target>`
# for every link, sorted.
manifest() {
    (
        cd "$1" || exit 3
        find . -type f -print0 | xargs -0 -r sha256sum | awk '{ h = $1; sub(/^[0-9a-f]+ [ *]/, ""); print $0 "\t" h }'
        find . -type l -printf '%p\t-> %l\n'
    ) | LC_ALL=C sort
}

# regions <copy>: the line numbers inside the marked regions of one copy of
# campaign.sh. A region that is unclosed, nested, or that reaches the check
# of the tools exits 2.
regions() {
    awk -v file="$2" '
        /^[ \t]*# base-neutral: begin/ {
            if (on) { bad = "a region begins inside another at line " NR; exit }
            on = 1; start = NR
        }
        /^for tool in git jq tar awk cargo claude bwrap/ {
            sentinel = 1
            if (on) { bad = "the region that begins at line " start " reaches the check of the tools at line " NR; exit }
        }
        on { print NR }
        /^[ \t]*# base-neutral: end/ {
            if (!on) { bad = "a region ends at line " NR " and none began"; exit }
            on = 0; regions++
            if (sentinel) { bad = "the region that ends at line " NR " is past the check of the tools"; exit }
        }
        END {
            if (bad == "" && on) bad = "the region that begins at line " start " never ends"
            if (bad != "") { print "campaign.sh copy " file ": " bad > "/dev/stderr"; exit 2 }
        }
    ' "$1"
}

# neutral <old> <new>: 0 when every line of `diff -U0` is neutral, and
# otherwise 1 and the first line that is not.
neutral() {
    neutral_dir=$(mktemp -d "${TMPDIR:-/tmp}/try-neutral.XXXXXX") || exit 3
    regions "$1" older > "$neutral_dir/old" || { rm -rf "$neutral_dir"; exit 2; }
    regions "$2" newer > "$neutral_dir/new" || { rm -rf "$neutral_dir"; exit 2; }
    diff -U0 "$1" "$2" > "$neutral_dir/diff"
    awk -v oldr="$neutral_dir/old" -v newr="$neutral_dir/new" '
        BEGIN {
            while ((getline n < oldr) > 0) ro[n] = 1
            while ((getline n < newr) > 0) rn[n] = 1
        }
        function quiet(text) { return text ~ /^[ \t]*(#.*)?$/ }
        NR <= 2 && (/^--- / || /^\+\+\+ /) { next }
        /^@@ / {
            split($2, a, ","); split($3, b, ",")
            o = substr(a[1], 2) + 0; n = substr(b[1], 2) + 0
            next
        }
        /^-/ {
            text = substr($0, 2)
            if (!quiet(text) && !(o in ro)) { print "removed line " o ": " text; bad = 1; exit }
            o++; next
        }
        /^\+/ {
            text = substr($0, 2)
            if (!quiet(text) && !(n in rn)) { print "added line " n ": " text; bad = 1; exit }
            n++; next
        }
        END { exit bad }
    ' "$neutral_dir/diff"
    neutral_status=$?
    rm -rf "$neutral_dir"
    return "$neutral_status"
}

# base_check <base> <sha>: build both trees, print what differs, and set
# `stale` to 1 when a path invalidates the base.
base_check() {
    invalidators=$(declared base_invalidators) || exit $?
    base_dir=$(mktemp -d "${TMPDIR:-/tmp}/try-plan.XXXXXX") || exit 3
    printf '%s\n' "$invalidators" > "$base_dir/invalidators"
    for base_side in pin head; do
        if [ "$base_side" = pin ]; then base_rev=$2; else base_rev=HEAD; fi
        sh "$campaign" --scenario-tree "$1" --out "$base_dir/$base_side" --at "$base_rev" \
            > "$base_dir/$base_side.out" 2> "$base_dir/$base_side.err"
        base_status=$?
        if [ "$base_status" != 0 ] || [ ! -d "$base_dir/$base_side/trees/additive-scenario-$1" ]; then
            say "the tree of scenario $1 at $base_rev did not build (campaign.sh exited $base_status):"
            tail -5 "$base_dir/$base_side.err" >&2
            rm -rf "$base_dir"
            exit 3
        fi
        manifest "$base_dir/$base_side/trees/additive-scenario-$1" > "$base_dir/$base_side.manifest"
    done
    awk -F'\t' '
        NR == FNR { a[$1] = $2; next }
        { seen[$1] = 1; if (!($1 in a) || a[$1] != $2) d[$1] = 1 }
        END { for (p in a) if (!(p in seen)) d[p] = 1; for (p in d) print p }
    ' "$base_dir/pin.manifest" "$base_dir/head.manifest" | sed 's|^\./||' | LC_ALL=C sort > "$base_dir/differs"
    stale=0
    echo "base: scenario $1 at $2 against HEAD $(git -C "$repo" rev-parse HEAD 2>/dev/null), $(wc -l < "$base_dir/differs" | tr -d ' ') paths differ"
    while IFS= read -r base_path; do
        base_pattern=
        while IFS= read -r base_candidate; do
            # shellcheck disable=SC2254
            case $base_path in
                $base_candidate) base_pattern=$base_candidate; break ;;
            esac
        done < "$base_dir/invalidators"
        if [ -z "$base_pattern" ]; then
            echo "differs: $base_path"
        elif [ "$base_path" = tools/probe/campaign.sh ] \
            && [ -f "$base_dir/pin/trees/additive-scenario-$1/$base_path" ] \
            && [ -f "$base_dir/head/trees/additive-scenario-$1/$base_path" ]; then
            base_line=$(neutral "$base_dir/pin/trees/additive-scenario-$1/$base_path" "$base_dir/head/trees/additive-scenario-$1/$base_path")
            case $? in
                0) echo "neutral: $base_path (every changed line is a comment, a blank or inside a base-neutral region)" ;;
                1) echo "invalidates: $base_path ($base_pattern): $base_line"; stale=1; echo "$base_path" >> "$base_dir/invalid" ;;
                *) rm -rf "$base_dir"; exit 2 ;;
            esac
        else
            echo "invalidates: $base_path ($base_pattern)"
            stale=1
            echo "$base_path" >> "$base_dir/invalid"
        fi
    done < "$base_dir/differs"
    # Each path that invalidates, with its digest in the tree at HEAD, for
    # the `accept` event that records it and for `plan`, which compares it.
    base_invalid=
    if [ -s "$base_dir/invalid" ]; then
        base_invalid=$(awk -F'\t' '
            NR == FNR { h[$1] = $2; next }
            { p = "./" $0; print $0 "\t" ((p in h) ? h[p] : "absent") }
        ' "$base_dir/head.manifest" "$base_dir/invalid")
    fi
    rm -rf "$base_dir"
}

# stage_root: the try root of the stage of `plan` or `launch`, by name.
stage_root() {
    case $stage_kind in
        screen) echo "$open_layer-r$open_try-v$stage_arg" ;;
        best) echo "$open_layer-r$open_try-best" ;;
        confirm) echo "$open_layer-r$open_try-confirm" ;;
        base)
            stage_k=1
            while [ -e "$root/base-$stage_k" ]; do stage_k=$((stage_k + 1)); done
            echo "base-$stage_k"
            ;;
    esac
}

# open_base: the base scenario of the open scenario, in `base`.
open_base() {
    scenarios=$(declared scenarios) || exit $?
    base=$(printf '%s\n' "$scenarios" | awk -v s="$open_scenario" '$1 == s { print $2 }')
    if [ -z "$base" ] || [ "$base" = - ]; then
        say "scenario $open_scenario names no base in the declaration."
        exit 2
    fi
}

# share_above <used> <sessions> <used> <sessions>: 0 when the first share is
# above the second, by whole numbers alone.
share_above() {
    [ "$2" -gt 0 ] && [ $(($1 * $4)) -gt $(($3 * $2)) ]
}

# round_check: the rules of rounds and order, and of the stage, which refuse
# with 6 before anything is priced or built. It sets `best_due`. A base is
# no stage of a round, so it reads none of those rules: it refuses a base at
# a commit the ledger has one at.
round_check() {
    if [ "$stage_kind" = base ]; then
        best_due=0
        open_base
        base_head=$(git -C "$checkout" rev-parse HEAD 2>/dev/null)
        base_measured=$(ledger_py base "$base" "${base_head:-none}" -) || exit $?
        if [ -n "$base_measured" ]; then
            say "scenario $base is measured at $base_head already, in $(printf '%s\n' "$base_measured" | cut -f4): a base runs once at a commit."
            exit 6
        fi
        echo "base: scenario $base, the base of scenario $open_scenario, at HEAD ${base_head:-unknown}"
        return
    fi
    case " $t_order " in
        *" $open_layer "*) ;;
        *)
            say "the open layer is $open_layer, and \`order\` of the tuning round, $t_order, does not name it, so it has no place in the order of the layers."
            exit 6
            ;;
    esac
    # shellcheck disable=SC2086
    layer_states=$(ledger_py layers "$t_rounds" "$t_stop" $t_order) || exit $?
    for order_layer in $t_order; do
        order_state=$(printf '%s\n' "$layer_states" | awk -v l="$order_layer" '$1 == l { print $2 }')
        order_closed=$(printf '%s\n' "$layer_states" | awk -v l="$order_layer" '$1 == l { print $3 }')
        if [ "$order_layer" = "$open_layer" ]; then
            echo "layer: $open_layer $order_state, rounds closed: $order_closed"
            case $order_state in
                stopped) say "the $open_layer layer stopped: its last $t_stop rounds kept nothing, so no round of it runs."; exit 6 ;;
                exhausted) say "the $open_layer layer took its $t_rounds rounds, the most a layer takes."; exit 6 ;;
            esac
            case ",$order_closed," in
                *",$open_try,"*) say "round $open_try of the $open_layer layer is closed in $root/ledger.jsonl, so it runs nothing more. Open round $((open_try + 1)) under rounds.tuning.open."; exit 6 ;;
            esac
            break
        fi
        if [ "$order_state" = open ]; then
            say "the $order_layer layer comes before $open_layer in \`order\`, $t_order, and it is open: it neither stopped nor took its $t_rounds rounds. No round of $open_layer runs before it."
            exit 6
        fi
    done
    best_line=$(ledger_py best "$open_layer") || exit $?
    best_due=1
    [ -n "$best_line" ] && best_due=0
    case $stage_kind in
        screen)
            if [ "$stage_arg" -gt "$t_variants" ]; then
                say "a round screens at most $t_variants variants, and this is variant $stage_arg."
                exit 6
            fi
            ;;
        best)
            if [ "$best_due" = 0 ]; then
                say "the best so far of $open_layer is measured at the confirmation's count already: $(printf '%s' "$best_line" | cut -f1). A best so far runs only where none is."
                exit 6
            fi
            ;;
        confirm) confirm_check ;;
    esac
}

# confirm_check: the variant of `confirm <variant>` is the top of the rank
# of the round, and it raises the use over the best so far.
confirm_check() {
    case $stage_arg in
        "$open_layer-r$open_try-v"[1-9]*) ;;
        *) say "$stage_arg is not a screened variant of round $open_try of $open_layer: a variant is $open_layer-r$open_try-v<k>."; exit 6 ;;
    esac
    for confirm_root in "$root/$open_layer-r$open_try-v"*/; do
        [ -d "$confirm_root" ] || continue
        if [ ! -f "${confirm_root}result.md" ]; then
            say "the screen ${confirm_root%/} has no result.md, so the round is not ranked yet."
            exit 6
        fi
    done
    ranked=$(ledger_py rank "$open_layer" "$open_try") || exit $?
    variant_row=$(printf '%s\n' "$ranked" | awk -F'\t' -v t="$stage_arg" '$1 == t')
    if [ -z "$variant_row" ]; then
        say "$stage_arg has no screen event in $root/ledger.jsonl, so it is not ranked."
        exit 6
    fi
    if [ -z "$best_line" ]; then
        say "the $open_layer layer has no best so far at the confirmation's count, so no gain in use can be read. Launch the best so far first: sh tools/probe/try.sh launch $root best."
        exit 6
    fi
    top=$(printf '%s\n' "$ranked" | head -1 | cut -f1)
    variant_used=$(printf '%s\n' "$variant_row" | cut -f2)
    variant_sessions=$(printf '%s\n' "$variant_row" | cut -f3)
    variant_passed=$(printf '%s\n' "$variant_row" | cut -f4)
    variant_graded=$(printf '%s\n' "$variant_row" | cut -f5)
    best_used=$(printf '%s\n' "$best_line" | cut -f2)
    best_sessions=$(printf '%s\n' "$best_line" | cut -f3)
    echo "best so far: $(printf '%s\n' "$best_line" | cut -f1), used in $best_used of $best_sessions sessions"
    echo "variant: $stage_arg, used in $variant_used of $variant_sessions sessions, passed $variant_passed of $variant_graded"
    if ! share_above "$variant_used" "$variant_sessions" "$best_used" "$best_sessions"; then
        if [ "$stage_arg" = "$top" ] && [ "$verb" = launch ]; then
            ledger_py append event=s:round "layer=s:$open_layer" "round=i:$open_try" kept=b:false \
                reason=s:no-gain-in-use "top=s:$top" "at=s:$(now)"
            echo "round: $open_try of $open_layer closed with nothing kept: its top variant does not raise the use"
        fi
        say "$stage_arg used the $open_layer layer in $variant_used of $variant_sessions sessions, and the best so far in $best_used of $best_sessions: no gain in use, so it is not confirmed, whatever its pass rate of $variant_passed of $variant_graded."
        exit 6
    fi
    if [ "$stage_arg" != "$top" ]; then
        say "$stage_arg is not the top of the rank of round $open_try: $top is. Only the top variant is confirmed."
        exit 6
    fi
}

# route_refuse <reason>: a confirmation waits for the base to be measured
# again at the confirmation's count. The re-measure is priced under the
# ceiling it is charged to, so a re-measure that the spend refuses exits 5,
# and one it does not exits 4.
route_refuse() {
    route_root=$(stage_kind=base && stage_root)
    route_price=$((t_confirm * t_rate))
    if [ "$(charge_of "$route_root")" = base ]; then
        echo "left of the bases with the re-measure: $t_confirm sessions, $route_price cents, in $route_root"
        if [ $((spent_base + route_price)) -gt "$t_base_ceiling" ]; then
            say "$1 The re-measure in $route_root is priced at $route_price cents, and the bases spent $spent_base, $((spent_base + route_price)) in all, above the base ceiling of $t_base_ceiling cents. Nothing is launched."
            exit 5
        fi
    else
        echo "left with the re-measure: $((left + t_confirm)) sessions of the round, $(((left + t_confirm) * t_rate)) cents, the re-measure in $route_root included"
        if [ $((spent_tries + (left + t_confirm) * t_rate)) -gt "$t_ceiling" ]; then
            say "$1 The re-measure in $route_root and what is left of round $open_try are priced at $(((left + t_confirm) * t_rate)) cents, and the tries spent $spent_tries, $((spent_tries + (left + t_confirm) * t_rate)) in all, above the tuning ceiling of $t_ceiling cents. Nothing is launched."
            exit 5
        fi
    fi
    say "$1 Measure scenario $base again at $t_confirm sessions at the variant's commit first: sh tools/probe/try.sh launch $root base, and then result."
    exit 4
}

# plan_check: the whole of `plan`, which `launch` runs first. It sets
# `open_layer`, `open_scenario`, `open_try` and `open_cap`, and exits with
# the status of a refusal.
plan_check() {
    plan_open=$(open_fields) || exit $?
    if [ "$plan_open" = none ]; then
        echo "open: none"
        say "the tuning round is open for no layer: \`open:\` under \`rounds.tuning\` in the declaration is \`none\`, so no try launches."
        exit 5
    fi
    # shellcheck disable=SC2086
    set -- $plan_open
    open_layer=$1 open_scenario=$2 open_try=$3 open_cap=$4 open_moment=${5:-}
    tuning_read
    echo "open: $open_layer on scenario $open_scenario, round $open_try of at most $t_rounds"
    round_check
    # The stage runs its selection at the stage's repetitions, and a base
    # the whole set (#1875).
    select_probes
    s_screen=$((t_screen_reps * selected_n))
    s_confirm=$((t_confirm_reps * selected_n))
    if [ "$open_layer" = hooks ]; then
        printf '%s\n' "$selected" | awk 'NF { print "selected: " $0 }'
    fi
    approvals=$(declared approved) || exit $?
    approved=$(printf '%s\n' "$approvals" | awk '$1 == "tuning" { print $2 }')
    spent=$(spent_round) || exit $?
    # shellcheck disable=SC2046
    set -- $(spent_split)
    spent_base=$1 spent_tries=$2
    # The first best so far of a layer is charged to the base ceiling.
    best_base=0
    if [ "$best_due" = 1 ] && [ "$(charge_of "$open_layer-r$open_try-best")" = base ]; then
        best_base=1
    fi
    # The sessions left under each ceiling from this stage on, and the
    # stage's own: `left` under the tuning ceiling, `left_base` under the
    # base ceiling.
    case $stage_kind in
        '') left=$((t_variants * s_screen + s_confirm + (best_due - best_base) * s_confirm)) left_base=$((best_base * s_confirm)) stage_sessions=0 ;;
        screen) left=$(((t_variants - stage_arg + 1) * s_screen + s_confirm + (best_due - best_base) * s_confirm)) left_base=$((best_base * s_confirm)) stage_sessions=$s_screen ;;
        best) left=$(((2 - best_base) * s_confirm)) left_base=$((best_base * s_confirm)) stage_sessions=$s_confirm ;;
        confirm) left=$s_confirm left_base=0 stage_sessions=$s_confirm ;;
        base)
            if [ "$(charge_of "$(stage_root)")" = base ]; then left=0 left_base=$t_confirm; else left=$t_confirm left_base=0; fi
            stage_sessions=$t_confirm
            ;;
    esac
    priced=$((left * t_rate))
    priced_base=$((left_base * t_rate))
    echo "spent: $spent, of which the bases $spent_base and the tries $spent_tries"
    echo "cap: $open_cap"
    echo "approved: $approved"
    echo "ceiling: $t_ceiling"
    echo "base ceiling: $t_base_ceiling"
    [ -n "$stage_kind" ] && echo "stage: $stage_kind${stage_arg:+ $stage_arg}, $stage_sessions sessions, $((stage_sessions * t_rate)) cents at $t_rate a session, in $(stage_root)"
    if [ "$stage_kind" = screen ]; then
        screen_draw=$(draw_lines) || exit 3
        printf '%s\n' "$screen_draw"
    fi
    echo "left: $left sessions of the round, $priced cents at $t_rate a session$([ $((best_due - best_base)) = 1 ] && echo ", the best so far at $s_confirm included")"
    echo "left of the bases: $left_base sessions, $priced_base cents at $t_rate a session"
    if [ $((spent + open_cap)) -gt "${approved:-0}" ]; then
        say "the round spent $spent cents, and a try of $open_cap more is $((spent + open_cap)), above the $approved cents approved for the tuning round. Nothing is launched."
        exit 5
    fi
    if [ $((spent_tries + priced)) -gt "$t_ceiling" ]; then
        say "the tries of the round spent $spent_tries cents, and what is left of round $open_try is priced at $priced more, $((spent_tries + priced)), above the tuning ceiling of $t_ceiling cents. Nothing is launched."
        exit 5
    fi
    if [ $((spent_base + priced_base)) -gt "$t_base_ceiling" ]; then
        say "the bases spent $spent_base cents, and the $([ "$stage_kind" = base ] && echo "base" || echo "best so far of $open_layer") is priced at $priced_base more, $((spent_base + priced_base)), above the base ceiling of $t_base_ceiling cents. Nothing is launched."
        exit 5
    fi
    if [ $((stage_sessions * t_rate)) -gt "$open_cap" ]; then
        say "the $stage_kind is priced at $((stage_sessions * t_rate)) cents, above the cap of one launch, $open_cap, at which the driver would stop it unfinished. Nothing is launched."
        exit 5
    fi
    if [ "$stage_kind" = base ]; then
        # The base is the measurement the base check asks for, so it builds
        # no tree and reads no pin.
        echo "plan: the base, scenario $base, at the confirmation's count, may launch"
        return
    fi
    open_base
    read_pin
    if [ "$pin_scenario" != "$base" ]; then
        say "$pin_file reads \`$(head -1 "$pin_file")\`, and it must be \`$base <sha>\`, the base of scenario $open_scenario and the commit it was measured at."
        exit 2
    fi
    base_check "$base" "$pin_sha"
    route_hit=0
    cover=
    if [ "$stale" = 1 ]; then
        cover=$(printf '%s\n' "$base_invalid" | ledger_py cover "$base" "$pin_sha") || exit $?
        if [ "$stage_kind" = confirm ]; then
            # A change to the engine's source can change what `headwater
            # route` prints, so no acceptance covers it for a confirmation.
            cover=$(printf '%s\n' "$cover" | awk -F'\t' '
                $1 == "covered" && $2 ~ /^engine\/crates\/[^\/]*\/src\// { print "route\t" $2 "\t" $3; next }
                { print }')
            printf '%s\n' "$cover" | awk -F'\t' '$1 == "route" { print "route: " $2 " (accepted by " $3 ", and no acceptance covers the engine for a confirmation)" }'
            printf '%s\n' "$cover" | grep -q '^route' && route_hit=1
        fi
        printf '%s\n' "$cover" | awk -F'\t' '$1 == "covered" { print "accepted: " $2 " (" $3 ")" }'
    fi
    if [ "$stage_kind" = confirm ]; then
        # A confirmation reads the base at the confirmation's count, measured
        # at the pin. A pin that a person wrote is no measurement (#1838).
        base_measured=$(ledger_py base "$base" "$pin_sha" "$t_confirm") || exit $?
        if [ -z "$base_measured" ]; then
            route_refuse "no base event at $t_confirm sessions in $root/ledger.jsonl measures scenario $base at $pin_sha, the pin of $pin_file, and a pin written by hand is not a measurement."
        fi
        echo "measured: scenario $base at $pin_sha, $t_confirm sessions, in $(printf '%s\n' "$base_measured" | cut -f4): $(printf '%s\n' "$base_measured" | cut -f1)"
        if [ "$route_hit" = 1 ]; then
            route_refuse "the engine's source differs from the base's pin, and a route change makes scenario $base stale for a confirmation whatever was accepted."
        fi
    fi
    if [ "$stale" = 1 ]; then
        if printf '%s\n' "$cover" | grep -q '^uncovered'; then
            say "the base, scenario $base at $pin_sha, is stale: a path above that differs is on base_invalidators, and no accept event covers it at its digest now. Re-measure the base, or have the owner accept the difference and record it with try.sh accept."
            exit 4
        fi
    fi
    if [ "$stage_kind" = screen ]; then
        variant_tree=$(scenario_digest "$open_scenario") || exit $?
        echo "tree: scenario $open_scenario at HEAD, $variant_tree"
        for other in "$root/$open_layer-r$open_try-v"*/; do
            [ -f "${other}tree" ] || continue
            [ "${other%/}" = "$root/$(stage_root)" ] && continue
            if [ "$(head -1 "${other}tree")" = "$variant_tree" ]; then
                say "the tree of scenario $open_scenario at HEAD is the tree ${other%/} screened, so two variants of round $open_try would share one tree."
                exit 6
            fi
        done
    fi
    case $stage_kind in
        '') echo "plan: round $open_try of $open_layer on scenario $open_scenario may start" ;;
        screen) echo "plan: the screen of variant $stage_arg of round $open_try of $open_layer on scenario $open_scenario may launch" ;;
        best) echo "plan: the best so far of $open_layer on scenario $open_scenario, at the confirmation's count, may launch" ;;
        confirm) echo "plan: the confirmation of $stage_arg in round $open_try of $open_layer on scenario $open_scenario may launch" ;;
    esac
}

# scenario_digest <scenario>: the sha256 of the manifest of the scenario's
# tree at HEAD, as `campaign.sh --scenario-tree` builds it.
scenario_digest() {
    digest_dir=$(mktemp -d "${TMPDIR:-/tmp}/try-variant.XXXXXX") || exit 3
    sh "$campaign" --scenario-tree "$1" --out "$digest_dir/head" --at HEAD > "$digest_dir/out" 2> "$digest_dir/err"
    digest_status=$?
    if [ "$digest_status" != 0 ] || [ ! -d "$digest_dir/head/trees/additive-scenario-$1" ]; then
        say "the tree of scenario $1 at HEAD did not build (campaign.sh exited $digest_status):"
        tail -5 "$digest_dir/err" >&2
        rm -rf "$digest_dir"
        exit 3
    fi
    manifest "$digest_dir/head/trees/additive-scenario-$1" | sha256sum | awk '{ print $1 }'
    rm -rf "$digest_dir"
}

# ------------------------------------------------------------ the driver

# alive <try>: 0 when the pid in `driver.pid` runs a driver of that root.
alive() {
    [ -f "$1/driver.pid" ] || return 1
    alive_pid=$(head -1 "$1/driver.pid")
    case $alive_pid in '' | *[!0-9]*) return 1 ;; esac
    kill -0 "$alive_pid" 2>/dev/null || return 1
    tr '\0' ' ' < "/proc/$alive_pid/cmdline" 2>/dev/null | grep -qF -- "--root $1 "
}

# start <try>: the driver detached, with the arguments of `<try>/driver.argv`.
start() {
    start_try=$1
    set --
    while IFS= read -r start_arg; do
        set -- "$@" "$start_arg"
    done < "$start_try/driver.argv"
    command -v setsid >/dev/null 2>&1 || { say "setsid is not on the path."; exit 3; }
    # The stage names the count of the batch, which the dry run reads (#1838).
    case $(awk '{ print $1; exit }' "$start_try/stage" 2>/dev/null) in
        screen) HEADWATER_TUNING_STAGE=screen ;;
        best | confirm | base) HEADWATER_TUNING_STAGE=confirm ;;
        *) say "$start_try/stage names no stage, so the driver would not know the count of its batch."; exit 2 ;;
    esac
    export HEADWATER_TUNING_STAGE
    # A screen's fixed draw, from the file launch wrote (#1875). Every other
    # stage spreads the phrasings, so it names no draw.
    if [ -f "$start_try/draw" ]; then
        HEADWATER_CAMPAIGN_DRAW=$start_try/draw
        export HEADWATER_CAMPAIGN_DRAW
    else
        unset HEADWATER_CAMPAIGN_DRAW
    fi
    # The usage gate reads the current `additive-launch.env`, on a relaunch
    # as on a launch, and not `driver.argv` (#1876).
    HEADWATER_DRIVE_USAGE_STOP=$TRY_USAGE_STOP_PERCENT
    HEADWATER_DRIVE_USAGE_LATE=$TRY_USAGE_LATE_PERCENT
    HEADWATER_DRIVE_USAGE_LATE_HOURS=$TRY_USAGE_LATE_HOURS
    export HEADWATER_DRIVE_USAGE_STOP HEADWATER_DRIVE_USAGE_LATE HEADWATER_DRIVE_USAGE_LATE_HOURS
    cd "$checkout" || exit 3
    setsid nohup sh "$driver" "$@" >> "$start_try/drive.out" 2>&1 < /dev/null &
    echo $! > "$start_try/driver.pid"
    date -u +%Y-%m-%dT%H:%M:%SZ >> "$start_try/launched"
    echo "launched: $start_try, driver pid $(cat "$start_try/driver.pid")"
}

# pin_ready <pin>: the checkout is in the state the driver's own pin check
# accepts, so that a launch never reports a driver that exits at once.
pin_ready() {
    command -v git >/dev/null 2>&1 || { say "git is not on the path."; exit 3; }
    pin_head=$(git -C "$checkout" rev-parse HEAD 2>/dev/null)
    if [ "$pin_head" != "$1" ]; then
        say "the checkout $checkout is at ${pin_head:-no commit}, not the pin $1."
        exit 6
    fi
    if git -C "$checkout" symbolic-ref -q HEAD >/dev/null 2>&1; then
        say "the checkout $checkout is on a branch, and the driver refuses that. Detach it at the pin: git switch --detach $1."
        exit 6
    fi
    pin_on=
    if git -C "$checkout" merge-base --is-ancestor "$1" origin/main 2>/dev/null; then
        pin_on=main
    else
        for ref in $(git -C "$checkout" for-each-ref --format='%(refname)' refs/remotes/origin/tune/ 2>/dev/null); do
            git -C "$checkout" merge-base --is-ancestor "$1" "$ref" 2>/dev/null && { pin_on=$ref; break; }
        done
    fi
    if [ -z "$pin_on" ]; then
        say "the pin $1 is not on origin/main or on a branch under origin/tune/, and the driver refuses that."
        exit 6
    fi
    if [ -n "$(git -C "$checkout" status --porcelain 2>/dev/null)" ]; then
        say "the checkout $checkout is dirty, and the driver refuses that."
        exit 6
    fi
}

markers() {
    marker_list=
    for marker in canary-recorded canary-passed canary-needs-reader canary-failed a.done capped stop usage-gated done; do
        [ -e "$1/$marker" ] && marker_list="$marker_list $marker"
    done
    printf '%s\n' "${marker_list# }"
}

# ------------------------------------------------------------ the verbs

case $verb in
    spent)
        spent_round
        # shellcheck disable=SC2046
        set -- $(spent_split) $(ceilings)
        echo "bases: $1 of the base ceiling, $4"
        echo "tries: $2 of the tuning ceiling, $3"
        ;;

    status)
        for status_try in "$root"/*/ "$root"/.[!.]*/ "$root"/..?*/; do
            [ -d "$status_try" ] || continue
            status_try=${status_try%/}
            status_driver=none
            if [ -f "$status_try/driver.pid" ]; then
                if alive "$status_try"; then status_driver=running; else status_driver=exited; fi
            fi
            echo "${status_try##*/} spent=$(try_spent "$status_try") driver=$status_driver markers=$(markers "$status_try")"
        done
        total=$(spent_round) || exit $?
        approvals=$(declared approved) || exit $?
        open_line=$(declared open) || exit $?
        echo "spent: $total of $(printf '%s\n' "$approvals" | awk '$1 == "tuning" { print $2 }') approved"
        # shellcheck disable=SC2046
        set -- $(spent_split) $(ceilings)
        echo "bases: $1 of the base ceiling, $4"
        echo "tries: $2 of the tuning ceiling, $3"
        echo "open: $open_line"
        ;;

    plan)
        read_env 0
        plan_check
        ;;

    rank)
        rank_open=$(open_fields) || exit $?
        if [ "$rank_open" = none ]; then
            say "no layer is open under rounds.tuning.open, so no round is ranked."
            exit 5
        fi
        # shellcheck disable=SC2086
        set -- $rank_open
        open_layer=$1 open_try=$3
        rank_best=$(ledger_py best "$open_layer") || exit $?
        if [ -n "$rank_best" ]; then
            rank_used=$(printf '%s\n' "$rank_best" | cut -f2)
            rank_sessions=$(printf '%s\n' "$rank_best" | cut -f3)
            echo "best so far: $(printf '%s\n' "$rank_best" | cut -f1), used in $rank_used of $rank_sessions sessions"
        else
            echo "best so far: none at the confirmation's count"
        fi
        rank_rows=$(ledger_py rank "$open_layer" "$open_try") || exit $?
        [ -n "$rank_rows" ] || echo "no screen of round $open_try of $open_layer is recorded"
        rank_n=0
        printf '%s\n' "$rank_rows" | while IFS='	' read -r r_try r_used r_sessions r_passed r_graded; do
            [ -n "$r_try" ] || continue
            rank_n=$((rank_n + 1))
            r_gain=-
            if [ -n "$rank_best" ]; then
                if share_above "$r_used" "$r_sessions" "$rank_used" "$rank_sessions"; then r_gain=gain; else r_gain="no gain"; fi
            fi
            echo "rank $rank_n: $r_try used in $r_used of $r_sessions sessions, passed $r_passed of $r_graded, $r_gain"
        done
        ;;

    launch)
        read_env 1
        plan_check
        try=$root/$(stage_root)
        if [ -e "$try" ]; then
            say "$try exists already, so this is not a first launch. Use relaunch to start its driver again."
            exit 6
        fi
        case $root in
            *"'"* | *[[:space:]]*) say "the round root $root holds a quote or a space, which the canary command cannot carry."; exit 2 ;;
        esac
        command -v git >/dev/null 2>&1 || { say "git is not on the path."; exit 3; }
        pin=$(git -C "$checkout" rev-parse HEAD) || exit 3
        if [ "$stage_kind" = confirm ]; then
            variant_pin=$(awk 'previous == "--pin" { print; exit } { previous = $0 }' "$root/$stage_arg/driver.argv" 2>/dev/null)
            if [ "$pin" != "$variant_pin" ]; then
                say "the checkout $checkout is at $pin, and $stage_arg was screened at ${variant_pin:-no pin it records}. Detach the checkout at that commit to confirm it."
                exit 6
            fi
        fi
        pin_ready "$pin"
        mkdir -p "$try" || exit 2
        if [ "$stage_kind" = base ]; then spec_scenario=$base; else spec_scenario=$open_scenario; fi
        # A probe outside a hook variant's selection is an exclude (#1875).
        printf 'additive scenario-%s tuning%s\n' "$spec_scenario" "${excluded:+ $excluded}" > "$try/spec"
        printf '%s\n' "$stage_kind${stage_arg:+ $stage_arg}" > "$try/stage"
        [ "$stage_kind" = screen ] && printf '%s\n' "$variant_tree" > "$try/tree"
        [ "$stage_kind" = screen ] && printf '%s\n' "$screen_draw" > "$try/draw"
        printf '%s\n' --root "$try" --pin "$pin" --claude-version "$TRY_HARNESS" \
            --spec-a "$try/spec" --model "$TRY_MODEL" --session-cents "$TRY_SESSION_CENTS" \
            --cap-cents "$open_cap" --parallel "$TRY_PARALLEL" --overshoot-cents "$TRY_OVERSHOOT_CENTS" \
            --seed "$TRY_SEED" --on-canary "sh '$self' canary '$root' '${try##*/}'" > "$try/driver.argv"
        start "$try"
        ;;

    relaunch)
        resolve_try
        [ -d "$try" ] || { say "no try root at $try."; exit 6; }
        for marker in done capped canary-failed stop; do
            if [ -e "$try/$marker" ]; then
                say "$try/$marker is present, so the driver is not started again. A person rules on it first."
                exit 6
            fi
        done
        if alive "$try"; then
            say "the driver of $try runs, as pid $(cat "$try/driver.pid")."
            exit 6
        fi
        [ -s "$try/driver.argv" ] || { say "no $try/driver.argv, so this try was not launched by try.sh."; exit 2; }
        read_env 1
        pin_ready "$(awk 'previous == "--pin" { print; exit } { previous = $0 }' "$try/driver.argv")"
        start "$try"
        ;;

    canary)
        resolve_try
        [ -f "$try/canary-passed" ] && { echo "canary-passed: present in $try"; exit 0; }
        [ -f "$try/canary-recorded" ] || { say "no $try/canary-recorded: the canary has not recorded."; exit 6; }
        # shellcheck disable=SC2046
        set -- $(head -1 "$try/spec" 2>/dev/null)
        case ${2:-} in
            scenario-[0-9]*) canary_scenario=${2#scenario-} ;;
            *) say "$try/spec does not name a scenario."; exit 2 ;;
        esac
        scenarios=$(declared scenarios) || exit $?
        canary_layers=$(printf '%s\n' "$scenarios" | awk -v s="$canary_scenario" '$1 == s { $1 = ""; $2 = ""; print }')
        case " $canary_layers " in
            *" hooks "*) hook_expected=live ;;
            *) hook_expected="not live" ;;
        esac
        bound=$((TRY_SESSION_CENTS + TRY_OVERSHOOT_CENTS))
        offences=
        records=0
        for job in $(awk 'NF { print $1 }' "$try/a/jobs" 2>/dev/null); do
            session=$try/a/sessions/$job
            [ -f "$session/status" ] || continue
            records=$((records + 1))
            record=$session/record.md
            status=$(head -1 "$session/status")
            cost=$(head -1 "$session/cost" 2>/dev/null)
            [ "$status" = 0 ] || offences="$offences
$job: status $status"
            case $cost in
                '' | *[!0-9]*) offences="$offences
$job: cost \`$cost\` is not a whole number of cents" ;;
                *) [ "$cost" -le "$bound" ] || offences="$offences
$job: cost $cost above $bound" ;;
            esac
            if [ ! -f "$record" ]; then
                offences="$offences
$job: no record.md"
                continue
            fi
            grep -qF 'The session made 0 calls to a web tool.' "$record" || offences="$offences
$job: web: $(grep -oE 'The session made [0-9]+ calls? to a web tool|The calls this session made to a web tool were not counted' "$record" | head -1)"
            grep -qF 'The session named 0 paths outside its workspace' "$record" || offences="$offences
$job: outside: $(grep -oE 'The session named [0-9]+ paths? outside its workspace|The paths outside its workspace this session named were not counted' "$record" | head -1)"
            grep -qF 'It refused no request of that kind' "$record" || offences="$offences
$job: provider: $(grep -oE 'It refused [^,]* of that kind' "$record" | head -1)"
            grep -qF "The intent hook was $hook_expected in this session." "$record" || offences="$offences
$job: hook: the scenario declares it $hook_expected, and the record says: $(grep -oE 'The intent hook was (not )?live in this session' "$record" | head -1)"
        done
        [ "$records" = 8 ] || offences="$offences
$records records of the canary, not 8"
        if [ -n "$offences" ]; then
            printf '%s\n' "${offences#?}" > "$try/canary-needs-reader"
            echo "canary-needs-reader: $try"
            cat "$try/canary-needs-reader"
            exit 7
        fi
        rm -f "$try/canary-needs-reader"
        echo "8 records, each status 0, cost at most $bound, no web call, no outside path, no refused provider fetch, the hook $hook_expected" > "$try/canary-passed"
        echo "canary-passed: $try"
        ;;

    wait)
        resolve_try
        read_env 0
        wait_root=$(df -Pk / | awk 'NR == 2 { print $4 }')
        wait_try=$(df -Pk "$root" | awk 'NR == 2 { print $4 }')
        if [ "${wait_root:-0}" -lt "$TRY_FLOOR_ROOT_KB" ] || [ "${wait_try:-0}" -lt "$TRY_FLOOR_TRY_KB" ]; then
            [ -d "$try" ] && [ ! -e "$try/stop" ] && echo "disk floor: / ${wait_root}K free, $root ${wait_try}K free" > "$try/stop"
            echo "disk-floor: / ${wait_root}K, $root ${wait_try}K"
            exit 0
        fi
        for marker in done canary-failed capped stop; do
            [ -e "$try/$marker" ] && { echo "$marker"; exit 0; }
        done
        if [ ! -e "$try/canary-passed" ]; then
            [ -e "$try/canary-needs-reader" ] && { echo canary-needs-reader; exit 0; }
            [ -e "$try/canary-recorded" ] && { echo canary-recorded; exit 0; }
        fi
        alive "$try" && exit 1
        # The driver stopped on the usage gate (#1876). The ledger records
        # each stop once, however often `wait` reads it: the content of
        # `usage-gated` starts with the time of the stop, and
        # `usage-gated.ledgered` holds the content already recorded.
        if [ -f "$try/usage-gated" ]; then
            if ! cmp -s "$try/usage-gated" "$try/usage-gated.ledgered" 2>/dev/null; then
                gate_field() { sed -n "s/^$1: //p" "$try/usage-gated" | head -n 1; }
                gate_resets=$(gate_field resets-at)
                gate_threshold=$(gate_field threshold)
                case $gate_resets in '' | *[!0-9]*) gate_resets=0 ;; esac
                case $gate_threshold in '' | *[!0-9]*) gate_threshold=0 ;; esac
                ledger_py append event=s:gate "try=s:${try##*/}" "percent=s:$(gate_field percent)" \
                    "resets_at=i:$gate_resets" "threshold=i:$gate_threshold" \
                    "source=s:$(gate_field source)" "at=s:$(now)"
                cp "$try/usage-gated" "$try/usage-gated.ledgered"
            fi
            echo "usage-gated: $(head -n 1 "$try/usage-gated")"
            exit 0
        fi
        echo driver-exited
        ;;

    result)
        resolve_try
        [ -d "$try" ] || { say "no try root at $try."; exit 2; }
        result_name=${try##*/}
        result_fields=$(printf '%s\n' "$result_name" | sed -n 's/^\([a-z][a-z0-9-]*\)-r\([1-9][0-9]*\)-\(v[1-9][0-9]*\|best\|confirm\)$/\1 \2 \3/p')
        if [ -z "$result_fields" ]; then
            result_fields=$(printf '%s\n' "$result_name" | sed -n 's/^base-\([1-9][0-9]*\)$/- \1 base/p')
        fi
        if [ -z "$result_fields" ]; then
            say "$try is not a try root of a round or a base: <layer>-r<n>-v<k>, <layer>-r<n>-best, <layer>-r<n>-confirm or base-<k>."
            exit 2
        fi
        # shellcheck disable=SC2086
        set -- $result_fields
        result_layer=$1 result_n=$2 result_part=$3
        result_stage=$(awk '{ print $1; exit }' "$try/stage" 2>/dev/null)
        result_variant=$(awk '{ print $2; exit }' "$try/stage" 2>/dev/null)
        case $result_stage:$result_part in
            screen:v"$result_variant" | best:best | confirm:confirm | base:base) ;;
            *) say "$try/stage reads \`$(head -1 "$try/stage" 2>/dev/null)\`, and the try root $result_name is not that stage."; exit 2 ;;
        esac
        [ ! -e "$try/result.md" ] || { say "$try/result.md is present: this try is recorded already."; exit 6; }
        result_spent=$(try_spent "$try")
        # Step 2: no `done`, so a person reads the try and nothing is decided.
        if [ ! -e "$try/done" ]; then
            ledger_py append event=s:halt "try=s:$result_name" "spent=i:$result_spent" "markers=s:$(markers "$try")" "at=s:$(now)"
            say "$try has no done, so it has no result: a halt event with its spend of $result_spent cents is in $root/ledger.jsonl."
            exit 7
        fi
        # Step 3: the session files against the driver's own last total.
        result_all=$(grep -o 'all=[0-9]*/' "$try/drive.out" 2>/dev/null | tail -1 | sed 's/^all=//; s|/$||')
        if [ -z "$result_all" ]; then
            say "$try/drive.out has no all=<n>/<cap> line, so the try's spend cannot be checked against the driver's."
            exit 2
        fi
        if [ "$result_all" != "$result_spent" ]; then
            say "the session files of $try sum to $result_spent cents, and the driver's last all= line in drive.out says $result_all."
            exit 2
        fi
        # The batch ran the stage's count, so a screen is never recorded as a
        # confirmation, nor a confirmation of another count (#1838).
        # A try that left probes out ran the others at the stage's
        # repetitions, so its count is those repetitions times the probes it
        # ran (#1875).
        tuning_read
        if [ "$result_stage" = screen ]; then result_reps=$t_screen_reps; else result_reps=$t_confirm_reps; fi
        result_set_n=$(declared tuning_set | awk 'NF' | wc -l | tr -d ' ')
        # shellcheck disable=SC2046
        set -- $(head -1 "$try/spec" 2>/dev/null)
        [ $# -ge 3 ] && shift 3
        result_count=$((result_reps * (result_set_n - $#)))
        result_jobs=$(awk 'NF' "$try/a/jobs" 2>/dev/null | wc -l | tr -d ' ')
        if [ "$result_jobs" != "$result_count" ]; then
            say "a $result_stage runs $result_count sessions, and the batch of $try planned $result_jobs jobs, so it is not recorded as a $result_stage."
            exit 2
        fi
        # A base adds no layer, so it counts no use.
        if [ "$result_stage" != base ]; then
            result_usage=$(python3 "$usage_counter" "$declaration" "$result_layer" "$try/a") || { say "the use of the $result_layer layer in $try/a was not counted."; exit 2; }
            # shellcheck disable=SC2046
            set -- $result_usage
            result_used=$2 result_sessions=$4
        fi
        # shellcheck disable=SC2046
        set -- $(head -1 "$try/spec" 2>/dev/null)
        case ${2:-} in
            scenario-[0-9]*) result_scenario=${2#scenario-} ;;
            *) say "$try/spec does not name a scenario."; exit 2 ;;
        esac
        scenarios=$(declared scenarios) || exit $?
        result_base=$(printf '%s\n' "$scenarios" | awk -v s="$result_scenario" '$1 == s { print $2 }')
        # A base is its own base: its base row compares it with the base
        # it measures again.
        [ "$result_stage" = base ] && result_base=$result_scenario
        if [ -z "$result_base" ] || [ "$result_base" = - ]; then
            say "scenario $result_scenario names no base in the declaration."
            exit 2
        fi
        # The controls, before anything is written.
        # The base row's control is the latest base that `base-pin` names and
        # a `base` event at the confirmation's count measured, and otherwise
        # the screening transcript of the base scenario (#1838).
        base_row_control=
        if [ -f "$root/base-pin" ]; then
            read_pin
            if [ "$pin_scenario" = "$result_base" ]; then
                base_backed=$(ledger_py base "$result_base" "$pin_sha" "$t_confirm") || exit $?
                if [ -n "$base_backed" ]; then
                    base_row_control=$(printf '%s\n' "$base_backed" | cut -f1)
                    [ -f "$checkout/$base_row_control" ] || { say "the base of scenario $result_base measured at $pin_sha, $base_row_control, is not in $checkout."; exit 2; }
                fi
            fi
        fi
        if [ -z "$base_row_control" ]; then
            base_row_control=$(one_transcript "base's screening transcript" additive screening "scenario-$result_base") || exit $?
        fi
        if [ "$result_stage" = base ]; then
            result_pin=$(awk 'previous == "--pin" { print; exit } { previous = $0 }' "$try/driver.argv" 2>/dev/null)
            case $result_pin in
                '' | *[!0-9a-f]*) say "$try/driver.argv names no --pin, so the base has no commit to be measured at."; exit 2 ;;
            esac
        fi
        best=
        if [ "$result_stage" = confirm ]; then
            best_line=$(ledger_py best "$result_layer") || exit $?
            if [ -z "$best_line" ]; then
                say "the $result_layer layer has no best so far at the confirmation's count in $root/ledger.jsonl, so the confirmation has nothing to be kept against."
                exit 2
            fi
            best=$(printf '%s\n' "$best_line" | cut -f1)
            best_count=$(printf '%s\n' "$best_line" | cut -f5)
            if [ "$best_count" != "$result_count" ]; then
                say "the best so far of $result_layer, $best, planned ${best_count:-an unrecorded count of} sessions, and the keep rule reads a confirmation against a best so far at $result_count alone."
                exit 2
            fi
            [ -f "$checkout/$best" ] || { say "the best so far of $result_layer, $best, is not in $checkout."; exit 2; }
        fi
        [ -x "$engine" ] || { say "no engine at $engine. Build it, or name it with HEADWATER_TRY_ENGINE."; exit 3; }
        # Step 4: the assembly, and the transcript that wraps it.
        if [ -s "$try/transcript" ] && [ -f "$checkout/$(head -1 "$try/transcript")" ]; then
            transcript=$(head -1 "$try/transcript")
        else
            (cd "$checkout" && sh "$campaign" --out "$try/a" --assemble) > "$try/assemble.out" 2> "$try/assemble.err"
            assemble_status=$?
            if [ "$assemble_status" != 0 ]; then
                say "campaign.sh --assemble on $try/a exited $assemble_status:"
                tail -5 "$try/assemble.err" >&2
                exit 2
            fi
            set -- "$try"/a/assembled/*.identity.yaml
            if [ $# != 1 ] || [ ! -f "$1" ]; then
                say "the assembly of $try/a wrote $# identity blocks, and a try has one line."
                exit 2
            fi
            stem=${1%.identity.yaml}
            [ -f "$stem.events.yaml" ] || { say "the assembly wrote no $stem.events.yaml."; exit 2; }
            case $result_stage in
                screen) result_what="variant $result_variant" ;;
                best) result_what="best so far" ;;
                confirm) result_what="confirmation" ;;
            esac
            if [ "$result_stage" = base ]; then
                result_title="Additive tuning of $(date -u +%Y-%m-%d), base re-measure of scenario $result_scenario"
                result_summary="The base re-measure of scenario $result_scenario in the additive tuning round, at the confirmation's count and the commit $result_pin: $(head -1 "$stem.summary" 2>/dev/null)."
            else
                result_title="Additive tuning of $(date -u +%Y-%m-%d), $result_layer round $result_n $result_what, scenario $result_scenario"
                result_summary="The $result_what of round $result_n of the $result_layer layer in the additive tuning round, the scenario-$result_scenario arm: $(head -1 "$stem.summary" 2>/dev/null)."
            fi
            "$engine" new probe_transcript --root "$checkout" \
                --title "$result_title" \
                --summary "$result_summary" \
                --facet tier=additive --facet "arm=scenario-$result_scenario" --facet round=tuning \
                > "$try/new.out" 2> "$try/new.err"
            new_status=$?
            transcript=$(sed -n 's/^wrote //p' "$try/new.out" | head -1)
            if [ "$new_status" != 0 ] || [ -z "$transcript" ] || [ ! -f "$checkout/$transcript" ]; then
                say "headwater new probe_transcript exited $new_status and wrote no transcript:"
                tail -5 "$try/new.err" >&2
                exit 3
            fi
            awk -v idf="$stem.identity.yaml" -v evf="$stem.events.yaml" '
                /^## / { section = $0 }
                $0 == "TODO write this section." && section == "## Run identity" { block(idf); next }
                $0 == "TODO write this section." && section == "## Events" { block(evf); next }
                { print }
                function block(file,    l) {
                    print "```yaml"
                    while ((getline l < file) > 0) print l
                    close(file)
                    print "```"
                }
            ' "$checkout/$transcript" > "$try/transcript.md" && cat "$try/transcript.md" > "$checkout/$transcript"
            rm -f "$try/transcript.md"
            if [ "$(grep -c '^```yaml$' "$checkout/$transcript")" != 2 ]; then
                say "the scaffold of $transcript did not take the two blocks: it has no Run identity or no Events section."
                exit 3
            fi
            printf '%s\n' "$transcript" > "$try/transcript"
        fi
        # Step 5: the base row, and for a screen or a best so far, the record.
        base_row=$(compare_row "$transcript" "$base_row_control") || exit $?
        # Each probe's pass: the oracle's verdict where the probe has a
        # command oracle, and the grader's count otherwise (#1875).
        result_probes=$(probe_lines "$transcript" "$result_scenario") || exit $?
        if [ "$result_stage" = base ]; then
            result_passed=$(printf '%s\n' "$base_row" | sed -n 's|^pooled: \([0-9][0-9]*\)/\([0-9][0-9]*\) against .*|\1 \2|p')
            if [ -z "$result_passed" ]; then
                say "the base row has no pass rate of the base, so it is not recorded: $base_row"
                exit 2
            fi
            # shellcheck disable=SC2086
            set -- $result_passed
            {
                echo "try: $result_name"
                echo "stage: base"
                echo "scenario: $result_scenario"
                echo "pin: $result_pin"
                echo "spent: $result_spent"
                echo "transcript: $transcript"
                echo "passed: $1 of $2"
                [ -z "$result_probes" ] || printf '%s\n' "$result_probes"
                echo "base row: $base_row, against $base_row_control"
                echo "keep: never: a base is never kept; it is the measurement a confirmation reads (#1838)"
            } > "$try/result.md"
            ledger_py append event=s:base "try=s:$result_name" "scenario=s:$result_scenario" "pin=s:$result_pin" \
                "count=i:$result_jobs" "passed=i:$1" "graded=i:$2" "spent=i:$result_spent" \
                "transcript=s:$transcript" "at=s:$(now)"
            printf '%s %s\n' "$result_scenario" "$result_pin" > "$root/base-pin"
            cat "$try/result.md"
            exit 0
        fi
        if [ "$result_stage" != confirm ]; then
            result_passed=$(printf '%s\n' "$base_row" | sed -n 's|^pooled: \([0-9][0-9]*\)/\([0-9][0-9]*\) against .*|\1 \2|p')
            if [ -z "$result_passed" ]; then
                say "the base row has no pass rate of the $result_stage, so it is not ranked: $base_row"
                exit 2
            fi
            # shellcheck disable=SC2086
            set -- $result_passed
            {
                echo "try: $result_name"
                echo "stage: $result_stage${result_variant:+ $result_variant}"
                echo "spent: $result_spent"
                echo "transcript: $transcript"
                echo "used: $result_used of $result_sessions sessions used the $result_layer layer"
                echo "passed: $1 of $2"
                [ -z "$result_probes" ] || printf '%s\n' "$result_probes"
                echo "base row: $base_row"
                echo "keep: never: a $result_stage is never kept, and only a confirmation is (owner, 2026-10-22, #1777)"
            } > "$try/result.md"
            ledger_py append "event=s:$result_stage" "try=s:$result_name" "layer=s:$result_layer" "round=i:$result_n" \
                "scenario=s:$result_scenario" "spent=i:$result_spent" "transcript=s:$transcript" "base_row=s:$base_row" \
                "used=i:$result_used" "sessions=i:$result_sessions" "count=i:$result_jobs" "passed=i:$1" "graded=i:$2" "at=s:$(now)"
            cat "$try/result.md"
            exit 0
        fi
        keep_row=$(compare_row "$transcript" "$best") || exit $?
        # Step 6: the keep rule, on the printed bounds.
        keep_fields=$(printf '%s\n' "$keep_row" | sed -n 's/.*, \([-+][0-9][0-9.]*\) points \[\([-+][0-9][0-9.]*\), \([-+][0-9][0-9.]*\)\]$/\1 \2/p')
        if [ -z "$keep_fields" ]; then
            say "the keep row has no interval, so nothing is kept: $keep_row"
            exit 2
        fi
        # shellcheck disable=SC2086
        set -- $keep_fields
        verdict=$(awk -v d="$1" -v low="$2" 'BEGIN {
            printf "%s ", (low + 0 > 0) ? "kept" : "not-kept"
            print (d + 0 > 0) ? "up" : (d + 0 < 0) ? "down" : "flat"
        }')
        # shellcheck disable=SC2086
        set -- $verdict
        if [ "$1" = kept ]; then keep=kept kept=true; else keep="not kept" kept=false; fi
        direction=$2
        # Step 7: the record.
        {
            echo "try: $result_name"
            echo "stage: confirm $result_variant"
            echo "spent: $result_spent"
            echo "transcript: $transcript"
            echo "used: $result_used of $result_sessions sessions used the $result_layer layer"
            echo "best so far: $best"
            echo "keep row: $keep_row"
            echo "base row: $base_row"
            [ -z "$result_probes" ] || printf '%s\n' "$result_probes"
            echo "keep: $keep"
            echo "direction: $direction"
            echo "rule: keep a tuning change only when the whole uncertainty range of the difference against the best so far sits above zero (owner, 2026-10-20, #1777, ruling 7)"
        } > "$try/result.md"
        ledger_py append event=s:result "try=s:$result_name" "layer=s:$result_layer" "scenario=s:$result_scenario" \
            stage=s:confirm "round=i:$result_n" "variant=s:$result_variant" \
            "spent=i:$result_spent" "transcript=s:$transcript" "best=s:$best" "keep_row=s:$keep_row" \
            "base_row=s:$base_row" "used=i:$result_used" "sessions=i:$result_sessions" "count=i:$result_jobs" \
            "kept=b:$kept" "direction=s:$direction" "at=s:$(now)"
        ledger_py append event=s:round "layer=s:$result_layer" "round=i:$result_n" "kept=b:$kept" \
            "try=s:$result_name" "at=s:$(now)"
        cat "$try/result.md"
        ;;

    ledger)
        ledger_py print
        ledger_total=$(spent_round) || exit $?
        echo "spent: $ledger_total"
        ;;

    accept)
        printf '%s' "$ruling" | grep -Eqx 'https://github\.com/[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+/(issues/[0-9]+#issuecomment-[0-9]+|pull/[0-9]+([#/?][A-Za-z0-9_./?=&#-]*)?)' || {
            say "\`$ruling\` is not the URL of the owner's ruling: a comment on an issue, https://github.com/<owner>/<repo>/issues/<n>#issuecomment-<id>, or a pull request, https://github.com/<owner>/<repo>/pull/<n>."
            exit 2
        }
        read_env 0
        read_pin
        base_check "$pin_scenario" "$pin_sha"
        if [ "$stale" != 1 ]; then
            echo "nothing to accept"
            exit 0
        fi
        accept_head=$(git -C "$repo" rev-parse HEAD 2>/dev/null)
        ledger_py append event=s:accept "base=s:$pin_scenario" "pin=s:$pin_sha" \
            "head=s:$accept_head" "ruling=s:$ruling" paths=t: "at=s:$(now)" <<EOF
$base_invalid
EOF
        printf '%s\n' "$base_invalid" | awk -F'\t' -v r="$ruling" '{ print "accept: " $1 " at " $2 " (" r ")" }'
        ;;
esac
exit 0
