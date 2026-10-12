# Current developer commands

There is no accepted application startup command or localhost URL yet. Docker/Ollama
are absent on the tested machine. `up`, `setup`, `demo`, `smoke`, `benchmark`, `down`
return NOT IMPLEMENTED instead of pretending to launch a functioning product.

On Windows PowerShell:

```powershell
.\scripts\tacitus.ps1 doctor
.\scripts\tacitus.ps1 test
cargo test --locked --manifest-path edgequake/Cargo.toml -p tacitus-core -p tacitus-capsules -p tacitus-analysis -p tacitus-evals
python scripts/generate-tacitus-schema.py --check
python scripts/tacitus-file-example.py
```

`doctor` intentionally returns exit 2 when Docker is missing. `test` uses an isolated
developer workspace and cached crates only; no models are downloaded. The regular Cargo
command uses the retained upstream workspace and lockfile. Rust 1.95 is required for
these source-development tests. The file example additionally needs Python and jsonschema;
it is not the application runtime and clearly reports lexical-only compilation.

The checked-in shell launcher provides corresponding `doctor` and `test` commands,
but has not been executed on Linux/WSL. No Linux/macOS support claim is made. For future
WSL2 development, install a Linux distribution and Docker Desktop WSL integration,
clone in that distribution's filesystem and use its Linux toolchains, not PowerShell
syntax inside bash. Model/container/hardware acceptance is still required.

File CLI operations (development only):

```text
cargo run --locked --manifest-path edgequake/Cargo.toml -p tacitus-capsules -- validate FILE.capsule
cargo run --locked --manifest-path edgequake/Cargo.toml -p tacitus-capsules -- inspect FILE.capsule
cargo run --locked --manifest-path edgequake/Cargo.toml -p tacitus-capsules -- preview MANIFEST.json DATA.json EVIDENCE_ROOT
cargo run --locked --manifest-path edgequake/Cargo.toml -p tacitus-capsules -- pack MANIFEST.json DATA.json EVIDENCE_ROOT OUTPUT.capsule
cargo run --locked --manifest-path edgequake/Cargo.toml -p tacitus-capsules -- compile FILE.capsule WORKSPACE CONTRACT.json OUTPUT.json
```

Preview shows every included field/file. Outputs are local and refuse overwrites; no
uploads, URL fetches or executable methods occur. Imported bundles remain untrusted.
Examples and resulting artifacts are intentional synthetic data under ignored `.tacitus`.
References-only bundles cannot fabricate missing evidence.
