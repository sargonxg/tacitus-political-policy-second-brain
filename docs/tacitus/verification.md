# Verification evidence — Windows development checkpoint

Tested OS: Windows 11 Pro x64, PowerShell, Rust 1.95.0, Node 24.12.0.
CPU/host-model/container application profiles: **NOT TESTED**.

## Passed

- `cargo test --offline --locked --manifest-path edgequake/Cargo.toml -p tacitus-core -p tacitus-capsules -p tacitus-analysis -p tacitus-evals`:
  current suite has 25 deterministic tests. Includes selected UTF-8 ingestion/hashes,
  malformed/empty text, citation offsets, stale/imported reviews, selection/workspace/time
  rejection before/after retrieval, budget errors, exact local identities/embedding-change
  rejection, malicious archives, substantive capsule round-trip, analysis output errors,
  absent model propagation, impact closure, hypothesis/commitment rules and pending ratings.
- `scripts/tacitus.ps1 test`: same new crates in an isolated locked cached workspace.
- `C:\Program Files\Git\bin\bash.exe ./scripts/tacitus test`: all 25 tests passed
  using Windows Git Bash; shell doctor exited 2 as expected. Linux/WSL remains untested.
- Scoped `cargo clippy ... --all-targets -- -D warnings` and scoped Rust formatting.
- `python scripts/generate-tacitus-schema.py --check`: schema artifact consistency.
- `python scripts/tacitus-file-example.py`: actual local CLI preview/pack/validate/import/
  fresh-workspace lexical compile. Three substantive public fictional sources, qualification
  text, source hashes/citations and assertion conditions preserved. JSON Schema validates
  before/after; unsupported schema rejected. This is **not** model/API/browser acceptance.
- Gitleaks 8.30.1 directory scans: no findings in new crates, scripts, fixtures,
  schemas or Tacitus docs. Raw scan reports remain in ignored `.tacitus`, not public data.
- Isolated Tacitus dependency audit: no vulnerability finding. See inherited audit below.
- A fresh **local** clone at `a4a7abf6c444928ea439c1d01e590169fdf11625` ran the PowerShell
  cached library suite (25 passed) and schema check with a clean Git working tree.
  This is developer-library rehearsal, not a remote clone or clean-volume application test.
- Five new local commits through that revision scanned clean with Gitleaks; only the
  inherited selected-pin history remains awaiting triage. Local main/development match;
  that rehearsal preceded publication. The Tacitus project entry point is now `main`;
  `edgequake-main` remains an inherited upstream reference.

The first normal workspace test attempted with `--offline --locked` failed because
`async-stream` index metadata was absent. A subsequent normal Cargo test permitted initial
dependency metadata/software downloads, added only new crate/ZIP lock entries, and passed.
Further scoped tests run offline. First isolated compilation took 19.12s; first normal
workspace scoped compilation took 28.99s. These are build timings, not inference benchmarks.

## Failed / unresolved

- `scripts/tacitus.ps1 doctor`: exit 2, Docker missing. `docker version` / `docker compose
  version` unavailable; standard Docker Desktop executable absent; no Ollama command;
  `wsl --list --quiet` showed no installed distribution.
- Whole upstream Cargo audit: exit 1, `rsa 0.9.10`, RUSTSEC-2023-0071 (Marvin timing
  side-channel advisory), plus 6 unmaintained, 2 unsound and 1 yanked warning categories.
  Not remediated by unrelated dependency upgrades. Tacitus's isolated dependency set is
  audited separately; this does not clear inherited authentication risk.
- Full-ref Gitleaks scan: 2,217 commits, 71 findings, 264 MB inspected, about 160s.
  Three OpenAI-shaped findings are on unrelated upstream refs, outside selected-pin
  ancestry and absent from current files. They were not printed, used or tested.
- Selected-pin history scan: 646 commits, 35 findings, 174 MB inspected, about 63s.
  Rules: 25 generic API key, 8 curl authorization, 2 private key. Most are associated
  with public examples, development configs and tests; final fingerprint-level triage
  remains pending. No blanket allowlist was used. These inherited findings remain disclosed; the user explicitly requested publication
  of the public-code checkpoint. Only main/development are pushed; unrelated upstream
  refs containing OpenAI-shaped findings are not pushed.

## Not run / not implemented

Full upstream backend suite; frontend dependency install/typecheck/lint/build;
real Ollama model availability/license/quantization/digest/context/dimensions/memory;
PostgreSQL extensions/migrations; live ingestion/vector/graph/source-cited generation;
HTTP authorization/local-only enforcement; persistent reviews; HTML/PDF extraction;
import cancellation/restart; offline egress isolation; model/embedding outage acceptance;
changed-vector index rebuild; source revocation caches; four end-to-end UI modules;
browser/accessibility/screenshots; local model benchmarks; human evaluations;
clean application volumes/restart/offline rehearsal; CI.

Unit-only model stubs are confined to tests and are not a selectable runtime provider.
Prompt instruction text does not prove resistance to model prompt injection.
No localhost application URL or successfully tested application startup exists yet.
