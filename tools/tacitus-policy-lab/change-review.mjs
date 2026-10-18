// SPDX-License-Identifier: Apache-2.0
// Explicit checkpoint-driven review aid. Recency is not truth or general salience.
import {contextRequest, key, briefPreparation} from './core.mjs';

export async function eligibleInput(input) {
  // Reuse lab access/time validation before ordering. No separate semantic validator.
  const context = await contextRequest({...input, ...input.contract, max_bytes:250000});
  if (context.receipt.omissions.some(o => o.reason === 'byte-budget')) {
    throw new Error('Inputs exceed eligibility-reference budget; refusing a truncated comparison');
  }
  const ids = new Set(context.receipt.selected);
  const assertions = input.assertions.filter(a => ids.has(a.id));
  const refs = new Set(assertions.flatMap(a => a.evidence.map(e => e.source)));
  return {sources:input.sources.filter(s => refs.has(key(s))), assertions, contract:{...input.contract},
    eligibility_omissions:context.receipt.omissions};
}

export function changeOrder(prepared, sinceKnownAsOf) {
  if (!Number.isSafeInteger(sinceKnownAsOf) || sinceKnownAsOf > prepared.contract.known_as_of) {
    throw new Error('Review checkpoint must be an integer at or before known_as_of');
  }
  const sources = new Map(prepared.sources.map(source => [key(source), source]));
  const rows = prepared.assertions.map((assertion, index) => {
    const time = Math.max(assertion.available.start, ...assertion.evidence.map(span => {
      const source = sources.get(span.source);
      if (!source) throw new Error('Missing source in prepared review input');
      return source.available.start;
    }));
    return {assertion, index, time, incoming:time > sinceKnownAsOf};
  });
  rows.sort((a,b) => Number(b.incoming)-Number(a.incoming) ||
    (a.incoming && b.incoming ? b.time-a.time : 0) || a.index-b.index);
  return {assertions:rows.map(row => row.assertion), incoming:rows.filter(row => row.incoming).map(row => row.assertion.id)};
}

export async function selectForChangeReview(prepared, sinceKnownAsOf) {
  const order = changeOrder(prepared, sinceKnownAsOf);
  const context = await contextRequest({sources:prepared.sources, assertions:order.assertions, ...prepared.contract});
  const selected = new Set(context.receipt.selected), incoming = new Set(order.incoming);
  const missing = order.incoming.filter(id => !selected.has(id));
  return {context, review:{method:'newly-available-first/1', since_known_as_of:sinceKnownAsOf,
    status:missing.length ? 'blocked-new-evidence-omitted' : 'needs-human-review',
    newly_available_ids:order.incoming, selected_new_ids:order.incoming.filter(id => selected.has(id)),
    omitted_new_ids:missing,
    omitted_existing_ids:context.receipt.omissions.filter(o => !incoming.has(o.id)).map(o => o.id),
    unhandled_change_types:['withdrawal-or-access-revocation','expiry','correction-with-unchanged-availability'],
    warning:'Availability-only review can miss revocation, expiry or corrections with unchanged availability. Newly available does not mean corrected, authoritative, independent, relevant or true. Historical evidence can be displaced; inspect every omission. No accepted judgment is changed.'}};
}

export async function changeReviewContext(input, sinceKnownAsOf) {
  return selectForChangeReview(await eligibleInput(input), sinceKnownAsOf);
}

export function changeReviewBrief(result) {
  if (result.review.omitted_new_ids.length) throw new Error('Brief blocked: newly available evidence did not fit. Increase the budget or explicitly narrow the source set.');
  return ['# Change-review preparation — HUMAN REVIEW REQUIRED',
    `Checkpoint (UTC epoch seconds): ${result.review.since_known_as_of}`,
    `Newly available evidence selected: ${result.review.selected_new_ids.join(', ') || 'none'}`,
    `Existing evidence omitted: ${result.review.omitted_existing_ids.join(', ') || 'none'}`,
    result.review.warning, '', briefPreparation(result.context)].join('\n');
}
