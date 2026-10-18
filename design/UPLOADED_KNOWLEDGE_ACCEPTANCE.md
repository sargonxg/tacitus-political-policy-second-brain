# Uploaded-source knowledge reuse: acceptance proposal

## Existing contracts reused
Native-v2 PolicyPackage; SourceRecord with original/text digest, byte-addressed Span and separate published/retrieved/available/valid timestamps; attributed PolicyAssertion with conditions/exceptions; tacitus-policy/1 relations; revision-bound layers::Graph nodes/links; strict JSON parser; existing byte-budget closure compiler. No native-v1 conversion.

The current native-v2 review builds an empty graph. The next bounded path must use an explicit recognized hash-bound graph artifact instead of quietly dropping uploaded structure.

## Extraction boundary
Raw UTF-8 documents are evidence. An extractor adapter produces proposals with exact source/version/hash/byte-span references, typed entity candidates, attributed relation/claim candidates, observation timestamps and optional world-validity intervals. Original documents remain separate from candidate paraphrases. No candidate carries review authority. Existing EdgeQuake EntityExtractor is a future adapter source, but chunk-text spans are not automatically sufficient to establish exact native byte offsets; no implicit bridge or live model call is claimed.

A deterministic line extractor can supply source-grounded statement candidates without manually typing every statement. It cannot infer entities, semantic relations, temporal truth or corrections. Explicit annotations demonstrate those semantics; a model/human extraction adapter is required for free-form understanding.

## Concrete fixture before implementation
Two original local UTF-8 policy notes, explicitly licensed for redistribution. Note one attributes a conditional port-access commitment to an institution and separately describes a heat-response event. Note two withdraws the commitment and preserves its exception; its recorded time is later. Include a non-ASCII actor name and an instruction-looking passage as quoted data.

1. Upload exact text and metadata; publication and world validity unknown remain null.
2. Derive line candidates and byte spans. Import explicit typed annotations for one actor, institution and event, and a revision-bound withdrawal/supersession edge. Observation time is distinct from world validity.
3. Strictly validate and package into native-v2 ZIP, with hash-bound recognized graph artifact; zero entities/links must not silently become successful semantic capture.
4. Query about port access versus heat response produces different graph-selected evidence. Follow explicit correction/dependency closure; no old promise without the withdrawal in a current snapshot.
5. Earlier known-as-of checkpoint cannot leak later annotations. Strict valid-at-date query rejects unknown validity; disclosure mode labels it per claim.
6. Withdrawing a cited source prevents later reuse. Export includes exact citations, unknown status, qualifiers, omitted IDs and selection reason. Injection-like source text is data.
7. Roundtrip preserves exact bytes and fingerprints; same request gives identical output; budget measures actual serialized payload. No-match is an omission-only receipt.

## Scope
No PDF binary, hosted database, paid model, credentials, Foundry deployment, automatic truth/rights verification or universal ontology mapping. This is source-backed knowledge-structure plumbing and deterministic retrieval, not completion of the full semantic product.

This bounded capture profile requires at least one typed entity and one explicit claim relationship. Entity-only extraction remains a proposal, not a completed capture under this profile.
