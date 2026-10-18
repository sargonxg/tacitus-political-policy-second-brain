# Duplicate-aware JSON boundary

This isolated delta stacks on experimental PR10. It adds no dependencies and changes no native schema or exported bytes.

- `tacitus_capsules::strict_json::validate` walks original UTF-8 bytes with a serde streaming visitor. Each object has a fresh set of decoded keys; escaped-equivalent spellings are duplicates, including Unicode surrogate-pair spellings. Unicode is not normalized, so canonically equivalent but differently encoded Unicode strings remain distinct.
- `from_slice<T>` performs that preflight, then deserializes the same bytes. Maximum 2 MiB, 64 nested value levels, 200,000 value nodes. Existing smaller mapping/request limits also apply. Errors are deterministic for the same input and library version; no arbitrary key contents are echoed.
- `PolicyPackage::import_strict` applies preflight to original manifest.json and capsule.json after existing bounded archive extraction, before typed JSON parsing. It requires schema 2 and never migrates v1. Legacy `import` remains unchanged, including its migration behavior. The local native-v2 catalog opts into strict import.
- Compact raw `compile_json` and CLI mapping/request input use strict parsing. Embedded artifacts are checked before projection, while ordinary opaque evidence bytes remain untouched. Known profile/recursive field allowlists still apply.

This does not implement JCS, source authenticity, authorization or semantic truth. Numbers retain existing serde_json and target-type behavior: finite floating values may round; this is not arbitrary precision numeric normalization. Hashes remain bound to exact original archive/artifact bytes. Already-deserialized typed values cannot reveal duplicate keys erased upstream; callers must use the raw strict boundary. Other legacy CLIs, importers and consumers are not silently upgraded by this patch.

## Verification

From repository `edgequake`, with Rust 1.95 and existing dependencies:

`cargo test --offline --workspace --all-targets`

The isolated three-crate workspace passes 88 tests (8 new). Tests cover nested/array duplicates, decoded ASCII/BMP/supplementary Unicode equivalence, independent object scopes, no Unicode normalization, exact depth/node/byte limits, malformed UTF-8/surrogates/syntax/trailing JSON, numeric edges, raw mapping/request rejection, native archive manifest/data rejection and embedded artifact rejection. Existing compact profile and source/provenance/budget tests pass. All three corrected real-capsule benchmark outputs remain byte-identical to PR10. This is not a full monorepo CI claim.
