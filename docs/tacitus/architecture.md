# Architecture and implemented boundaries

The EdgeQuake Rust workspace and Next frontend remain in place. New crates are members
of the existing workspace; no second database or private Praxis code is included.

- `tacitus-core`: immutable source versions, UTF-8 extraction of selected text, spans,
  assertions, temporal/decision contracts, review fingerprints, local identity policy,
  source/storage/Praxis interfaces.
- `tacitus-capsules`: deterministic version-1 data ZIPs, bounded validation and explicit
  fresh-workspace proposal rebinding; scoped evidence compiler; developer file CLI.
- `tacitus-analysis`: proposed decision-brief execution through a model interface,
  hypothesis matrix validation, commitment evidence records, dependency impact closure.
- `tacitus-evals`: comparison records and equal-access checks; no measured model runner yet.

The context compiler filters selected versions by workspace, access and two temporal
intervals before calling its retrieval interface, then checks every returned candidate.
It combines lexical ranking with optional adapter semantic scores, keeps complete short
source versions, groups common origins and reports omissions. Lexical matching is not
decision salience. Reviewed-method prerequisites, graph traversal, counterevidence
prioritization and exact model tokenization remain required integration work.

The capsule developer CLI uses lexical retrieval only. It is not the Tacitus UI or a
replacement for EdgeQuake ingestion/vector/graph/model processing.

Canonical PostgreSQL tables, transactions/CAS review writes, source-change events,
projection rebuild/deletion and cache invalidation are not implemented. The traits state
the required boundary; they do not claim storage or API security. Review history must be
append-only where appropriate, but an audit table must not be called tamper-proof.

Next integration: authenticate a namespaced `/api/v1/tacitus` route group using existing
middleware; enforce the local-only policy at every provider construction path, including
direct upstream endpoints/connections; add namespaced migrations and authoritative API
types; reuse source/graph components in the existing frontend. None is enabled yet.
