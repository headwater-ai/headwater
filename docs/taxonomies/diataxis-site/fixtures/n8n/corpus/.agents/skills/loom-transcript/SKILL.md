---
# Added by Headwater. Everything after this block is the pinned upstream file,
# byte for byte.
# Upstream: n8n-io/n8n, master, b0550cb3cb4d1752546a69056c55eccfb9111a12,
# .agents/skills/loom-transcript/SKILL.md
# n8n's own Agent Skills front matter is below, unchanged. Headwater's keys are
# merged into the same block because a second block would not be read.
name: n8n:loom-transcript
description: Fetch and display the full transcript from a Loom video URL. Use when the user wants to get or read a Loom transcript.
argument-hint: [loom-url]
id: N8N-HOW-loom-transcript
status: current
status_since: 2026-09-01
last_verified: 2026-09-01
summary: "Fetch and display the full transcript from a Loom video URL. Use when the user wants to get or read a Loom transcript."
provenance:
  warrant: asserted
  agency: mixed
  evidence_basis: evidenced
---

# Loom Transcript Fetcher

Fetch the transcript from a Loom video using Loom's GraphQL API.

## Instructions

Given the Loom URL: $ARGUMENTS

### 1. Extract the Video ID

Parse the Loom URL to extract the 32-character hex video ID. Supported URL formats:
- `https://www.loom.com/share/<video-id>`
- `https://www.loom.com/embed/<video-id>`
- `https://www.loom.com/share/<video-id>?sid=<session-id>`

The video ID is the 32-character hex string after `/share/` or `/embed/`.

### 2. Fetch Video Metadata

Use the `WebFetch` tool to POST to `https://www.loom.com/graphql` to get the video title and details.

Use this curl command via Bash:

```bash
curl -s 'https://www.loom.com/graphql' \
  -H 'Content-Type: application/json' \
  -H 'Accept: application/json' \
  -H 'x-loom-request-source: loom_web_45a5bd4' \
  -H 'apollographql-client-name: web' \
  -H 'apollographql-client-version: 45a5bd4' \
  -d '{
    "operationName": "GetVideoSSR",
    "variables": {"id": "<VIDEO_ID>", "password": null},
    "query": "query GetVideoSSR($id: ID!, $password: String) { getVideo(id: $id, password: $password) { ... on RegularUserVideo { id name description createdAt owner { display_name } } } }"
  }'
```

### 3. Fetch the Transcript URLs

Use curl via Bash to call the GraphQL API:

```bash
curl -s 'https://www.loom.com/graphql' \
  -H 'Content-Type: application/json' \
  -H 'Accept: application/json' \
  -H 'x-loom-request-source: loom_web_45a5bd4' \
  -H 'apollographql-client-name: web' \
  -H 'apollographql-client-version: 45a5bd4' \
  -d '{
    "operationName": "FetchVideoTranscript",
    "variables": {"videoId": "<VIDEO_ID>", "password": null},
    "query": "query FetchVideoTranscript($videoId: ID!, $password: String) { fetchVideoTranscript(videoId: $videoId, password: $password) { ... on VideoTranscriptDetails { id video_id source_url captions_source_url } ... on GenericError { message } } }"
  }'
```

Replace `<VIDEO_ID>` with the actual video ID extracted in step 1.

The response contains:
- `source_url` — JSON transcript URL
- `captions_source_url` — VTT (WebVTT) captions URL

### 4. Download and Parse the Transcript

Fetch **both** URLs returned from step 3 (if available):

1. **VTT captions** (`captions_source_url`): Download with `curl -sL "<url>"`. This is a WebVTT file with timestamps and text.
2. **JSON transcript** (`source_url`): Download with `curl -sL "<url>"`. This is a JSON file with transcript segments.

Prefer the VTT captions as the primary source since they include proper timestamps. Fall back to the JSON transcript if VTT is unavailable.

### 5. Present the Transcript

Format and present the full transcript to the user:

**Video:** [Title from metadata]
**Author:** [Owner name]
**Date:** [Created date]

---

**0:00** - First transcript segment text...

**0:14** - Second transcript segment text...

(continue for all segments)

---

## Error Handling

- If the GraphQL response contains a `GenericError`, report the error message to the user.
- If both `source_url` and `captions_source_url` are null/missing, tell the user that no transcript is available for this video.
- If the video URL is invalid or the ID cannot be extracted, ask the user for a valid Loom URL.

## Notes

- No authentication or cookies are required — Loom's transcript API is publicly accessible.
- Only English transcripts are available through this API.
- Transcripts are auto-generated and may contain minor errors.
