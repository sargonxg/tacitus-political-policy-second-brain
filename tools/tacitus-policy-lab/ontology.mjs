// SPDX-License-Identifier: Apache-2.0
// Original Tacitus vocabulary. This is a versioned application ontology, not an
// endorsed international standard. Relations are claims, not automatically true edges.
const concept = (id, label, definition) => ({ id, label, definition });
export const ONTOLOGY = Object.freeze({
  id: 'urn:tacitus:policy:0.1', version: '0.1.0', status: 'application-vocabulary',
  concepts: [
    concept('actor', 'Actor', 'An attributed individual or collective participant; identity matches require evidence.'),
    concept('institution', 'Institution', 'A body whose authority and membership may change over time.'),
    concept('claim', 'Claim', 'An attributed proposition, not an assertion that its contents are true.'),
    concept('position', 'Position', 'An expressed stance tied to speaker, audience, wording and date.'),
    concept('interest', 'Interest', 'A declared interest or separately labeled analyst inference.'),
    concept('constraint', 'Constraint', 'A legal, institutional, political, resource or practical limit.'),
    concept('leverage', 'Leverage', 'An evidenced capacity to affect another actor, not assumed control.'),
    concept('commitment', 'Commitment', 'An undertaking preserving action, conditions, authority and implementation status.'),
    concept('event', 'Event', 'An occurrence with event time distinct from reporting and knowledge times.'),
    concept('narrative', 'Narrative', 'An attributed explanatory frame, not an established causal account.'),
    concept('mandate', 'Mandate', 'A source-bound assignment of scope and powers.'),
    concept('authority', 'Decision authority', 'Who may authorize a specific action under specified rules.'),
    concept('funding', 'Funding stage', 'Authorization, allocation, obligation, disbursement and expenditure are distinct.'),
    concept('instrument', 'Policy instrument', 'A measure with mechanism, authority and implementation conditions.'),
    concept('option', 'Policy option', 'A possible action, including the status quo, distinct from a recommendation.'),
    concept('hypothesis', 'Hypothesis', 'An explanation to test against alternatives and discriminating evidence.'),
    concept('assumption', 'Assumption', 'A premise requiring explicit scope and a revision trigger.'),
    concept('indicator', 'Indicator', 'An observable signpost with limits and possible false positives.'),
    concept('forecast', 'Forecast', 'A resolvable forward statement with horizon and outcome criteria.'),
    concept('decision', 'Decision', 'An attributable choice with alternatives, rationale and subsequent outcomes.'),
    concept('judgment', 'Analytical judgment', 'A supported interpretation separated from source wording and value preferences.'),
    concept('source', 'Source version', 'An immutable version with rights, availability, valid time and provenance.'),
    concept('evidence_gap', 'Evidence gap', 'Missing information whose acquisition could change the assessment.'),
    concept('affected_group', 'Affected group', 'A population experiencing benefits, costs or rights impacts; not individual targeting.')
  ],
  relations: [
    ['holds_position', 'actor', 'position'], ['expresses_interest', 'actor', 'interest'],
    ['faces_constraint', 'actor', 'constraint'], ['makes_commitment', 'actor', 'commitment'],
    ['mandated_by', 'institution', 'mandate'], ['authorizes', 'authority', 'option'],
    ['depends_on', '*', '*'], ['supports', 'source', 'claim'],
    ['challenges', 'claim', 'claim'], ['supersedes', 'claim', 'claim'],
    ['affects', 'option', 'affected_group'], ['tests', 'indicator', 'hypothesis']
  ].map(([id, subject, object]) => ({ id, subject, object, requires_evidence: true })),
  epistemic_types: ['reported_statement','observation','allegation','inference','hypothesis','forecast','value_judgment','recommendation'],
  distinctions: [
    ['discussion', 'acceptance'], ['conditional willingness', 'unconditional commitment'],
    ['authorization', 'expenditure'], ['alignment', 'membership or control'],
    ['reported allegation', 'established event'], ['inferred motive', 'declared interest'],
    ['scenario', 'forecast'], ['source repetition', 'independent confirmation']
  ]
});
export const MODULES = Object.freeze([
  { id: 'semantic-checks', title: 'Language and institutional distinctions', status: 'implemented', input: 'selected assertions and source spans', output: 'review flags', network: false },
  { id: 'decision-brief', title: 'Decision brief preparation', status: 'implemented', input: 'decision contract and selected assertions', output: 'cited extractive brief and open questions; not LLM analysis', network: false },
  { id: 'competing-hypotheses', title: 'Competing hypotheses', status: 'implemented', input: 'hypotheses and explicit evidence judgments', output: 'matrix completeness and source-family diagnostics; no posterior probability', network: false },
  { id: 'change-impact', title: 'Dependency impact', status: 'implemented', input: 'changed IDs and dependency edges', output: 'transitive affected IDs; does not revise judgments', network: false },
  { id: 'cognitive-memory', title: 'Scoped method proposals', status: 'implemented', input: 'correction, provenance, prerequisites, exceptions', output: 'suggested, blocked, stale or guarded method', network: false },
  { id: 'native-capsule-input', title: 'Rust capsule pack input', status: 'implemented', input: 'authorized selected sources, assertions and methods', output: 'manifest.json, data.json, evidence files; Rust ZIP pack/validation still required', network: false }
]);
