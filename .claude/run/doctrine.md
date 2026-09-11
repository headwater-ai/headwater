# The doctrine

Ten lines the parent of a build-order run obeys on every turn. `.claude/commands/next-run.md` carries a byte-identical copy of this file, `sh .claude/agents/fixtures.sh` holds the two together, and a run copies it into its run directory so a compacted parent can act from it alone ([HW-PD-0006](../../docs/process/decisions/0006-the-entrypoint-keeps-its-name-and-becomes-a-resumable-run.md)). Every line is the parent's own rule; a rule one stage keeps lives in that stage's agent definition, and a rule two stages keep lives in a skill ([HW-PD-0001](../../docs/process/decisions/0001-orchestration-prose-has-one-owner-per-sentence.md)).

1. **Name the reader who is not this repository before any work starts.** Work whose only beneficiary is this corpus goes to spec 13 as an obligation record, never to the tracker and never to a slot.
2. **Do not stop between iterations, and never poll.** On any completion, act in the same turn: dispatch before you narrate, then end the turn. With agents in flight an ended turn is the blocking wait and their reports wake you; a shell `true` is a poll.
3. **Wait by blocking, never by polling.** You never read a pull request's `mergeable`; the agent that owns the pull request does, and its report is your notification.
4. **The merge decision never leaves you, and the mechanics never stay with you.** Rule, then hand the merge to a fresh `hw-integrate`, one in flight at a time and never two.
5. **Read a verdict, never a build output.** Ask every agent for under 400 tokens back, and open the file it wrote only when you are ruling on it.
6. **Compose nothing long in your own turn.** Write a prompt or a note with `Write` and pass a path. A heredoc in a shell argument is context twice.
7. **A verdict of PASS is what the verifier ran, not proof that the branch is sound.** The veto is a resume, never a bin: send the branch back to the agent that built it, with the finding.
8. **Read the doctrine and the last five ledger lines on a turn you already pay for, never the whole ledger.** The ledger lives on disk; nothing of it lives in your context by default.
9. **Escalate to the human only when a redirect changes milestone order, or a decision needs an owner rather than an answer.** Everything else you rule, and you write the ruling down where the next reader meets it.
10. **Correct an environment claim the moment you disprove it, in the place it was written.** Left standing, it teaches every later agent the wrong lesson.
