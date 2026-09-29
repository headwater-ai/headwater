# The recorded terminal demonstration that README.md embeds.
#
# It runs the commands of docs/tutorials/your-first-governed-corpus.md, steps
# 1 to 7, against a scratch directory. HW-DR-0078 rules that this recording is
# frozen: nothing re-runs it or compares it with a later run, and the README
# holds it with a visible `Recorded on YYYY-MM-DD` marker beside the embed,
# which tools/repo/readme-fixtures.sh group 9 checks.
#
# To record it again, from the repository root, with vhs, ttyd and ffmpeg on
# PATH and a network that reaches the taxonomy release:
#
#   cargo build --profile dev-release -p headwater-cli --manifest-path engine/Cargo.toml --locked
#   PATH="$PWD/engine/target/dev-release:$PATH" vhs .github/assets/headwater-demo.tape
#
# The release binary that the tutorial pins works too: put it on PATH in place
# of the dev-release build. Then change the marker date in README.md to the
# day of the new recording.

Output .github/assets/headwater-demo.gif

Require headwater

Set Shell "bash"
Set FontSize 14
Set Width 1000
Set Height 560
Set Padding 16
Set Framerate 12
Set TypingSpeed 25ms
Set Theme "Builtin Dark"

Hide
Type `export PS1='$ ' && cd "$(mktemp -d)" && mkdir -p docs/decisions && git init -q`
Enter
Type "printf '# Store attempts in Postgres\n\nThe queue keeps every delivery attempt in Postgres.\n' > docs/decisions/postgres-note.md && clear"
Enter
Show

Type "headwater init"
Sleep 300ms
Enter
Sleep 3s

Type "headwater taxonomy vendor https://github.com/headwater-ai/headwater/releases/download/taxonomy/headwater-standard/v4.12.0/headwater-standard-4.12.0.zip --expect sha256:2470c4b4557761746467d4bb828b0d9610360347c247a02c6c121d72245cbee9"
Sleep 300ms
Enter
Sleep 4s

Type "clear"
Enter
Type "# Steps 5 and 6: pin the version, and give the identifiers a namespace"
Enter
Type "sed -i 's/  version: 0.0.0/  version: 4.12.0/' .headwater/taxonomy.yml"
Enter
Type "sed -i 's/^add: {}$/add:\n  identifier_schemes.decision_id.namespace: ACME/' .headwater/overlay.yml"
Enter
Sleep 1s

Type "headwater taxonomy resolve"
Sleep 300ms
Enter
Sleep 3s

Type "clear"
Enter
Type "headwater check | head -20"
Sleep 300ms
Enter
Sleep 7s
