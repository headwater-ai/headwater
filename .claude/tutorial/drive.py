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

**One command reaches the network, and it is step 3's.** Step 3 curls
`tools/headwater-bootstrap.sh` off the default branch and pipes it into `sh`,
against the real `v0.1.0` release, the same tag `README.md` pins. Every assertion
before it runs first and reaches no network, so a fetch failure here is reported
as its own claim and never read as a defect earlier. Nothing after step 3 reaches
the network again.

**The first block is the one command this script does not run.** It installs the
engine with `cargo install`, which is not a claim about the engine's output. The
runner supplies a built binary on `PATH` instead, and the tutorial's own check
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
STATED_DATE = '2026-09-09'

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
    if len(blocks) != 46:
        raise SystemExit(f'tutorial: expected 46 code blocks and found {len(blocks)}')

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
    env['HOME'] = scratch
    env['GIT_AUTHOR_NAME'] = env['GIT_COMMITTER_NAME'] = 'You'
    env['GIT_AUTHOR_EMAIL'] = env['GIT_COMMITTER_EMAIL'] = 'you@example.com'
    cwd = {'at': scratch}

    def run(command):
        return subprocess.run(['bash', '-c', command], cwd=cwd['at'], env=env,
                              capture_output=True, text=True)

    try:
        # Before you start. `cargo install` is the one block not run.
        assert_true('before you start: headwater --version prints a number',
                    re.match(r'^\d+\.\d+\.\d+', run('headwater --version').stdout.strip()) is not None)

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

        # Step 3. The one command in this suite that reaches the network: it
        # curls `tools/headwater-bootstrap.sh` off the default branch and pipes
        # it into `sh`, against the real `v0.1.0` release. A fetch or network
        # failure here is reported as its own claim, never read as a defect in
        # a step above it, and nothing below this step reaches the network
        # again.
        result = run(blocks[5].strip())
        cut('step 3: the account of what the script fetched', result.stdout + result.stderr, 6)
        whole('step 3: ls packages/headwater-standard',
              run('ls packages/headwater-standard').stdout, 7)

        # Step 4.
        result = run(blocks[8].strip())
        whole('step 4: the first refusal', result.stdout + result.stderr, 9)
        assert_true('step 4: exit status 1', result.returncode == 1)

        # One `match` arm of `headwater init` emits two messages under the same
        # condition: the report line that step 2 above compares whole, and the
        # comment it writes above `version: 0.0.0`. #276 found them drifted apart
        # — the printed one naming two routes and the written one naming a route
        # an adopter cannot take — because the printed one is held here and the
        # written one was held by nothing. The page now says what the comment
        # says, so these hold the half that had no reader. Their proper home is
        # `engine/crates/cli/tests/init.rs`, which holds both written files byte
        # for byte; these stay because they read the tutorial's own scratch tree
        # and because a drift between the page and the file is what #276 was.
        declaration = open(os.path.join(cwd['at'], '.headwater/taxonomy.yml')).read()
        for claim, token in [
                ('the copy route', 'Copy a package directory into `packages/`'),
                ('the vendor route', '`headwater taxonomy vendor <dir>`'),
                ('where the version comes from',
                 'the version that package declares'),
                ('the field the vendor route needs first', '# digest:')]:
            assert_true('step 4: what init wrote names ' + claim, token in declaration)

        # Step 5. The tutorial names the two lines to change rather than a
        # command, so this makes the edit the reader would make by hand.
        #
        # The new lines are read out of block 10, which is the output the page
        # says `grep -E 'digest:|version:' .headwater/taxonomy.yml` prints after
        # the edit, and never typed here. A literal in this file would be a
        # second copy of the package version and the release digest, and #427
        # bumped the package to 4.0.0 and moved six mentions in the tutorial
        # while this one stayed at 3.5.0, so every step from 5 to 16 failed on a
        # tutorial that was right.
        path = os.path.join(cwd['at'], '.headwater/taxonomy.yml')
        source = open(path).read()
        assert_true('step 5: the lines the tutorial names are in the file',
                    '  # digest: sha256:<the digest the publisher printed>' in source
                    and '  version: 0.0.0' in source)
        lines = blocks[10].strip('\n').split('\n')
        assert_true('step 5: the page states the digest and the version to pin',
                    len(lines) == 2
                    and lines[0].startswith('  digest: sha256:')
                    and lines[1].startswith('  version: ') and lines[1] != '  version: 0.0.0',
                    blocks[10])
        source = source.replace(
            '  # digest: sha256:<the digest the publisher printed>\n  version: 0.0.0',
            lines[0] + '\n' + lines[1])
        open(path, 'w').write(source)
        whole("step 5: grep -E 'digest:|version:'",
              run("grep -E 'digest:|version:' .headwater/taxonomy.yml").stdout, 10)
        result = run(blocks[11].strip())
        whole('step 5: the second refusal', result.stdout + result.stderr, 12)
        assert_true('step 5: exit status 1', result.returncode == 1)

        # Step 6. The tutorial names the last line to replace.
        path = os.path.join(cwd['at'], '.headwater/overlay.yml')
        source = open(path).read()
        assert_true('step 6: the last line is `add: {}`', source.rstrip('\n').endswith('add: {}'))
        open(path, 'w').write(source.rstrip('\n')[:-len('add: {}')] + blocks[13].strip('\n') + '\n')
        whole('step 6: tail -2 .headwater/overlay.yml',
              run('tail -2 .headwater/overlay.yml').stdout, 13)
        result = run(blocks[14].strip())
        whole('step 6: the lock is written', result.stdout + result.stderr, 15)
        compare('step 6: ls .headwater/taxonomy.lock',
                run('ls .headwater/taxonomy.lock').stdout, '.headwater/taxonomy.lock', today)

        # Step 7.
        result = run(blocks[16].strip())
        cut('step 7: the census', result.stdout, 17)
        cut('step 7: the coverage line', result.stdout, 18)
        cut('step 7: the findings line', result.stdout, 19)
        compare('step 7: strict exit',
                run('headwater check --strict > /dev/null 2>&1; echo $?').stdout, '0', today)

        # Step 8.
        result = run(blocks[20].strip())
        cut('step 8: what the taxonomy decided', result.stdout, 21)
        whole('step 8: the document the verb wrote',
              run('cat docs/decisions/0001-store-attempts-in-postgres.md').stdout, 22)

        # Step 9.
        result = run(blocks[23].strip())
        cut('step 9: the census', result.stdout, 24)
        cut('step 9: the coverage line', result.stdout, 25)
        whole('step 9: grep check instances',
              run("headwater check 2>/dev/null | grep 'check instances'").stdout, 25)

        # Step 10. `headwater check` already wrote the cache's own ignore file
        # by step 9, so this step is one commit and no more.
        for command in blocks[26].strip('\n').split('\n'):
            run(command)
        log = run('git log --oneline').stdout.strip()
        assert_true('step 10: one commit',
                    len(log.split('\n')) == 1 and log.endswith('A first governed corpus'), log)
        whole('step 10: git ls-files .headwater', run('git ls-files .headwater').stdout, 27)
        # Held to the file on disk rather than to a copy of it: the pattern the
        # page claims the run wrote is read back from the run's own tree.
        ignore = os.path.join(cwd['at'], '.headwater/cache/.gitignore')
        compare('step 10: the cache excludes itself', open(ignore).read(),
                '*\n!.gitignore\n', today)

        # Step 11.
        result = run(blocks[28].strip())
        cut('step 11: wrote and edited', result.stdout, 29)
        cut('step 11: the edges it proposed', result.stdout, 30)
        compare('step 11: strict exit',
                run('headwater check --strict > /dev/null 2>&1; echo $?').stdout, '0', today)

        # Step 12.
        commands = blocks[31].strip('\n').split('\n')
        run(commands[0])
        result = run(commands[1])
        cut('step 12: the finding', result.stdout, 32)
        cut('step 12: the head of the register', result.stdout, 33)
        cut('step 12: every rule reaches one obligation', result.stdout, 34)
        # The page's own check names the `31 obligations:` line of the register block,
        # which is the second line of it.
        compare('step 12: grep obligations:',
                run("headwater check 2>/dev/null | grep 'obligations:'").stdout,
                blocks[33].strip('\n').split('\n')[1], today)
        compare('step 12: strict exit',
                run('headwater check --strict > /dev/null 2>&1; echo $?').stdout, '1', today)

        # Step 13.
        result = run(blocks[35].strip())
        cut('step 13: the fix account, on standard error', result.stderr, 36)
        compare('step 13: strict exit',
                run('headwater check --strict > /dev/null 2>&1; echo $?').stdout, '0', today)
        whole('step 13: the repaired front matter',
              run("grep -A2 '^relations:' docs/decisions/"
                  "0001-store-attempts-in-postgres.md").stdout, 37)

        # Step 14.
        whole('step 14: explain', run(blocks[38].strip()).stdout, 39)

        # Step 15.
        whole('step 15: route', run(blocks[40].strip()).stdout, 41)

        # Step 16. Pinning the digest in step 5 already carried this corpus onto
        # `L0` and `L1`, unlike the copy route the page used to take, so the
        # only gap left here is `projections.current`.
        result = run(blocks[42].strip())
        cut('step 16: the levels', result.stdout, 43)
        for command in blocks[44].strip('\n').split('\n'):
            run(command)
        compare("step 16: grep 'L2'",
                run("headwater conformance 2>/dev/null | grep 'L2'").stdout,
                '  L2 Regenerated — reached, 4 of 4 rules met\n'
                'L2 reached, against headwater/standard 4.1.0', today)

        # Where to go next.
        assert_true('where to go next: headwater infer exits 0',
                    run(blocks[45].strip()).returncode == 0)
    finally:
        shutil.rmtree(scratch, ignore_errors=True)

    print()
    print(f'{checks} claims, {len(failures)} failed')
    for failure in failures:
        print(' - ' + failure)
    return 1 if failures else 0


if __name__ == '__main__':
    sys.exit(main())
