# Minister/diplomatic briefing acceptance fixture

## What runs

This fixture reuses the uploaded-knowledge CLI and existing native-v2 graph contracts without changing the ontology or parser. It captures a small, source-backed historical legislative dossier and supplies purpose-limited context for a publication briefing or French-language speech preparation.

- Official English and French Article 113 excerpts remain separate records.
- An assistant English translation of the French span is explicitly marked unreviewed and bound to that exact original span. It is not the official English text.
- Original TACITUS research notes identify publication metadata, phased exceptions and a later-version warning. They are never labeled publisher quotations.
- Public institutional nodes name the European Parliament and Council. No personal contacts, ideological affiliations or political beliefs are inferred.
- Calendar dates retain date-only precision and EU jurisdiction. Event instants and publication timestamps stay null where the source only supplies dates.
- A drafting recommendation is labeled recommendation, not fact, approved speech or ministerial position. Its explicit evidence dependencies bring both originals, exceptions and version warning together.

## Official sources checked 18 October 2026

Original English act: https://eur-lex.europa.eu/eli/reg/2024/1689/oj/eng
Original French act: https://eur-lex.europa.eu/eli/reg/2024/1689/oj/fra
Reuse notice: https://eur-lex.europa.eu/content/legal-notice/legal-notice.html

The original record lists Official Journal publication on 12 July 2024. Its Article 113 contains a general application date and phased exceptions. Crucially, the page marks the act as changed and links a consolidated version dated 27 July 2026. This fixture does not determine the current legal effect of those changes. It must not be used as an actionable deadline service or legal advice.

Only short Article 113 excerpts are embedded. Legal documents may be reused under the cited EUR-Lex notice, subject to any document-specific conditions. The supplied evidence hashes cover uploaded excerpt/note bytes, not the complete official webpage or authenticated Official Journal PDF. Native integrity verification does not establish legal authenticity.

## Exact acceptance cases

Run the existing knowledge_capture CLI against tests/fixtures/minister/capture.json. Queries are supplied beside it.

1. `publication institutions` returns exactly `publication-record`, plus the sourced institutional/date-only event nodes.
2. `discours` returns `speech-guidance`, `application-en`, `application-fr`, `phased-exceptions`, and `version-warning` through declared dependency closure.
3. `rhétorique diplomatique` returns an explicit omission-only receipt. This control demonstrates the absence of semantic cross-language retrieval.
4. Strict-date mode rejects unknown validity. Date-only event_time and published_at remain null; the original date and precision remain in retained conditions.
5. The French quote, original-language label, separate translation and translation authorship/review conditions survive native packing and context reuse.

These are annotations over checked sources. Automatic literal extraction does not infer legal stages, translations or semantic relations. The bilingual institution label is explicitly authored, not a learned cross-language match.

## Bounded ontology roadmap

The next schema proposal should separate a source ledger, reviewable claim/event records and task-specific capsules. Add only fields backed by concrete workflows:

- Translation record: original span fingerprint, target language, translator, method, revision, review status and qualified terminology. Do not overload a free-text translation string indefinitely.
- Legislative event: stage vocabulary, jurisdiction, calendar date/precision, source version, recorded time and applicability status. Publication, adoption, entry into force and application are distinct events.
- Public institutional role: organization, post/role, named holder only with public evidence, valid interval and source. Membership is not a political-belief inference.
- Revision comparison: explicit contested/superseding/withdrawn relationships and known-at checkpoints. Newer ingestion alone does not override an earlier source.

Standards inform this proposal but no standards conformance or lossless mapping is claimed:

- W3C PROV-O distinguishes derivation, quotation and revision: https://www.w3.org/TR/prov-o/
- Akoma Ntoso represents legislative document lifecycle events and source references. The v1 vocabulary remains a conceptual reference, while Version 2.0 Part 2 (AKN 3.1) became an OASIS Standard effective 2 October 2026. Future adapters must pin their chosen version explicitly: https://docs.oasis-open.org/legaldocml/ns/akn/3.1/index.html and https://groups.oasis-open.org/discussion/akoma-ntoso-v2-0-part-2-akn-3-1-approved-as-an-oasis-standard-effective-2-oct
- W3C ORG is the next reference for public organizational roles, and SKOS for controlled multilingual labels; those adapters are not implemented here.

The graph remains an index over exact evidence. Uncertainty, disagreement and source rights survive into the capsule. No graph database deployment, model call, personal-network ingestion or Foundry execution is asserted.

A future W3C Web Annotation adapter must explicitly map its character-position conventions to TACITUS exact UTF-8 byte offsets; the offsets are not interchangeable. No such standards adapter is implemented or validated by this fixture.
