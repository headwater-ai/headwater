---
id: HW-IFACE-headwater-neighbors
status: current
status_since: 2026-09-17
summary: "How the pinned local model ranks document summaries by meaning for the shadow log. A model file that fails its digest stops the run."
last_verified: 2026-09-17
title: "headwater neighbors"
relations:
  governs:
    - [engine/crates/cli/src/lib.rs, engine/crates/cli/src/main.rs]
    - engine/crates/embed/src/lib.rs
    - engine/crates/embed/src/wordpiece.rs
---

# headwater neighbors

## Synopsis

    headwater neighbors <task description> [--model <dir>] [--top <n>] [--json] [--root <path>]

Every word after `neighbors` forms the task description.

## Description

`headwater neighbors` ranks the typed documents of the corpus by the similarity in meaning between each summary and the task description. It is the embedding path of [HW-DR-0064](../decisions/0064-q64-whether-intent-time-routing-gains-an-offline-embedding-path-in-shadow-mode.md). Its only caller is `.claude/hooks/intent.sh`, which writes the result into the shadow-mode routing log. No agent reads the ranking, and `headwater route` does not read it.

The verb reads the model pin at `.headwater/embedding.yml`. The pin names each model file by URL and by digest. The verb does not fetch a file. `tools/embed/fetch-model.sh` fetches the files. The verb calculates the digest of each file before it loads the model, and refuses a file that does not match. A verified file that has not changed since is not hashed again, as the Files table states.

Inference runs locally in pure Rust. The verb opens no network connection. It calculates one unit-length vector for the task and one for each summary. It ranks the documents by the dot product of the two vectors. A tie goes to the lower path in byte order. A document with no summary is not ranked.

The summary vectors are a cache under `.headwater/cache/embeddings/`. The model digest names the cache file, and the digest of the summary text is the key of each entry. The same model can give different vectors on different instruction sets, so a vector is a fact about one machine. No `headwater generate` emitter writes a vector.

Text prints the model, the three digests and the ranked paths with their scores. JSON carries `task`, `tree_digest`, `lock_digest`, `model`, `model_digest`, `considered` and `neighbors`. Each member of `neighbors` carries `path`, `score` to four decimal places and `summary_digest`. The tree digest is the same value that a probe plan records.

## Preconditions

The repository must carry a readable `.headwater/taxonomy.lock`, consumer declaration and corpus.

`.headwater/embedding.yml` must pin `model.onnx` and `vocab.txt`, each with a `sha256:` digest. The model directory must hold both files, and the bytes of each file must match the pin.

## Options

| Option | What it does |
|---|---|
| `--model <dir>` | Select the directory that holds the model files. The default is `.headwater/models` under the root. |
| `--top <n>` | Set the number of ranked documents to print. The default is 10. |
| `--json` | Write the ranking as a machine-readable JSON document. |
| `--root <path>` | Select the repository to read. |

## Exit status

**0** means that the verb ranked the corpus.

**1** means that the verb refused the run. The six reasons are these: the task was missing, an option was invalid, or the repository could not load. Or the pin could not be read, a model file was missing, or a model file did not match its digest. Standard output is empty, and one sentence on standard error gives the reason. [HW-DR-0043](../decisions/0043-q43-whether-a-refusal-under-json-is-a-json-document.md) rules that a refusal under `--json` is not a JSON document.

## Environment

No environment variable reaches this verb. The hook reads `HEADWATER_MODEL_DIR` and passes its value as `--model`.

## Files

| Path | How this verb treats it |
|---|---|
| `.headwater/embedding.yml` | Read for the model name, the token limit and the digest of each model file. |
| `<model dir>/model.onnx`, `<model dir>/vocab.txt` | Read, and refused where a digest differs. |
| `<model dir>/.<file>.verified` | Written after a file matches its pin. It holds the digest, the length and the modification time. While all three still agree with the file, the verb does not calculate the digest again. |
| `.headwater/cache/embeddings/<model digest>` | Read, and written through a rename. It keeps only the entries of the current run. |
| `.headwater/taxonomy.lock`, `.headwater/taxonomy.yml`, the corpus | Read for the typed documents and their summaries. |

Standard error carries one line that counts the summary vectors that the run calculated.

## See also

[`headwater route`](headwater-route.md) is the deterministic read that an agent receives.

[HW-DR-0064](../decisions/0064-q64-whether-intent-time-routing-gains-an-offline-embedding-path-in-shadow-mode.md) rules on the shadow mode, the pin, the cache and the build order.

[`headwater taxonomy vendor`](headwater-taxonomy.md) uses the same division: the caller fetches, and the verb holds the digest.
