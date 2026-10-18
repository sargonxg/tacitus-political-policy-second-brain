# Core release-gate repair — 18 October 2026

The cumulative source candidate preserves main `6fc3da15855324186620f0abb44c6c845a87d85f`
and the complete PR14–18 ancestry through PR18
`0d47eafd93f2b4a5e4a926d3c761410b1c319cd6`. A separate commit adds the
existing reviewed original-source-span recall patch and four regressions.
The original PR branches and published tags are unchanged.

Migration `170_tacitus_policy.sql` already existed on main, but its compatibility
maximum, checksum registration, schema documentation and epoch coverage were
inconsistent. The repair advances current metadata to 170 and appends its exact
SHA-384 to the lock. Every historical SQL byte and prior checksum entry remains
unchanged. Published `v0.34.0` remains schema 169; current source is explicitly
unreleased. A schema-169 database with 170 pending still refuses fail-mode
serving. No database migration was executed.

The existing epoch `tag` field now additionally records immutable source commit
`7fbd854456fd3a0c19ff138308057941cdd56770` in replay mode. Both coverage and
matrix replay already resolve Git refs. This source epoch has the same 168-file,
maximum-170 SQL set as HEAD, hash `d8a6390a223d914e`. All 112 published tags
remain covered. This is metadata coverage; PostgreSQL epoch replay was not run.

The four internal TACITUS Cargo package versions now match the application's
existing `0.34.0` parity requirement. Only their local package entries change in
the application and isolated-capsule locks; registry dependencies and checksums
remain pinned. Native schema versions, native integer package versions and wire
format identifiers are unchanged. All three real native-v2 archive hashes match
the earlier source-span-patched baseline exactly.

The unused `Store::event` helper is removed. Existing event writes continue
through their transactional `insert_in` paths; no audit operation was disabled.
The budget-catalog test adds `--locked` with the same package, filter and
assertions, preventing incidental lockfile rewriting. No lint exemption or
release-gate skip was added.

The root README again exposes the application package version consumed by the
existing parity check, with an explicit unreleased-source and native-format
distinction. The parity assertion itself is unchanged.

Local verification uses Rust/Cargo 1.95.0 and Bun 1.4.3. The fresh isolated
capsule gate passes 135 Rust tests, 194 Node tests, four preparation regressions,
Clippy with warnings denied, formatting, generated schemas, examples and
real/synthetic workflows. The repaired application library stage passes 3,813
tests, with zero failures and one existing ignored test. The actual
`make release-gates DATABASE_URL= POSTGRES_PASSWORD=` command exits zero,
including workspace/per-crate Clippy with warnings denied, resource and
observability proofs, WebUI typecheck and selected unit tests, and all metadata
checks. Its optional PostgreSQL bootstrap stage skips because no database is
configured; its WebUI performance stub skips because `.next` is absent. Neither
is represented as executed acceptance. Required remote CI is separate and is
not claimed by these local results.

Reproduction entry points:

```sh
cargo fetch --locked --manifest-path edgequake/Cargo.toml
make release-gates DATABASE_URL= POSTGRES_PASSWORD=
python3 scripts/check-capsule-release.py /path/to/new-output --offline
```

The executor sets `CARGO_HOME=/workspace/.cargo` and
`RUSTUP_HOME=/workspace/.rustup`, with the installed tool directories on PATH.
The full application run uses four build jobs, debug symbols disabled and
incremental compilation disabled to fit local disk. Debug assertions and test
scope remain enabled. Registry fetching uses the committed lock; subsequent
Cargo commands use its populated offline cache.

The default dependency build scripts could not resolve their GitHub asset
download host. Local application checks use the documented `PDFIUM_BUNDLE_LIB`
and `PDFIUM_LIB_PATH` overrides pointing to the already installed Pdfium
153.0.7999.0 library, SHA-256
`224f8ece41f7e35891f11c10073b7b7062d7a18e9ef870586162a85c46130f7d`,
with a writable `PDFIUM_AUTO_CACHE_DIR`. This does not establish the default
bundled Pdfium 7961 path. `SWAGGER_UI_DOWNLOAD_URL` selects a local ZIP containing
unchanged official npm `swagger-ui-dist@5.17.14` assets and their license, ZIP
SHA-256 `a8b81ce264f2781e65fab0339c35e931248998904d548de5516f7bf0e7b680f3`.
That package mirrors the upstream distribution; its ZIP container is locally
prepared rather than the original GitHub source archive.

No live database, hosted authentication/provider, deployment or answer-quality
acceptance is established here. Publication and merge require separate review
of the exact cumulative head and actual required CI evidence.
