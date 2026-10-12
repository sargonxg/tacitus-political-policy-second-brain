#!/usr/bin/env bash
# Explicit future-host verification. Do not run on the storage-constrained laptop.
set -euo pipefail
cd "$(dirname "$0")/.."
if [[ "${TACITUS_CLOUD_EXECUTE:-}" != "1" ]]; then
  echo 'Set TACITUS_CLOUD_EXECUTE=1 on the chosen host to permit dependency downloads/builds/tests.' >&2
  exit 2
fi
node --test tools/tacitus-policy-lab/test.mjs scripts/tacitus-stack.test.mjs
python3 scripts/generate-tacitus-schema.py --check
python3 scripts/generate-tacitus-schema.py --v2 --check
cargo fmt --manifest-path edgequake/Cargo.toml --all --check
cargo test --locked --manifest-path edgequake/Cargo.toml -p tacitus-core -p tacitus-capsules -p tacitus-analysis -p tacitus-evals
cargo check --locked --manifest-path edgequake/Cargo.toml -p edgequake-api --features postgres
cargo test --locked --manifest-path edgequake/Cargo.toml -p edgequake-api --features postgres --test tacitus_provider_policy
cargo clippy --locked --manifest-path edgequake/Cargo.toml -p tacitus-core -p tacitus-capsules -p tacitus-analysis -- -D warnings
node scripts/tacitus-lab-roundtrip.mjs
cd edgequake_webui
pnpm install --frozen-lockfile
pnpm typecheck
pnpm lint
pnpm test
pnpm build
pnpm audit --audit-level=high
# Inherited RustSec/audit findings remain a release gate; never ignore or waive them.
echo 'Source checks finished. Real runtime/cloud/browser acceptance is still a separate step.'
