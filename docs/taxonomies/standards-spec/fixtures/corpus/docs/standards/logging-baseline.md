---
id: SCR-STD-logging-baseline
status: current
status_since: 2026-03-01
last_verified: 2026-08-20
summary: What every Beacon component writes to its log, and the planted omission of the Conformance heading.
provenance:
  warrant: accepted
  agency: human
  accepted_by: fixture-acceptor
  evidence_basis: unevidenced
relations:
  regulates:
    - SCR-FS-delivery
---

# Logging baseline

## Scope

Every Beacon component that runs as a long-lived process. A command-line tool that runs and exits is out of scope, because its output is its log.

## Requirements

### 1. Every log line is one JSON object

A log a person greps is a log a machine cannot read. One object per line is the form both readers share.

### 2. Every log line carries the tenant and the delivery attempt

An operator reads a log to answer a question about one tenant, and a line with no tenant on it answers nobody.

### 3. No log line carries a payload body

A payload is a customer's data, and a log is copied to places a payload may not go.

This document is the planted defect of this corpus. The `standard` kind requires a `Conformance` heading and this document has none, so a run reports the omission and names the heading.
