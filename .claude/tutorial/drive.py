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

DOC = 'docs/tutorials/your-first-governed-corpus.md'
STATED_DATE = '2026-08-17'

failures = []
checks = 0


def read_blocks(root):
    """Every fenced code block of the tutorial body, in order."""
    lines = open(os.path.join(root, DOC)).read().split('\n')
    # Strip the front matter, which ends at the first `---` after the opening one.
    # A split on the delimiter would cut at the scaffolded front matter that step
    # 8 prints inside a fence, which is a `---` the document quotes rather than
    # one it uses.
    if lines and lines[0] == '---':
        lines = lines[lines.index('---', 1) + 1:]
    blocks, current, inside = [], [], False
    for line in lines:
        if line.strip() == '```':
            if inside:
                blocks.append('\n'.join(current))
                current = []
            inside = not inside
            continue
        if inside:
            current.append(line)
        elif line.startswith('    ') and line.strip():
            raise SystemExit(
                'tutorial: an indented code block is outside a fence, and CommonMark '
                'joins two of those into one <pre>: ' + line[:60])
    if inside:
        raise SystemExit('tutorial: an unclosed code fence')
    return blocks


def normalize(text, today):
    return [line.rstrip() for line in text.replace(today, STATED_DATE).strip('\n').split('\n')]


def compare(label, actual, stated, today, subset=False):
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

    blocks = read_blocks(root)
    if len(blocks) != 42:
        raise SystemExit(f'tutorial: expected 42 code blocks and found {len(blocks)}')

    today = datetime.date.today().isoformat()
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

        # Step 1.
        first = blocks[1].strip('\n').split('\n')
        run(first[0])
        cwd['at'] = os.path.join(scratch, 'headwater-tutorial')
        for command in first[2:]:
            run(command)
        compare('step 1: ls docs/decisions', run('ls docs/decisions').stdout, blocks[2], today)

        # Step 2.
        result = run(blocks[3].strip())
        compare('step 2: headwater init', result.stdout, blocks[4], today)
        compare('step 2: ls .headwater', run('ls .headwater').stdout,
                'overlay.yml\ntaxonomy.yml', today)

        # Step 3.
        run(blocks[5].strip())
        compare('step 3: ls packages/headwater-standard',
                run('ls packages/headwater-standard').stdout, blocks[6], today)

        # Step 4.
        result = run(blocks[7].strip())
        compare('step 4: the first refusal', result.stdout + result.stderr, blocks[8], today)
        assert_true('step 4: exit status 1', result.returncode == 1)

        # Step 5. The tutorial names the line to change rather than a command.
        path = os.path.join(cwd['at'], '.headwater/taxonomy.yml')
        source = open(path).read()
        assert_true('step 5: the line the tutorial names is in the file',
                    '  version: 0.0.0' in source)
        open(path, 'w').write(source.replace('  version: 0.0.0', '  version: 3.2.0'))
        compare("step 5: grep 'version:'",
                run("grep 'version:' .headwater/taxonomy.yml").stdout, blocks[9], today)
        result = run(blocks[10].strip())
        compare('step 5: the second refusal', result.stdout + result.stderr, blocks[11], today)
        assert_true('step 5: exit status 1', result.returncode == 1)

        # Step 6. The tutorial names the last line to replace.
        path = os.path.join(cwd['at'], '.headwater/overlay.yml')
        source = open(path).read()
        assert_true('step 6: the last line is `add: {}`', source.rstrip('\n').endswith('add: {}'))
        open(path, 'w').write(source.rstrip('\n')[:-len('add: {}')] + blocks[12].strip('\n') + '\n')
        compare('step 6: tail -2 .headwater/overlay.yml',
                run('tail -2 .headwater/overlay.yml').stdout, blocks[12], today)
        result = run(blocks[13].strip())
        compare('step 6: the lock is written', result.stdout + result.stderr, blocks[14], today)
        compare('step 6: ls .headwater/taxonomy.lock',
                run('ls .headwater/taxonomy.lock').stdout, '.headwater/taxonomy.lock', today)

        # Step 7.
        result = run(blocks[15].strip())
        compare('step 7: the census', result.stdout, blocks[16], today, subset=True)
        compare('step 7: the coverage line', result.stdout, blocks[17], today, subset=True)
        compare('step 7: the findings line', result.stdout, blocks[18], today, subset=True)
        compare('step 7: strict exit',
                run('headwater check --strict > /dev/null 2>&1; echo $?').stdout, '0', today)

        # Step 8.
        result = run(blocks[19].strip())
        compare('step 8: what the taxonomy decided', result.stdout, blocks[20], today, subset=True)
        compare('step 8: the document the verb wrote',
                run('cat docs/decisions/0001-store-attempts-in-postgres.md').stdout,
                blocks[21], today)

        # Step 9.
        result = run(blocks[22].strip())
        compare('step 9: the census', result.stdout, blocks[23], today, subset=True)
        compare('step 9: the coverage line', result.stdout, blocks[24], today, subset=True)
        compare('step 9: grep check instances',
                run("headwater check 2>/dev/null | grep 'check instances'").stdout,
                blocks[24], today)

        # Step 10.
        for command in blocks[25].strip('\n').split('\n'):
            run(command)
        log = run('git log --oneline').stdout.strip()
        assert_true('step 10: one commit',
                    len(log.split('\n')) == 1 and log.endswith('A first governed corpus'), log)

        # Step 11.
        result = run(blocks[26].strip())
        compare('step 11: wrote and edited', result.stdout, blocks[27], today, subset=True)
        compare('step 11: the edges it proposed', result.stdout, blocks[28], today, subset=True)
        compare('step 11: strict exit',
                run('headwater check --strict > /dev/null 2>&1; echo $?').stdout, '0', today)

        # Step 12.
        commands = blocks[29].strip('\n').split('\n')
        run(commands[0])
        result = run(commands[1])
        compare('step 12: the finding', result.stdout, blocks[30], today, subset=True)
        compare('step 12: strict exit',
                run('headwater check --strict > /dev/null 2>&1; echo $?').stdout, '1', today)

        # Step 13.
        result = run(blocks[31].strip())
        compare('step 13: the fix account, on standard error',
                result.stderr, blocks[32], today, subset=True)
        compare('step 13: strict exit',
                run('headwater check --strict > /dev/null 2>&1; echo $?').stdout, '0', today)
        compare('step 13: the repaired front matter',
                run("grep -A2 '^relations:' docs/decisions/"
                    "0001-store-attempts-in-postgres.md").stdout, blocks[33], today)

        # Step 14.
        compare('step 14: explain', run(blocks[34].strip()).stdout, blocks[35], today)

        # Step 15.
        compare('step 15: route', run(blocks[36].strip()).stdout, blocks[37], today)

        # Step 16.
        result = run(blocks[38].strip())
        compare('step 16: the levels', result.stdout, blocks[39], today, subset=True)
        for command in blocks[40].strip('\n').split('\n'):
            run(command)
        compare('step 16: projections.current met',
                run("headwater conformance 2>/dev/null | grep 'projections.current'").stdout,
                '  projections.current met', today)

        # Where to go next.
        assert_true('where to go next: headwater infer exits 0',
                    run(blocks[41].strip()).returncode == 0)
    finally:
        shutil.rmtree(scratch, ignore_errors=True)

    print()
    print(f'{checks} claims, {len(failures)} failed')
    for failure in failures:
        print(' - ' + failure)
    return 1 if failures else 0


if __name__ == '__main__':
    sys.exit(main())
