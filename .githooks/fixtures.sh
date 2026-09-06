#!/bin/sh
# What holds the commit gate and the producer under it.
#
# Spec 12 refuses a check that ships with no failing fixture, and the reason
# carries to a gate: a gate that has refused nothing is a gate nobody has seen
# work. Every refusal below is provoked on purpose, and so is every pass, because
# a gate that refuses everything is as useless as one that refuses nothing.
#
# Run it from anywhere:
#     sh .githooks/fixtures.sh
#
# # It runs over this corpus and not over a fixture tree
#
# `lifecycle.transition.not_permitted` reads the lifecycle regime a kind binds,
# and this repository binds two. A fixture taxonomy would test the rule, which
# `engine/crates/check/tests/` already does. What has never been tested is the
# path from a commit of *this* repository to that rule, so the tree below is
# this one: every tracked and untracked file, copied into a scratch repository
# whose one commit is the corpus as it stands. The prior version a case reads is
# then a real `HEAD` of a real git repository.
#
# The built engine is copied in rather than found, because the hook reads
# `engine/target/release/headwater` under the root git reports, and the scratch
# repository has its own root. `engine/target/` is ignored there, as it is here.
#
# It needs a built engine and it says so and stops when there is none. It writes
# only under a temporary directory, and it removes it on an interrupt.

set -u

root=$(cd "$(dirname "$0")/.." && pwd)
engine="$root/engine/target/release/headwater"

if [ ! -x "$engine" ]; then
    echo "no built engine, so nothing here can run."
    echo "  cargo build --release -p headwater-cli --manifest-path engine/Cargo.toml"
    exit 1
fi

scratch=$(mktemp -d) || exit 1
# The producer writes outside the repository it describes, which is what the
# hook does with `mktemp -d`. A holding directory inside the tree would be an
# untracked file, which this producer correctly names as one the change adds.
hold=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch" "$hold"' EXIT HUP INT TERM

git -C "$root" ls-files -co --exclude-standard -z \
    | tar -C "$root" --null -T - -cf - \
    | tar -C "$scratch" -xf - || exit 1
mkdir -p "$scratch/engine/target/release"
cp "$engine" "$scratch/engine/target/release/headwater"
(
    cd "$scratch" || exit 1
    git init -q
    git add -A
    git -c user.name=fixtures -c user.email=fixtures@invalid commit -qm "the corpus as it stands" --no-verify
) || exit 1

passed=0
failed=0

# The two documents these cases move, named once. The first is seeded to
# `draft`, which admits `current` and `deprecated` and not `discharged`. The
# second stands at `discharged`, which the `obligation` regime gives no exit at
# all.
draft="docs/obligations/0126-every-asserted-document-carries-the-freshness-date-that-spec-3-says-it-cannot.md"
terminal="docs/obligations/0117-a-cached-verdict-about-an-anchor-survives-the-change-that-falsifies-it.md"

reset() {
    git -C "$scratch" reset -q --hard HEAD
    git -C "$scratch" clean -qfd
}

move() {
    file=$scratch/$1
    from=$2
    to=$3
    sed -i "s/^status: $from\$/status: $to/" "$file"
    grep -q "^status: $to\$" "$file" || {
        printf 'FAIL setup: %s did not move from %s to %s\n' "$1" "$from" "$to"
        exit 1
    }
}

# No document of this corpus stands at `draft`. HW-DR-0052 rules that an author
# writes the state a document will hold once the branch lands, so a merged
# document stands at `current` and the population these cases used to borrow
# from is empty. The producer reads the prior state out of `HEAD`, so a case
# that moves a document away from `draft` needs a committed `draft` version.
# This commits one, which is what the cases below were always really asking for.
seed_draft() {
    sed -i "s/^status: current\$/status: draft/" "$scratch/$draft"
    grep -q '^status: draft$' "$scratch/$draft" || {
        printf 'FAIL setup: %s did not seed to draft\n' "$draft"
        exit 1
    }
    git -C "$scratch" -c user.name=fixtures -c user.email=fixtures@invalid \
        commit -qam "seed a draft version" --no-verify
}

# Run the commit gate in the scratch repository, the way git runs it.
gate() {
    (cd "$scratch" && sh .githooks/pre-commit 2>&1)
}

# Run the producer alone, and print the manifest it wrote.
produce() {
    rm -rf "$hold/out" "$hold/why"
    (cd "$scratch" && sh .githooks/change-manifest "$1" "$hold/out" >/dev/null 2>"$hold/why")
}

# The text match is over the words and not over the line breaks.
#
# `headwater check` lays its report out at 80 columns, so a finding message this
# engine composed as one sentence reaches a reader over two or three lines, and
# an expectation written here as one string would span a fold point. Both sides
# have every whitespace run squeezed to one space before they are compared, so a
# case states what the gate says and never where the fill broke it. The failure
# message prints the output as it arrived.
judge() {
    name=$1 want_status=$2 got_status=$3 want_text=$4 got_text=$5
    if [ "$got_status" -ne "$want_status" ]; then
        printf 'FAIL %s\n  expected exit %s, got %s:\n%s\n' "$name" "$want_status" "$got_status" "$got_text"
        failed=$((failed + 1))
        return
    fi
    flat_want=$(printf '%s' "$want_text" | tr -s '[:space:]' ' ')
    flat_got=$(printf '%s' "$got_text" | tr -s '[:space:]' ' ')
    case $want_text in
        "") ;;
        *)
            case $flat_got in
                *"$flat_want"*) ;;
                *)
                    printf 'FAIL %s\n  expected output to hold: %s\n  got:\n%s\n' "$name" "$want_text" "$got_text"
                    failed=$((failed + 1))
                    return
                    ;;
            esac
            ;;
    esac
    printf 'ok   %s\n' "$name"
    passed=$((passed + 1))
}

printf '# the gate, over this corpus\n'

reset
seed_draft
move "$draft" draft discharged
out=$(gate); status=$?
judge 'a movement the regime does not admit is refused at the commit' 1 "$status" \
    'lifecycle.transition.not_permitted (OB-LIFE-1): HW-OBL-0126 moved from `draft` to `discharged`' "$out"

# The instrument, and it is the whole reason the case above means anything. The
# same tree, the same engine, and no manifest: the rule reports a skip and the
# gate exits 0. So the refusal above comes from the producer and from nothing
# else in the tree.
out=$(cd "$scratch" && ./engine/target/release/headwater check --strict 2>&1); status=$?
judge 'the same tree with no change described is not refused, and says why' 0 "$status" \
    '' "$out"
out=$(cd "$scratch" && ./engine/target/release/headwater check 2>/dev/null); status=$?
judge 'and the instance reports the reason rather than a pass' 0 "$status" \
    'change-scoped-only' "$out"

reset
seed_draft
move "$draft" draft current
out=$(gate); status=$?
judge 'the same document moved to a state the regime admits passes' 0 "$status" '' "$out"

reset
move "$terminal" discharged current
out=$(gate); status=$?
judge 'a movement out of a terminal state is refused' 1 "$status" \
    'lifecycle.transition.not_permitted' "$out"

reset
seed_draft
git -C "$scratch" mv "$draft" "docs/obligations/0126-renamed.md" >/dev/null 2>&1
move "docs/obligations/0126-renamed.md" draft discharged
out=$(gate); status=$?
judge 'a renamed document is read against the version at the path it left' 1 "$status" \
    'moved from `draft` to `discharged`' "$out"

printf '# the producer, and the arms with nothing in them\n'

reset
produce HEAD
out=$(cd "$scratch" && ./engine/target/release/headwater check --change "$hold/out/manifest" 2>/dev/null); status=$?
judge 'a tree that moved nothing is a change that names nothing' 0 "$status" \
    'scoped to a change: 0 documents named, 0 added, 0 with a prior version' "$out"

reset
printf '\n// one line nothing governs\n' >> "$scratch/engine/crates/check/src/change.rs"
produce HEAD
out=$(cd "$scratch" && ./engine/target/release/headwater check --change "$hold/out/manifest" 2>/dev/null); status=$?
judge 'a change that carries no document of the corpus names what it carried' 0 "$status" \
    'engine/crates/check/src/change.rs' "$out"

reset
printf -- '---\nid: X\n---\n' > "$scratch/docs/obligations/9999-untracked.md"
produce HEAD
out=$(grep -c '^added	docs/obligations/9999-untracked.md$' "$hold/out/manifest")
judge 'a file the index does not hold is named as one the change adds' 0 $? "1" "$out"

reset
git -C "$scratch" rm -q "$terminal"
produce HEAD
out=$(cd "$scratch" && ./engine/target/release/headwater check --change "$hold/out/manifest" 2>/dev/null); status=$?
judge 'a document the change deleted is named, and the report says no row holds it' 0 "$status" \
    "$terminal" "$out"

printf '# a document that left, and the three departures that are not one\n'

# The gate rather than the verb, because this is the moment the rule exists
# for. `$terminal` stands at `discharged`, which the `obligation` regime gives
# no exit, and that regime declares `retain_terminal: true`.
reset
git -C "$scratch" rm -q "$terminal"
out=$(gate); status=$?
judge 'deleting a document at a terminal state of a retaining regime is refused' 1 "$status" \
    'lifecycle.deletion.not_permitted (OB-LIFE-4): HW-OBL-0117 stood at `discharged`' "$out"

# The instrument, on the terms the transition case above states. The same tree,
# the same engine, and no manifest: the rule reports a skip and the gate exits
# 0. So the refusal above comes from the producer and from nothing else.
out=$(cd "$scratch" && ./engine/target/release/headwater check --strict 2>&1); status=$?
judge 'the same deletion with no change described is not refused' 0 "$status" '' "$out"

# The same file, moved rather than removed. Git reports a rename as one path
# with a prior version, the census holds a row where it arrived, and the entry
# binds. A rule that read a `prior` line as a departure refuses this.
reset
git -C "$scratch" mv "$terminal" "docs/obligations/0117-renamed.md" >/dev/null 2>&1
out=$(gate); status=$?
judge 'the same document renamed at the same state is not a deletion' 0 "$status" '' "$out"

# A document that never reached a terminal state. `$draft` opens at `draft`,
# which reaches `current` and `deprecated`, so nothing about it is retained.
#
# `HEADWATER_SKIP_FIGURE_CHECK` is set for the same reason the site cases below
# set `HEADWATER_SKIP_CRAWLER_CHECK`: removing a document moves `census.seen`
# and every figure derived from it, so the figures clause refuses this commit
# for a reason that has nothing to do with the lifecycle. That clause has its
# own cases, and one of them provokes exactly this in the other direction.
reset
git -C "$scratch" rm -q "$draft"
out=$(cd "$scratch" && HEADWATER_SKIP_FIGURE_CHECK=1 sh .githooks/pre-commit 2>&1); status=$?
judge 'deleting a document that stands at no terminal state is not refused' 0 "$status" '' "$out"

# A file that is no document of this corpus. Its prior version does not parse as
# one, so nothing is held against a regime.
reset
git -C "$scratch" rm -q "engine/crates/check/src/change.rs"
out=$(gate); status=$?
judge 'deleting a file that is no document of this corpus is not refused' 0 "$status" '' "$out"

reset
produce "0000000000000000000000000000000000000000"
status=$?
judge 'a base revision this clone does not hold is refused rather than skipped' 1 "$status" \
    'does not hold' "$(cat "$hold/why")"

reset
touch "$scratch/a$(printf '\t')b.md"
produce HEAD
status=$?
judge 'a path a manifest line cannot carry is refused before it is written' 1 "$status" \
    'holds a tab' "$(cat "$hold/why")"

printf '# the deployed site, which no engine reads\n'

# `site/` is what Cloudflare serves as https://headwater.tools/, with no build
# step, so the bytes a commit removes from it are the bytes that go off the air.
# The clause that refuses them sits above the engine check in the hook and reads
# the index rather than the working tree, so every case below stages its change.
#
# The file the refusal case below removes is `site/_headers`, and any file
# under `site/` would serve for it, because this clause sits above the engine
# check and refuses before a relation is read.
#
# The choice that has to be measured is the escape-hatch case further down,
# which asserts exit 0 and so needs a path that no relation of this corpus
# names. HW-DR-0037 declares `governs` over the eight pages it names and
# HW-DR-0047 declares it over `site/_headers`, so removing any of those nine
# raises `relation.target.unresolved` whatever this clause decides.
reset
git -C "$scratch" rm -q site/_headers
out=$(gate); status=$?
judge 'a commit that removes a file from the deployed site is refused' 1 "$status" \
    'deletes a file from `site/`' "$out"

# The instrument. Without it the case above only proves the gate refuses
# something about `site/`, rather than a deletion in particular.
reset
printf '\n<!-- a hand edit of the deployed site -->\n' >> "$scratch/site/index.html"
git -C "$scratch" add site/index.html
out=$(gate); status=$?
judge 'a hand edit of a page of the deployed site is not refused' 0 "$status" '' "$out"

reset
printf '<p>a new page</p>\n' > "$scratch/site/about.html"
git -C "$scratch" add site/about.html
out=$(gate); status=$?
judge 'a page added to the deployed site by hand is not refused' 0 "$status" '' "$out"

# An escape hatch nobody has watched work is an escape hatch nobody knows works.
#
# The case needs a page of the deployed site that no document of this corpus
# declares `governs` over, because a governed path is refused a second time by
# a rule the engine already runs — which is the case below. So the path is
# stated as a measurement rather than as a constant, the same way the width
# boundary further down is. This case named `site/_headers` until HW-DR-0047
# declared a `governs` edge onto it, and the suite then failed here saying
# nothing about why.
hatch="site/llms.txt"
reset
governed=$(grep -rl "^    - $hatch\$" "$scratch/docs" 2>/dev/null | tr '\n' ' ')
[ -n "$governed" ] || governed=ungoverned
judge 'the escape-hatch case still names a page no document governs' 0 0 \
    'ungoverned' "$governed (declares \`governs\` over $hatch)"

# `HEADWATER_SKIP_CRAWLER_CHECK` is set here and nowhere else in this block,
# and the reason is a collision worth stating rather than working around. The
# only files no document governs are `site/llms.txt`, `site/robots.txt` and
# `site/sitemap.xml`, because HW-DR-0037 governs every hand-built page by name.
# Those three are also exactly the three the crawler clause protects, since
# each is derived from the pages beside them. So the file this case needs and
# the files that clause guards are the same files, and there is no fourth one
# to pick. This case is about the site-delete clause, so it declares which
# clause it is testing and lets the other one alone. The crawler clause has
# its own cases below.
git -C "$scratch" rm -q "$hatch"
out=$(cd "$scratch" && HEADWATER_ALLOW_SITE_DELETE=1 HEADWATER_SKIP_CRAWLER_CHECK=1 \
    sh .githooks/pre-commit 2>&1); status=$?
judge 'the same removal with the named variable set is allowed through' 0 "$status" '' "$out"

# The crawler clause. `site/llms.txt`, `site/robots.txt` and `site/sitemap.xml`
# are derived from the title, the description and the path of every page under
# `site/`, and a retitled page makes them stale with nothing else to notice.
# That happened once, in `0219db3`, fourteen minutes after the first two files
# were introduced, and cost a second commit and issue #443 to repair.
reset
sed -i 's|<title>|<title>RETITLED |' "$scratch/site/proof/index.html"
out=$(gate); status=$?
judge 'a retitled page whose crawler files are stale is refused' 1 "$status" \
    'no longer matches the' "$out"
judge 'and the refusal names the line that moved' 1 "$status" \
    'RETITLED' "$out"
judge 'and it names the command that repairs it' 1 "$status" \
    'sh tools/refresh-crawler-files.sh' "$out"

# The sitemap joined the derived files in #554, and it is the one of the three
# that a retitle does not move: a title is not a URL. So the case that holds it
# adds a page instead of retitling one. The clause reads the working tree
# rather than the index, which is why nothing is staged here and why the case
# above stages nothing either. The sitemap #535 committed listed seven URLs
# where `site/` already held eight pages, and this is that drift refused.
reset
mkdir -p "$scratch/site/about"
cat > "$scratch/site/about/index.html" <<'HTML'
<title>About</title>
<meta name="description" content="A page added to hold the sitemap case.">
HTML
out=$(gate); status=$?
judge 'a page added under site/ whose sitemap is stale is refused' 1 "$status" \
    'site/sitemap.xml' "$out"
judge 'and the refusal names the URL the sitemap is missing' 1 "$status" \
    'https://headwater.tools/about/' "$out"

reset
sed -i 's|<title>|<title>RETITLED |' "$scratch/site/proof/index.html"
out=$(cd "$scratch" && HEADWATER_SKIP_CRAWLER_CHECK=1 sh .githooks/pre-commit 2>&1); status=$?
judge 'and the named variable releases that one clause' 0 "$status" '' "$out"

# What the variable releases, and what it does not. It lifts this clause and
# nothing else, so a page that a document of this corpus declares `governs`
# over is refused a second time, by a rule the engine already runs. Nobody
# designed that pairing and it is worth a case, because it is the reason the
# case above measures for an ungoverned page rather than naming any page.
reset
git -C "$scratch" rm -q site/index.html
out=$(cd "$scratch" && HEADWATER_ALLOW_SITE_DELETE=1 sh .githooks/pre-commit 2>&1); status=$?
judge 'the variable releases this clause and not the rule that reads a governed path' 1 "$status" \
    'relation.target.unresolved' "$out"

# The token clause. `tools/site-tokens.css` is the one copy of the visual
# register, and each hand-built page carries it between two markers. Before
# HW-DR-0050 every page carried its own copy of the six color tokens, and four
# of the eight had drifted to a seventh the other four did not declare. The
# case edits a value inside a page's block, which is the drift with no other
# witness: a title is unchanged, so the crawler clause above passes and this
# one is what refuses.
reset
sed -i 's|--accent: #1d5c54|--accent: #b30000|' "$scratch/site/proof/index.html"
out=$(gate); status=$?
judge 'a page whose visual register was edited by hand is refused' 1 "$status" \
    'disagrees with' "$out"
judge 'and the refusal names the page that moved' 1 "$status" \
    'site/proof/index.html' "$out"
judge 'and it names the command that repairs it' 1 "$status" \
    'sh tools/refresh-site-tokens.sh' "$out"

# A page with no marker pair is refused rather than skipped, which is the half
# of the clause a stale-block case cannot reach. A ninth page that opted out of
# the shared register would do it by carrying no marker, so a silent skip is
# what would let one through. `HEADWATER_SKIP_CRAWLER_CHECK` is set for the
# same reason the site-delete cases above set it: adding a page necessarily
# stales the crawler files, that clause runs first, and this case is about this
# clause.
reset
mkdir -p "$scratch/site/unmarked"
cat > "$scratch/site/unmarked/index.html" <<'HTML'
<title>Unmarked</title>
<meta name="description" content="A page added with no visual register markers.">
<style>
  body { background: rebeccapurple; }
</style>
HTML
out=$(cd "$scratch" && HEADWATER_SKIP_CRAWLER_CHECK=1 sh .githooks/pre-commit 2>&1); status=$?
judge 'a page added under site/ with no register markers is refused' 1 "$status" \
    'no marker pair in site/unmarked/index.html' "$out"

reset
sed -i 's|--accent: #1d5c54|--accent: #b30000|' "$scratch/site/proof/index.html"
out=$(cd "$scratch" && HEADWATER_SKIP_TOKEN_CHECK=1 sh .githooks/pre-commit 2>&1); status=$?
judge 'and the named variable releases that one clause' 0 "$status" '' "$out"

# The figures clause. Every number on a hand-built page comes from a run of
# this engine, written into a `data-figure` element by
# `tools/refresh-figures.sh`, and HW-DR-0039 rules it. The five cases below are
# the three directions a derived figure has to move in, plus the escape hatch
# and the denominator.
#
# The three directions are the whole argument that these are figures rather
# than numbers a script once wrote: a hand edit of one has to fail, an
# unrelated edit has to pass, and a change to the corpus the figure measures
# has to make it stale.

# Direction 1 — a figure edited by hand is refused, and the refusal names the
# figure and the page.
#
# The value is read off the page rather than written here. A constant would
# stop matching the next time this corpus grows, and the case would then edit
# nothing and pass while measuring nothing, which is the failure the
# escape-hatch case above is also written to avoid. `run.date` is deliberately
# not the figure chosen: it is the clock rather than a function of the tree,
# and the clause exempts it.
reset
figpage="site/proof/index.html"
seen=$(grep -o 'data-figure="census.seen"[^>]*>[0-9]*<' "$scratch/$figpage" \
    | head -1 | sed 's/.*>//; s/<$//')
[ -n "$seen" ] || seen=none
judge 'the figure case still finds a census.seen figure on the page it names' 0 0 \
    'a number' "$(case $seen in none) echo "no census.seen span in $figpage" ;; *) echo "a number ($seen)" ;; esac)"
sed -i "s|data-figure=\"census.seen\">$seen<|data-figure=\"census.seen\">$((seen - 1))<|g" \
    "$scratch/$figpage"
out=$(gate); status=$?
judge 'a figure edited by hand on a page of the deployed site is refused' 1 "$status" \
    'disagrees with a fresh run' "$out"
judge 'and the refusal names the figure and the page' 1 "$status" \
    "census.seen in $figpage" "$out"
judge 'and it names the command that repairs it' 1 "$status" \
    'sh tools/refresh-figures.sh' "$out"

# The escape hatch, which is a silent pass in the two clauses above and an
# announced one here. The hook cannot write into the commit, so the line it
# prints on the author's terminal is the whole local record of the bypass, and
# the CI step of the same name carries no hatch at all.
reset
sed -i "s|data-figure=\"census.seen\">$seen<|data-figure=\"census.seen\">$((seen - 1))<|g" \
    "$scratch/$figpage"
out=$(cd "$scratch" && HEADWATER_SKIP_FIGURE_CHECK=1 sh .githooks/pre-commit 2>&1); status=$?
judge 'and the named variable releases that one clause' 0 "$status" '' "$out"
judge 'and the release is announced rather than silent' 0 "$status" \
    'HEADWATER_SKIP_FIGURE_CHECK is set' "$out"

# Direction 2 — an edit that touches no figure is not refused. Without this the
# case above only proves the gate refuses something about `site/`.
#
# `site/compare/index.html` carries no `data-figure` element, no marker the
# crawler files read beyond its title, and the edit changes neither its title
# nor its description, so this clause is the only one with anything to say
# about it.
reset
printf '\n<p>A paragraph added by hand, carrying no figure.</p>\n' \
    >> "$scratch/site/compare/index.html"
out=$(gate); status=$?
judge 'an edit to a page that carries no figure is not refused' 0 "$status" '' "$out"

# Direction 3 — a change to the corpus a figure measures makes that figure
# stale. This is the direction that separates a derived figure from a number
# somebody typed once, and no other case here can reach it: directions 1 and 2
# both move the page, and this one moves what the page is about.
#
# An untyped Markdown file is the cheapest probe. It moves `census.seen` and
# `census.untyped` and touches no blessed fixture of the engine. The judge
# names `census.seen` rather than any count, because the count moves whenever
# this corpus does.
reset
printf 'A file added to move the census.\n' > "$scratch/docs/ZZ-census-probe.md"
out=$(gate); status=$?
judge 'a document added to the corpus makes a figure on the site stale' 1 "$status" \
    'census.seen in site/index.html' "$out"

# The denominator. `never` is the set of figures this run measured that reached
# no page, and it used to be printed and dropped: an empty `site/`, a renamed
# marker attribute or a moved page all left the figures half reporting
# `0 used across 8 pages` at exit 0, so the check ran over nothing and passed.
# HW-DR-0050 already refuses a page that opts out of the shared register rather
# than skipping it, and this is the same discipline for a figure.
reset
find "$scratch/site" -name '*.html' -exec sed -i 's/data-figure=/data-figurex=/g' {} +
out=$(gate); status=$?
judge 'a renamed marker that leaves every figure on no page is refused' 1 "$status" \
    'measured but on no page' "$out"
judge 'and the refusal states the denominator it ran over' 1 "$status" \
    '0 used across 8 pages' "$out"

# A finding whose location line lands on the width boundary, printed whole.
#
# #340 lays the report out at 80 columns, and the fill leaves a line alone when
# its opening word leaves no room for the word after it. A finding's location
# line is `<path>:<line>:<column> <severity>`, so where the path is long enough
# that the opening word *reaches* 80 without passing it, a fill that asked only
# whether the first word passed the width would put the severity on a line of
# its own. The hook then reads that bare `error` as the header of a new finding
# and drops the rule line and the `fix:` line under the real one, and a refused
# commit says a commit was refused and nothing about why.
#
# The case is stated as a measurement rather than as a path. `boundary` is a
# document of this corpus whose location line sits in that band today, and the
# first judge below fails loudly if it stops sitting there — a case that quietly
# left the band would go on passing while measuring nothing, which is exactly how
# this defect survived a full suite once.
boundary="docs/decisions/0041-q41-whether-vale-becomes-a-declared-regime-backend.md"

reset
# A British spelling is an error-severity finding whose remediation is mechanical.
printf '\nThe behaviour of this sentence is deliberately wrong.\n' >> "$scratch/$boundary"

# The opening word of the location line, at its indent of two. In the band when
# it reaches the width without passing it: a five-character severity then cannot
# join it, and 74 is the width less that severity and its space.
opening=$(cd "$scratch" && ./engine/target/release/headwater check --root . 2>/dev/null \
    | grep -F "$boundary" | grep -v '^  input ' | head -1 \
    | awk '{print 2 + length($1)}')
[ -n "$opening" ] || opening=0
band=no
[ "$opening" -gt 74 ] && [ "$opening" -le 80 ] && band=yes
judge 'the boundary case still sits on the width boundary it was chosen for' 0 0 \
    'yes' "$band (the opening word of $boundary is $opening columns, and the band is 75 to 80)"

out=$(gate); status=$?
judge 'a finding whose location line lands on the width boundary is refused' 1 "$status" \
    'language.controlled.not_met' "$out"
judge 'and the hook prints the location and the severity on one line' 1 "$status" \
    "$boundary:49:1 ✗ error" "$out"
judge 'and the whole message under it, not the first line of it' 1 "$status" \
    'and this sentence writes `behaviour`' "$out"
judge 'and the fix line, which is the last line of the finding' 1 "$status" \
    'fix (mechanical): write `behavior`' "$out"
# The severity never reaches a line of its own. `judge` matches on flattened
# whitespace, so this asks the raw output directly.
alone=no
printf '%s\n' "$out" | grep -qE '^[[:space:]]+error[[:space:]]*$' && alone=yes
judge 'and no line of the refusal is a bare severity word' 0 0 'no' "$alone"

reset
printf '\n%s passed, %s failed\n' "$passed" "$failed"
[ "$failed" -eq 0 ]
