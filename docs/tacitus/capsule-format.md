# Provisional .capsule format v1

An ordinary ZIP contains `manifest.json`, `capsule.json`, optional extracted-text files
under `evidence/<SHA256-of-source-version-key>.txt`. Files are stored without compression;
ordering is lexical, ZIP metadata fixed, JSON compact and deterministic for these Rust
records/BTreeMaps. This is a project canonicalization rule, not RFC 8785 or signing.

Manifest: schema/package versions, identity/author/title/domain/scope/purpose/profile,
creation/expiry, supersession, pinned dependency versions/digests, license,
confidentiality/sharing policy, export mode/limitations and per-file SHA-256 hashes.
The manifest does not hash itself. `capsule.json` carries source metadata, assertions,
declarative cognitive objects, reviews and textual artifacts. Source text is prohibited
in metadata; evidence inclusion is explicit. Original bytes are retained by ingestion
records but portable original-byte attachment is not implemented yet.

References-only requires explicit limitations and cannot compile without supplied
authorized evidence. Evidence-included checks rights/license/access/withdrawal and
text hashes. Preview returns every exact included file/field. No automatic upload,
URL dependency resolution, executable code, tools or permissions are activated.

Limits: 8 MiB archive/expanded total; 2 MiB per entry; 64 entries; 32 source versions;
256 assertions; 32 methods; 16 pinned dependencies. Import is in-memory without path
extraction; traversal, absolute/Windows paths, unexpected payloads, symlinks, duplicates,
oversized entries, mismatched hashes and unsupported schema versions are rejected.
Imports downgrade every review to an imported attestation. Hashes prove integrity only.

Supported create/preview/validate/inspect/export/import/rebind/compile are developer
library/file operations. UI editing, compare, composition/conflict receipts, redaction
and migration from future versions are not implemented. Schema 2 is rejected; no imaginary
migration is claimed. Praxis compatibility is provisional and unverified.

Revocation cannot recall exported copies. Offline recipients cannot automatically learn
about later withdrawal. Imported rights statements remain untrusted attestations;
the integrated server must independently enforce local access and disclosure consent.
