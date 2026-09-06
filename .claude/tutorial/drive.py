#!/usr/bin/env python3
"""Follow the tutorial, and compare every stated output with what the verb printed.

`docs/tutorials/your-first-governed-corpus.md` states, after every step, what the
reader should now see. Each of those is a claim about this engine, and a claim in
prose is a wait stated by a string literal. This script is the verb that produces
them: it reads the commands out of the document, runs them against a scratch
repository, and diffs the result against the block the document prints.

Three properties are deliberate.

**The commands come out of the rendered document rather than out of a shell
history.** Two indented code blocks separated by a blank line are one `<pre>` in
CommonMark, so a document can present two commands as one undifferentiated paste
with no visible seam. Every code block in the tutorial is fenced, for which the
source and the rendered form agree by construction, and `assert_all_fenced`
refuses an indented block so that the property stays true.

**Nothing here writes inside the checkout.** The scratch repository goes under a
temporary directory that this script removes at the end, and `HOME` is redirected
into it so that a `~` in a tutorial command cannot escape.

**The first block is the one command this script does not run.** It clones the
repository and builds the engine, which is not a claim about the engine's output.
The runner supplies a built binary on `PATH` instead, and the tutorial's own check
for that block is the one assertion kept.

Run it through `.claude/tutorial/fixtures.sh`, which locates the binary.
"""

import datetime
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time

DOC = 'docs/tutorials/your-first-governed-corpus.md'
STATED_DATE = '2026-08-17'

failures = []
checks = 0


def today_reading(clock=time.time):
    """The date this run substitutes, on the same clock the engine stamps.

    `headwater_check::Context::from_system_clock`
    (`engine/crates/check/src/context.rs:93`) reads
    `SystemTime::now().duration_since(UNIX_EPOCH).as_secs() / 86_400` — the
    whole day count since the Unix epoch, in UTC, with no zone applied at
    all. `datetime.date.today()` reads the process's *local* calendar
    instead, which names a different day than the engine's for part of every
    day in any zone ahead of UTC — `Australia/Brisbane`, this machine's own
    zone, included. So this function mirrors the engine's arithmetic on the
    epoch second count rather than reading a local clock of any kind, and the
    two can no longer disagree.

    `clock` defaults to the real one and takes an injected one only so that
    `clock_reads_the_engine_s_day_and_not_the_local_one`, below, can hold
    this exact function to a fixed instant rather than wait for the real
    clock to reach an hour where a regression would show.
    """
    days = int(clock()) // 86_400
    return (datetime.date(1970, 1, 1) + datetime.timedelta(days=days)).isoformat()


def clock_reads_the_engine_s_day_and_not_the_local_one():
    """The regression case for #302, held by behavior rather than by having
    been fixed once: reverting `today_reading` to read `datetime.date.today()`
    makes this fail, deterministically, on whatever day it happens to run.

    `2026-01-01T12:00:00Z` is a fixed instant that falls on 2026-01-01 in UTC
    and on 2026-01-02 under `Pacific/Kiritimati` (UTC+14, the zone furthest
    ahead of UTC there is, and fixed year-round — no DST to add a second
    variable). The two readings provably differ under it, on any day this
    check itself runs.
    """
    instant = datetime.datetime(2026, 1, 1, 12, 0, 0, tzinfo=datetime.timezone.utc).timestamp()
    expect_utc = '2026-01-01'
    previous = os.environ.get('TZ')
    os.environ['TZ'] = 'Pacific/Kiritimati'
    time.tzset()
    try:
        local_reading = datetime.date.fromtimestamp(instant).isoformat()
        got = today_reading(clock=lambda: instant)
    finally:
        if previous is None:
            os.environ.pop('TZ', None)
        else:
            os.environ['TZ'] = previous
        time.tzset()
    assert local_reading != expect_utc, (
        'the fixed instant no longer diverges under Pacific/Kiritimati, so this '
        'proves nothing; pick another instant')
    detail = (f'today_reading said {got!r}; the engine (UTC) says {expect_utc!r}; '
              f'a local read under Pacific/Kiritimati said {local_reading!r}')
    return got == expect_utc, detail


def read_today_under_a_hostile_zone():
    """Call the real production expression once, with the process's zone
    forced to one deliberately ahead of UTC for the read.

    `clock_reads_the_engine_s_day_and_not_the_local_one`, above, holds
    `today_reading` itself to a fixed instant, and never touches the
    statement that calls it. That leaves one gap open: a future edit that
    reverts the call site back to a local-clock read
    (`datetime.date.today().isoformat()`, as it read before #302) rather
    than editing `today_reading`, would pass that fixture unchanged, since
    the fixture never runs the call site at all. On a UTC CI runner such a
    revert is invisible on top of that, because the runner's own zone never
    diverges from UTC by itself — which is exactly why #302 went unnoticed
    by CI in the first place. Forcing a hostile zone around this read, here,
    makes that revert diverge from UTC regardless of the machine's real
    zone, CI included.

    Returns `(today, expect_utc)` for the caller to assert on. Built from
    the real clock rather than a fixed instant, on purpose: it means to run
    the literal call site as production runs it, not a stand-in for it. The
    trade-off is a race, on the order of microseconds, if the read crosses a
    UTC day boundary between the two calls below — not eliminated, but far
    too small to be a source of a flaky run in practice.
    """
    previous = os.environ.get('TZ')
    os.environ['TZ'] = 'Pacific/Kiritimati'
    time.tzset()
    try:
        today = today_reading()
    finally:
        if previous is None:
            os.environ.pop('TZ', None)
        else:
            os.environ['TZ'] = previous
        time.tzset()
    expect_utc = datetime.datetime.now(datetime.timezone.utc).date().isoformat()
    return today, expect_utc


def read_blocks(root):
    """Every fenced code block of the tutorial body, and whether it is marked trimmed.

    The tutorial states its own convention in *Before you start*: "A block that
    is shorter than the real output says so on the line above it." A block is
    marked when the nearest prose paragraph above it opens with `Trimmed`, and
    one marker covers the run of fences under it, which is how steps 7, 9, 11
    and 12 show two or three cuts of one run.

    The flag is what `compare` holds the page to. A comparison that passes on a
    subset of the real output is a claim that the block is a cut of it, and a cut
    the page does not declare is the page breaking its own rule. That is the one
    thing sixteen steps of substring matching cannot otherwise see.
    """
    lines = open(os.path.join(root, DOC)).read().split('\n')
    # Strip the front matter, which ends at the first `---` after the opening one.
    # A split on the delimiter would cut at the scaffolded front matter that step
    # 8 prints inside a fence, which is a `---` the document quotes rather than
    # one it uses.
    if lines and lines[0] == '---':
        lines = lines[lines.index('---', 1) + 1:]
    blocks, trimmed, current, inside, paragraph = [], [], [], False, ''
    for line in lines:
        if line.strip() == '```':
            if inside:
                blocks.append('\n'.join(current))
                trimmed.append(paragraph.lstrip('*').lower().startswith('trimmed'))
                current = []
            inside = not inside
            continue
        if inside:
            current.append(line)
            continue
        if line.startswith('    ') and line.strip():
            raise SystemExit(
                'tutorial: an indented code block is outside a fence, and CommonMark '
                'joins two of those into one <pre>: ' + line[:60])
        if line.strip():
            paragraph = line.strip()
    if inside:
        raise SystemExit('tutorial: an unclosed code fence')
    return blocks, trimmed


def normalize(text, today):
    return [line.rstrip() for line in text.replace(today, STATED_DATE).strip('\n').split('\n')]


def compare(label, actual, stated, today, subset=False, index=None, trimmed=None):
    global checks
    checks += 1
    got, want = normalize(actual, today), normalize(stated, today)
    ok = all(line in got for line in want) if subset else got == want
    print(('ok   ' if ok else 'FAIL ') + label)
    if not ok:
        failures.append(label)
        print('  the tutorial states:')
        for line in want:
            print('    | ' + line)
        print('  the run printed:')
        for line in got[:40]:
            print('    | ' + line)
    # A subset match means the block is a cut of the real output, so the page owes
    # the reader the marker its own convention promises.
    if subset and index is not None and trimmed is not None:
        assert_true(f'{label}: the page declares this block trimmed', trimmed[index],
                    'the block matches part of the output and no `Trimmed` line stands above it')


def assert_true(label, condition, detail=''):
    global checks
    checks += 1
    print(('ok   ' if condition else 'FAIL ') + label)
    if not condition:
        failures.append(label)
        if detail:
            print('  ' + detail)


def main():
    root = os.path.abspath(sys.argv[1]) if len(sys.argv) > 1 else os.getcwd()
    binary = os.environ.get('HEADWATER_BIN') or os.path.join(root, 'engine/target/release/headwater')
    if not os.access(binary, os.X_OK):
        raise SystemExit('tutorial: no engine at ' + binary)

    blocks, trimmed = read_blocks(root)
    if len(blocks) != 45:
        raise SystemExit(f'tutorial: expected 45 code blocks and found {len(blocks)}')

    ok, detail = clock_reads_the_engine_s_day_and_not_the_local_one()
    assert_true("the substitution clock reads the engine's UTC day, not the local one", ok, detail)

    # #302: today's date must come from today_reading(), never
    # datetime.date.today() directly — see that function's docstring for why.
    today, expect_utc = read_today_under_a_hostile_zone()
    assert_true("the call site reads the engine's UTC day under a hostile local zone",
                today == expect_utc,
                f'the call site read {today!r} with the zone forced to Pacific/Kiritimati; '
                f'the UTC date at the same moment is {expect_utc!r}')
    scratch = tempfile.mkdtemp(prefix='headwater-tutorial-')
    env = dict(os.environ)
    env['PATH'] = os.path.dirname(binary) + os.pathsep + env['PATH']
    env['HEADWATER_SRC'] = root
    env['HOME'] = scratch
    env['GIT_AUTHOR_NAME'] = env['GIT_COMMITTER_NAME'] = 'You'
    env['GIT_AUTHOR_EMAIL'] = env['GIT_COMMITTER_EMAIL'] = 'you@example.com'
    cwd = {'at': scratch}

    def run(command):
        return subprocess.run(['bash', '-c', command], cwd=cwd['at'], env=env,
                              capture_output=True, text=True)

    try:
        # Before you start. The clone and the build are the one block not run.
        assert_true('before you start: headwater is on the path',
                    run('command -v headwater').stdout.strip().endswith(
                        'engine/target/release/headwater'))

        def cut(label, actual, index):
            """A block the page presents as a cut of the real output."""
            compare(label, actual, blocks[index], today, subset=True,
                    index=index, trimmed=trimmed)

        def whole(label, actual, index):
            """A block the page presents as the whole of what a command printed."""
            compare(label, actual, blocks[index], today)

        # Step 1.
        first = blocks[1].strip('\n').split('\n')
        run(first[0])
        cwd['at'] = os.path.join(scratch, 'headwater-tutorial')
        for command in first[2:]:
            run(command)
        whole('step 1: ls docs/decisions', run('ls docs/decisions').stdout, 2)

        # Step 2.
        whole('step 2: headwater init', run(blocks[3].strip()).stdout, 4)
        compare('step 2: ls .headwater', run('ls .headwater').stdout,
                'overlay.yml\ntaxonomy.yml', today)

        # Step 3.
        run(blocks[5].strip())
        whole('step 3: ls packages/headwater-standard',
              run('ls packages/headwater-standard').stdout, 6)

        # Step 4.
        result = run(blocks[7].strip())
        whole('step 4: the first refusal', result.stdout + result.stderr, 8)
        assert_true('step 4: exit status 1', result.returncode == 1)

        # One `match` arm of `headwater init` emits two messages under the same
        # condition: the report line that step 2 above compares whole, and the
        # comment it writes above `version: 0.0.0`. #276 found them drifted apart
        # — the printed one naming two routes and the written one naming a route
        # an adopter cannot take — because the printed one is held here and the
        # written one was held by nothing. The page now says what the comment
        # says, so these hold the half that had no reader. Their proper home is a
        # test of the CLI crate, which has none: #174.
        declaration = open(os.path.join(cwd['at'], '.headwater/taxonomy.yml')).read()
        for claim, token in [
                ('the copy route', 'Copy a package directory into `packages/`'),
                ('the vendor route', '`headwater taxonomy vendor <dir>`'),
                ('where the version comes from',
                 'the version that the package itself declares')]:
            assert_true('step 4: what init wrote names ' + claim, token in declaration)

        # Step 5. The tutorial names the line to change rather than a command,
        # so this makes the edit the reader would make by hand.
        #
        # The new line is read out of block 9, which is the output the page says
        # `grep 'version:'` prints after the edit, and never typed here. A
        # literal in this file would be a second copy of the package version,
        # and #427 bumped the package to 4.0.0 and moved six mentions in the
        # tutorial while this one stayed at 3.5.0, so every step from 5 to 16
        # failed on a tutorial that was right.
        path = os.path.join(cwd['at'], '.headwater/taxonomy.yml')
        source = open(path).read()
        assert_true('step 5: the line the tutorial names is in the file',
                    '  version: 0.0.0' in source)
        pinned = blocks[9].strip('\n')
        assert_true('step 5: the page states the version to pin',
                    pinned.startswith('  version: ') and pinned != '  version: 0.0.0',
                    pinned)
        open(path, 'w').write(source.replace('  version: 0.0.0', pinned))
        whole("step 5: grep 'version:'",
              run("grep 'version:' .headwater/taxonomy.yml").stdout, 9)
        result = run(blocks[10].strip())
        whole('step 5: the second refusal', result.stdout + result.stderr, 11)
        assert_true('step 5: exit status 1', result.returncode == 1)

        # Step 6. The tutorial names the last line to replace.
        path = os.path.join(cwd['at'], '.headwater/overlay.yml')
        source = open(path).read()
        assert_true('step 6: the last line is `add: {}`', source.rstrip('\n').endswith('add: {}'))
        open(path, 'w').write(source.rstrip('\n')[:-len('add: {}')] + blocks[12].strip('\n') + '\n')
        whole('step 6: tail -2 .headwater/overlay.yml',
              run('tail -2 .headwater/overlay.yml').stdout, 12)
        result = run(blocks[13].strip())
        whole('step 6: the lock is written', result.stdout + result.stderr, 14)
        compare('step 6: ls .headwater/taxonomy.lock',
                run('ls .headwater/taxonomy.lock').stdout, '.headwater/taxonomy.lock', today)

        # Step 7.
        result = run(blocks[15].strip())
        cut('step 7: the census', result.stdout, 16)
        cut('step 7: the coverage line', result.stdout, 17)
        cut('step 7: the findings line', result.stdout, 18)
        compare('step 7: strict exit',
                run('headwater check --strict > /dev/null 2>&1; echo $?').stdout, '0', today)

        # Step 8.
        result = run(blocks[19].strip())
        cut('step 8: what the taxonomy decided', result.stdout, 20)
        whole('step 8: the document the verb wrote',
              run('cat docs/decisions/0001-store-attempts-in-postgres.md').stdout, 21)

        # Step 9.
        result = run(blocks[22].strip())
        cut('step 9: the census', result.stdout, 23)
        cut('step 9: the coverage line', result.stdout, 24)
        whole('step 9: grep check instances',
              run("headwater check 2>/dev/null | grep 'check instances'").stdout, 24)

        # Step 10. The ignore file is written before the commit, so the cache stays out.
        for command in blocks[25].strip('\n').split('\n'):
            run(command)
        log = run('git log --oneline').stdout.strip()
        assert_true('step 10: one commit',
                    len(log.split('\n')) == 1 and log.endswith('A first governed corpus'), log)
        whole('step 10: git ls-files .headwater', run('git ls-files .headwater').stdout, 26)
        # The step tells the reader to ignore what this repository ignores, and it
        # says so in the paragraph under it. Hold the two to each other rather than
        # to a copy: an ignore rule that moves in `.gitignore` and not on the page
        # is the second-copy defect the page is teaching against.
        line = blocks[25].strip('\n').split('\n')[0]
        prefix, suffix = "printf '", "\\n' > .gitignore"
        named = line.startswith(prefix) and line.endswith(suffix)
        rule = line[len(prefix):-len(suffix)] if named else None
        assert_true('step 10: the page ignores what this repository ignores',
                    named and rule in open(os.path.join(root, '.gitignore')).read().split('\n'),
                    f'the page writes `{rule}` and `.gitignore` here does not carry that line')

        # Step 11.
        result = run(blocks[27].strip())
        cut('step 11: wrote and edited', result.stdout, 28)
        cut('step 11: the edges it proposed', result.stdout, 29)
        compare('step 11: strict exit',
                run('headwater check --strict > /dev/null 2>&1; echo $?').stdout, '0', today)

        # Step 12.
        commands = blocks[30].strip('\n').split('\n')
        run(commands[0])
        result = run(commands[1])
        cut('step 12: the finding', result.stdout, 31)
        cut('step 12: the head of the register', result.stdout, 32)
        cut('step 12: every rule reaches one obligation', result.stdout, 33)
        # The page's own check names the `27 obligations:` line of the register block,
        # which is the second line of it.
        compare('step 12: grep obligations:',
                run("headwater check 2>/dev/null | grep 'obligations:'").stdout,
                blocks[32].strip('\n').split('\n')[1], today)
        compare('step 12: strict exit',
                run('headwater check --strict > /dev/null 2>&1; echo $?').stdout, '1', today)

        # Step 13.
        result = run(blocks[34].strip())
        cut('step 13: the fix account, on standard error', result.stderr, 35)
        compare('step 13: strict exit',
                run('headwater check --strict > /dev/null 2>&1; echo $?').stdout, '0', today)
        whole('step 13: the repaired front matter',
              run("grep -A2 '^relations:' docs/decisions/"
                  "0001-store-attempts-in-postgres.md").stdout, 36)

        # Step 14.
        whole('step 14: explain', run(blocks[37].strip()).stdout, 38)

        # Step 15.
        whole('step 15: route', run(blocks[39].strip()).stdout, 40)

        # Step 16.
        result = run(blocks[41].strip())
        cut('step 16: the levels', result.stdout, 42)
        for command in blocks[43].strip('\n').split('\n'):
            run(command)
        compare('step 16: projections.current met',
                run("headwater conformance 2>/dev/null | grep 'projections.current'").stdout,
                '  projections.current met', today)

        # Where to go next.
        assert_true('where to go next: headwater infer exits 0',
                    run(blocks[44].strip()).returncode == 0)
    finally:
        shutil.rmtree(scratch, ignore_errors=True)

    print()
    print(f'{checks} claims, {len(failures)} failed')
    for failure in failures:
        print(' - ' + failure)
    return 1 if failures else 0


if __name__ == '__main__':
    sys.exit(main())
