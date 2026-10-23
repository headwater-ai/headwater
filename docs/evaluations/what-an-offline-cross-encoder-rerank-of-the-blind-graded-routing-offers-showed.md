---
id: HW-EVAL-what-an-offline-cross-encoder-rerank-of-the-blind-graded-routing-offers-showed
status: current
status_since: 2026-10-23
summary: "A pinned cross-encoder, run offline under tract, reranked both paths' graded offers. Its top 3 reached 35 of 230 disjoint prompts against 19, at exact McNemar p 0.0004. Its silent first pick scored 2 on 9 of 71 against 4, so the bars of #1688 accept it."
last_verified: 2026-10-23
title: "What an offline cross-encoder rerank of the blind-graded routing offers showed"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0105
    - HW-EVAL-what-a-blind-grade-of-the-two-routing-paths-offers-showed
    - HW-DR-0064
    - HW-REQ-0001
    - tools/run/rerank-experiment.sh
---

# What an offline cross-encoder rerank of the blind-graded routing offers showed

[The blind grade of #1670](what-a-blind-grade-of-the-two-routing-paths-offers-showed.md) scored every document that the deterministic route or the embedding path offered on 361 prompts. It found that the embedding path's offer on a silent prompt is mostly noise. [#1688](https://github.com/headwater-ai/headwater/issues/1688) asked a narrower question. Suppose that a second model puts the same offered documents in a new order. Does the top of the list then hold more documents that a session needs? This document reports the answer. It used the scores that the raters already wrote, and no rater graded a new pair.

## What was run, and on which bytes

**The model.** The reranker is the cross-encoder `cross-encoder/ms-marco-MiniLM-L6-v2` at Hugging Face revision `233902d25c440f23af6f7d6e94d2946bac0bee0a`, under the Apache-2.0 license. #1688 names it `cross-encoder/ms-marco-MiniLM-L-6-v2`, which is its old name. Hugging Face renamed the repository, and the pin uses the current name. `.headwater/reranker.yml` pins two files by URL and sha256 digest: `onnx/model.onnx`, the fp32 export (`sha256:5d3e70fd…4d4a`), and `vocab.txt` (`sha256:07eced37…38a3`). The vocabulary is the same bytes as the vocabulary of the embedding model that `.headwater/embedding.yml` pins. The pin has the shape of the embedding pin, and its `max_tokens` is 512.

**It runs under tract 0.23.7.** `headwater-embed` gains a `Reranker` type. It loads the graph with `tract-onnx` at the exact version `=0.23.7` that the crate already pins, and it gives no error. The tokenizer writes `[CLS] task [SEP] document [SEP]`, with token type 0 for the task side and 1 for the document side. A pair longer than 512 pieces loses pieces from the end of the longer side first, which is the Hugging Face `longest_first` rule. The score is the raw logit of the one output. The model card prints the scores `8.607138` and `-4.320078` for its own two example pairs, which sentence-transformers computed in PyTorch. This crate scores the same pairs `8.607142` and `-4.320080`. The test `the_pinned_reranker_scores_a_relevant_passage_above_an_unrelated_one` holds both figures to within 0.001, and it skips when no `HEADWATER_RERANKER_DIR` names the fetched files.

**How the run satisfies [HW-REQ-0001](../requirements/0001-the-engine-reaches-no-network-at-check-time.md).** The engine reaches no network. `sh tools/embed/fetch-model.sh --pin .headwater/reranker.yml` fetches the two files once, into `<git common dir>/headwater-models/reranker`, and refuses a file whose digest is not the pinned one. That script is the only step that opens a connection. The `Reranker` reads only local bytes. It checks each file against its pinned digest before it reads it, as `Model::load` does for the embedding model. No byte of the model is committed. The runner is the example `engine/crates/embed/examples/rerank.rs`, and it is not a verb of `headwater`. [HW-DR-0077](../decisions/0077-the-consumer-surface-is-what-an-adopter-receives-runs-and-must-have-installed-and-it-is-a-closed-and-declared-list.md) closes the consumer surface. No verb is owed until the owner rules to put the reranker in `headwater route`.

**The text the model read.** The model read the blind packets that `tools/run/relevance-grade.sh packets` wrote for the raters, and no text rendered from the current tree. For each prompt, the query is the task text of the packet. For each document, the passage is its title, a newline, and its summary, as the raters saw them. The packets are not committed, because the owner ruled for #1670 that no prompt text is committed. The committed score file holds only a prompt id, a path and a score on each line.

## The procedure

`sh tools/run/rerank-experiment.sh run` did these steps on 2026-10-23, from 05:53 to 05:56 UTC, on one Intel Core Ultra 5 250K Plus host with 18 threads. The run took 183 seconds.

1. For each of the 361 prompts of `tools/run/relevance-grade/sample.tsv`, it takes the union of the route's list and the embedding path's list.
2. It drops each obligation, each review and each sealed probe, as #1688 requires. 2,760 (prompt, document) pairs remain.
3. It scores each pair with the pinned cross-encoder and writes `tools/run/relevance-grade/rerank-scores.tsv`. This step reads the sample and the packets, and it never reads `consensus.tsv`. So no grade can reach the order.
4. It sorts each prompt's documents by score, from high to low. On a tie, the path in ascending order comes first. Then it cuts the list to 5.
5. It grades the cut from `consensus.tsv` and writes `tools/run/relevance-grade/rerank.txt`.

Step 5 reads only the sample, the consensus and the score file. So a clone with no packets and no model prints the same `rerank.txt` with `sh tools/run/rerank-experiment.sh grade`. The digests of the four files are `fa01fc85…5690` (sample), `b08388eb…0262` (consensus), `7e3a8370…fbbf` (scores) and `ee5a750b…c4f0` (report). The first two are the same files that the #1670 grade read.

**The test.** A prompt counts when its top 3 holds a document that the consensus scores 2. The baseline is the first 3 of the embedding path's list, unfiltered, as the #1670 grade read it. The run compares the two paths prompt by prompt, with an exact McNemar test. That is a two-sided binomial test on the discordant prompts, with probability one half. Here b counts the prompts that the embedding top 3 reaches and the rerank misses, and c counts the reverse. The discordant counts are small, so the run uses no chi-square approximation.

## The figures

Every pair of the 2,760 has a consensus score. So no pair in the cut is without a score. That is 0 of 283 silent pairs, 0 of 1,124 disjoint pairs and 0 of 300 overlap pairs. No pair was graded again.

| stratum | prompts | embedding top 3 at 2 | rerank top 3 at 2 | b | c | exact p | embedding first pick at 2 | rerank first pick at 2 | oracle, filtered | oracle, unfiltered |
|---|---|---|---|---|---|---|---|---|---|---|
| silent | 71 | 7 | 15 | 0 | 8 | 0.0078 | 4 | 9 | 17 | 17 |
| disjoint | 230 | 19 | 35 | 2 | 18 | 0.0004 | 12 | 23 | 43 | 48 |
| overlap | 60 | 13 | 17 | 2 | 6 | 0.2891 | 7 | 12 | 23 | 24 |

The oracle is the count of prompts that hold a document at score 2 anywhere in the union. No reorder of the union can reach more. The filtered oracle is the ceiling of this rerank. The filter takes 5 disjoint prompts and 1 overlap prompt out of reach. In each of them, the only document at score 2 is an obligation, a review or a sealed probe. On disjoint prompts the rerank reached 35 of the 43 that it could reach, and the embedding path reached 19.

On silent prompts the rerank orders only the embedding path's own documents, because the route offered nothing. So the silent gain comes from the order alone. The top 3 at score 2 moves from 7 to 15 of 71. The first pick at score 2 moves from 4 to 9 of 71. No silent prompt lost a document at score 2 from its top 3.

## The verdict

**By the bars of #1688, the verdict is accept.** The issue set three bars before the run:

- **Accept** if the paired difference on disjoint prompts is significant at 0.05, and the silent first pick at score 2 at least doubles. That is from 4 to 8 or more. On disjoint prompts c is 18 and b is 2, and the exact p is 0.000402. The silent first pick at score 2 is 9 of 71. Both conditions hold.
- **Reject** if the rerank reaches fewer than 30 of the oracle's 48 disjoint prompts. It reaches 35, so this bar does not hold.
- Anything else is inconclusive.

The script applies the bars in this order, and it prints `inconclusive` when the accept bar and the reject bar both hold. That did not occur here.

**What the verdict changes for [HW-DR-0105](../decisions/0105-whether-the-embedding-path-becomes-the-silence-only-fallback-of-routing-and-what-happens-to-the-shadow-collection.md).** HW-DR-0105 stays a draft, and only the owner rules on it. The #1670 grade showed that the embedding path's silent offer is mostly noise in its given order. This run shows that a second, offline model makes the top of the same offer more useful. A silent first pick at score 2 on 9 of 71 prompts is still not a useful offer on most silent prompts. On 2026-10-22 the owner ruled on the case where this cross-encoder is the best offline variant. Then the fp32 export may later go into `headwater route` as a hook variant. That work belongs to [#1777](https://github.com/headwater-ai/headwater/issues/1777) and [#1839](https://github.com/headwater-ai/headwater/issues/1839), and this document adds no flag to any verb.

**The rater ruling.** On 2026-10-04 the owner ruled that a second rater grades the same 19 batches before any further grading spend. This run spent nothing on raters. Every score it read was already in `consensus.tsv`. So that ruling does not gate this result.

## What the result cannot say

- No person checked a score, and both raters of #1670 are one model. A systematic error of that model is in every figure here.
- The three strata are three tests. The disjoint test is the one the accept bar names. The silent and overlap p values are reported, and no correction for three tests is applied.
- The packets show a document by its title and summary alone. The reranker and the raters read the same text, so the rerank is evaluated on what a pointer shows, and not on the whole document.
- A cross-encoder score is a floating-point fact about this machine. Another instruction set can move a logit in its last digits, and a tie between two documents can then fall the other way. The committed score file is the record. `grade` reprints `rerank.txt` from it, and it does not compute the scores again.
- The figures are over the person prompts of one owner's sessions in one repository, under one embedding-model digest.

## How to run it again

    sh tools/embed/fetch-model.sh --pin .headwater/reranker.yml
    sh tools/run/rerank-experiment.sh run
    sh tools/run/rerank-experiment-fixtures.sh

The first command is the one fetch, and it needs the network. The second needs the packets of #1670 under `<git common dir>/headwater-relevance-grade`, or a directory that `HEADWATER_RELEVANCE_PACKETS` names. The third runs the cases that hold the script. They inject scores in place of the model and work each count, each McNemar p and each verdict by hand. They also permute the consensus and require the same score file and the same order.
