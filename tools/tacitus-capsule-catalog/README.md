# TACITUS local capsule catalog

A runnable, dependency-free development slice for discovering, exchanging and attaching
policy context and cognitive-method proposals. This is the local precursor to a
marketplace, not a deployed marketplace, payment service or authenticated PRAXIS UI.

Requires Node 22+ (tested Node 24 on Linux). No installation or API key.

```sh
node --test tools/tacitus-capsule-catalog/test.mjs
node tools/tacitus-capsule-catalog/cli.mjs demo /tmp/my-tacitus-catalog
node tools/tacitus-capsule-catalog/cli.mjs list /tmp/my-tacitus-catalog pilot
node tools/tacitus-capsule-catalog/cli.mjs export /tmp/my-tacitus-catalog fictional-pilot-capsule@1 /tmp/pilot.exchange.json
node tools/tacitus-capsule-catalog/cli.mjs import /tmp/another-catalog /tmp/pilot.exchange.json
node tools/tacitus-capsule-catalog/cli.mjs attach /tmp/another-catalog fictional-pilot-capsule@1 1780315201
node tools/tacitus-capsule-catalog/cli.mjs retract /tmp/another-catalog fictional-pilot-capsule@1 1780315202 'Correction needs review'
```

Paths are explicit local paths. Export refuses to overwrite a file. Re-running demo
is idempotent for the exact same revision; a locally retracted revision stays retracted.
The historical timestamp is intentional for the fictional fixture. Real attachment
callers must supply current trusted time, not a caller-chosen date to defeat expiry.

## Product behavior

- Discover by title, policy domain, scope or context/cognitive profile.
- Keep immutable capsule ID + package version + content digest; reject version collision.
- Import and export a small canonical JSON exchange of existing native-v1 pack inputs.
- Check pinned schema, source hashes, exact UTF-8 evidence spans, workspace, provenance,
  declared redistribution permission and limitations. Unknown fields are rejected.
- Produce an explicitly unreviewed attachment proposal with a pinned digest, rights,
  source references and limitations. Does not execute imported instructions or tools.
- Recheck local retraction/revocation plus capsule/source/assertion availability and
  expiry at every attachment operation. Local operator can reverse a mistaken retirement
  with a reason; append-only in-memory event history is saved with the catalog.
- Persist imports and local status history across process restarts. Duplicate imports
  cannot resurrect a locally retracted exact revision. New revisions do not silently
  retire old ones: a reviewer must decide that separately.

## Format boundaries and integration

The exchange format is `tacitus-local-catalog-exchange/1` with exactly:
`format`, `content_digest`, and `inputs`. Inputs retain the existing
`{manifest,data,evidence}` native-v1 **pack-input** shape. This is not a ZIP capsule,
not the PRAXIS `tacitus-native-capsule/1` camelCase format, not native policy v2, and
not DIALECTICA v3. Unsupported versions and unresolved dependencies fail closed.
No implicit conversion, native validation or migration is claimed.

`content_digest` is lowercase, unprefixed SHA-256 of UTF-8 `canonical(inputs)`.
Canonicalization recursively sorts object keys, preserves array order, emits compact
JSON, allows only safe integer numbers, and rejects malformed Unicode, unsafe keys and
depth >=48. It is this adapter's serialization contract, not RFC8785/signing.
Imports require the entire exchange to use that exact serialization; duplicate keys
and alternative whitespace/encodings are rejected rather than normalized invisibly.

The separately named `tacitus-praxis-attachment-proposal/1` descriptor carries:
- capsule_id, package_version, content_digest, native_schema_version (=1)
- title and profile (context, cognitive_method, output_artifact)
- review_status (=imported-unreviewed), trust (=untrusted-data)
- rights {license, confidentiality, sharing_policy}
- attached_at (epoch seconds), valid_until (epoch seconds or null)
- source_refs (sourceID@version strings), limitations (strings)

A PRAXIS integration must independently validate bytes, bind the digest to an explicit
local review and owner/workspace, enforce its own access rights and recheck current
status/time on each use. The descriptor itself grants no permission, proves no identity,
and must not promote proposed claims to accepted facts. Shared test fixtures are under
`fixtures/`; every name and evidence passage is fictional, authored for the demo.

## Security and honest limits

Hashes detect corruption, not authorship, truth, entitlement, or malicious instructions.
Rights are declarations requiring independent verification before public distribution.
Catalogs are local/private by default. Import can contain quotations or sensitive claims;
review everything before export. No remote publication, listing, upload, buyer, payment,
payout, credential, model execution or URL fetch is performed. No fees or revenue claimed.

References-only records can be inspected but cannot attach as verified evidence. Imported
reviews are refused, and all output remains unreviewed. The adapter is stricter than native
v1 and does not purport to implement every native feature. Native Rust ZIP round-trip,
production PostgreSQL/AGE integration, authenticated multi-user PRAXIS, browser acceptance,
and Windows testing have not run. Rust remains the authoritative native implementation.

A remote withdrawal cannot recall exported copies or reach an offline recipient.
A fresh catalog cannot learn external revocations without a future authenticated update
channel. Local stores are trusted operator-owned files, not encrypted or cryptographically
signed audit logs; an operator who edits/restores an older store can alter history.
Atomic replace and exclusive writer lock prevent routine simultaneous writes; a crashed
writer may leave `.catalog.lock`/`.catalog.next`. Inspect for an active process and preserve
files before manually recovering. The CLI does not silently clear uncertain locks.

Limits: 2 MiB canonical inputs/exchange, pinned native schema bounds (32 sources,
256 assertions, 32 methods), 100 local entries, 1,000 local status events, 16 MiB store.
There is no private-source public marketplace permission model yet. Implement that in the
existing authenticated product rather than deploying this local tool as a public server.
