# chatty-factory-rebuild

Minimal governed factory-loop library.

## Terminology Boundary

`Factory loop`, `factory`, and `rebuild` name a governed artifact-production
pipeline. They do not mean an autonomous builder, self-directed agent, or
general-purpose product generator.

`Learning` in this crate means journal-backed failure evidence that can produce
scoped constraint candidates and, after explicit promotion, change future
admissibility. It does not mean hidden model self-learning, model memory, or
unreviewed doctrine.

`Operator` and `authority` mean externally confirmed control over intent and
capability spend. The crate records an external operator assertion but does not
provide cryptographic human identity proof.

ChattyFactory is a local agentic build system. Its trust boundary assumes a
legitimate local operator/host and focuses on preventing model-side authority
fabrication, escalation, replay, or bypass.

## Storage and Portability

This crate does not choose a machine-specific data directory. Callers provide
explicit roots:

- `RuntimeJournal::new(root, trace_id, request_id)` writes
  `runtime_records.jsonl` and `journal_head.json` under `root`, creating the
  directory on first use.
- `HostBounds { workspace_root, ... }` scopes generated files to the supplied
  workspace root.

That makes the crate suitable for CLI, desktop, service, or test harness use:
the wrapper application decides whether storage should be portable-local,
per-user app data, or an explicit command-line path.

Release packages should not include Cargo build output such as `target/`.

License: AGPLv3 - see license file for information.
