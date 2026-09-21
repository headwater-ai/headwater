"""The Python side of the one home for "which built engine answers".

`tools/repo/resolve-engine.sh` states the rule and the reason in full: prefer
the newer of `engine/target/release/headwater` and
`engine/target/dev-release/headwater` under a given root, and report only
when neither exists, so that a `dev-release`-only worktree is found rather
than silently skipped (#647). This module is the same rule for the one
consumer here that is not `sh`: `.claude/tutorial/drive.py` cannot source a
shell file, so it imports this instead of writing the check a third time in a
third syntax.
"""

import os


def resolve_engine_bin(root):
    """The executable to run, or None when neither profile is built."""
    release = os.path.join(root, 'engine', 'target', 'release', 'headwater')
    dev_release = os.path.join(root, 'engine', 'target', 'dev-release', 'headwater')
    engine = release if os.access(release, os.X_OK) else None
    if os.access(dev_release, os.X_OK):
        if engine is None or os.path.getmtime(dev_release) > os.path.getmtime(engine):
            engine = dev_release
    return engine


def missing_message(root):
    """The line a caller prints when `resolve_engine_bin` returns `None`."""
    release = os.path.join(root, 'engine', 'target', 'release', 'headwater')
    dev_release = os.path.join(root, 'engine', 'target', 'dev-release', 'headwater')
    return 'no engine at {} or {}'.format(release, dev_release)
