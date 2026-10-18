# Explicit empty-evidence receipts

Compact exports now carry an optional machine-readable evidence_status: selected_evidence or omission_only. New exports always emit it, and its bytes count toward the same budget. Empty exports prominently state that no evidence was selected and that the document is an omission receipt, not an answer. Text begins with OMISSION RECEIPT - NO EVIDENCE SELECTED before any metadata.

Compatibility: existing document format and fields remain. Old documents without the optional field deserialize and reserialize without adding it; status derives from actual retained claims. A supplied status inconsistent with claims is rejected. Native capsule schemas, evidence bytes, provenance and selection policy do not change.

The CLI preserves exit 0 for successfully generated documents, including receipts, and emits TACITUS_EVIDENCE_STATUS plus FINDINGS on stderr. Automation must inspect evidence_status (or the stderr status) before treating output as evidence. Error/refusal remains exit 1 with empty stdout. This is not a claim that process success means the question was answered.

Verification: 90 tests, Clippy warnings-as-errors, and 18 real CLI cases pass. 16 KiB produces zero-finding receipts for all three cases, 32 KiB retains three findings each, and 8 KiB refuses. New 32 KiB JSON sizes are 28,995/29,136/29,718 bytes; text sizes are 30,249/30,204/31,038. Payload hashes and lengths change because status is now included; no unchanged-byte claim is made. Old zero-evidence artifacts remain historical.

The same delta applied cleanly to the combined PR6–11 tree and passed 107 TACITUS tests through the actual full repository Cargo.toml with --offline --locked, including the separate v1 cited-brief and evaluation crates. This is a four-package test selection, not full application build/runtime verification. Full application gates still require the bundled third-party PDFium binary path and missing runtime/WebUI prerequisites; no approval, deployment or full gate success is implied.
