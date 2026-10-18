// SPDX-License-Identifier: Apache-2.0
// Developer-label scoring, deliberately separate from benchmark selectors.
import {key, passage, validateSource, dependencyImpact} from './core.mjs';
export const fraction = (numerator, denominator) => ({numerator, denominator,
  rate:denominator ? numerator / denominator : null});

export async function scoreContext(context, input, gold) {
  for (const source of input.sources) await validateSource(source);
  const sources = new Map(input.sources.map(s => [key(s), s]));
  const assertions = new Map(input.assertions.map(a => [a.id, a]));
  const eligible = new Set(gold.eligible), supported = new Set(), families = new Set();
  let totalCitations = 0, validCitations = 0, supportedRows = 0;
  const selected = new Set(), evidence = context.packet.evidence, rowSupport = [];
  for (const row of evidence) {
    selected.add(row.assertion_id);
    const citations = row.citations ?? [];
    const citationChecks = citations.map(citation => {
      totalCitations++;
      const source = sources.get(citation.source);
      let valid = false;
      try {
        valid = !!source && citation.span.source === citation.source &&
          citation.source_family === source.source_family && passage(citation.span, source) === citation.quote;
      } catch { /* Invalid citations receive zero credit, never fabricated support. */ }
      if (valid) { validCitations++; families.add(source.source_family); }
      return valid;
    });
    const original = assertions.get(row.assertion_id);
    const literalSupport = !!original && row.proposition === original.proposition && citations.length > 0 &&
      citationChecks.every(Boolean) && row.epistemic_type === original.epistemic_type &&
      citations.every(c => original.evidence.some(span => Object.keys(span).every(field => span[field] === c.span[field]))) &&
      citations.some(c => c.quote === row.proposition);
    rowSupport.push(literalSupport && eligible.has(row.assertion_id));
    if (literalSupport) {
      supportedRows++;
      if (eligible.has(row.assertion_id)) supported.add(row.assertion_id);
    }
  }
  const newlyAvailable = input.assertions.filter(a => eligible.has(a.id) &&
    Math.max(a.available.start, ...a.evidence.map(e => sources.get(e.source).available.start)) > input.review_since).map(a => a.id);
  const critical = [...new Set(gold.critical)];
  const qualifiers = gold.qualifications;
  const preserved = q => supported.has(q.id) && evidence.some(row => row.assertion_id === q.id && row.proposition.includes(q.text));
  const selectedQualifiers = evidence.flatMap((row,index) => qualifiers.filter(q => q.id === row.assertion_id).map(q => ({...q,index})));
  const occurrencePreserved = q => rowSupport[q.index] && evidence[q.index].proposition.includes(q.text);
  return {
    selected_ids:[...selected], duplicate_record_count:evidence.length-selected.size,
    new_evidence_retention:fraction(newlyAvailable.filter(id => supported.has(id)).length, newlyAvailable.length),
    missing_new_evidence:newlyAvailable.filter(id => !supported.has(id)),
    critical_claim_coverage:fraction(critical.filter(id => supported.has(id)).length, critical.length),
    missing_critical_claims:critical.filter(id => !supported.has(id)),
    qualification_coverage:fraction(qualifiers.filter(preserved).length, qualifiers.length),
    selected_qualification_integrity:fraction(selectedQualifiers.filter(occurrencePreserved).length, selectedQualifiers.length),
    citation_integrity:fraction(validCitations, totalCitations),
    literal_claim_support:fraction(supportedRows, evidence.length),
    ineligible_ids:[...selected].filter(id => !eligible.has(id)),
    declared_source_families:families.size,
    warning:'Literal quote/ID checks are not truth, entailment, source independence, or human quality scores.'
  };
}

export function scoreReviewQueue(queue, records, gold) {
  const universe = new Set(records.map(record => record.id));
  const expected = new Set(gold.affected);
  if ([...expected].some(id => !universe.has(id))) throw new Error('Impact gold outside reviewable universe');
  const selected = [...new Set(queue)].filter(id => universe.has(id));
  const found = selected.filter(id => expected.has(id));
  return {queue_ids:selected.sort(), queue_size:selected.length, reviewable_records:universe.size,
    affected_recall:fraction(found.length, expected.size), queue_precision:fraction(found.length, selected.length),
    missed_affected:[...expected].filter(id => !selected.includes(id)).sort(),
    unnecessary_reviews:selected.filter(id => !expected.has(id)),
    nonqueued_records:universe.size-selected.length};
}

export function benchmarkChange(input, gold) {
  const incomplete = structuredClone(input.records);
  incomplete.find(record => record.id === 'participation-assumption').dependencies = [];
  return [
    {arm:'review-all', ...scoreReviewQueue(input.records.map(r => r.id), input.records, gold)},
    {arm:'declared-dependency-closure', ...scoreReviewQueue(dependencyImpact(input.changed, input.records).affected, input.records, gold)},
    {arm:'missing-link-negative-control', ...scoreReviewQueue(dependencyImpact(input.changed, incomplete).affected, input.records, gold)}
  ];
}
