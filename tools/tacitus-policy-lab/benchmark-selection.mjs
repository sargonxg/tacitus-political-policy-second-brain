// SPDX-License-Identifier: Apache-2.0
// Benchmark comparators only. No gold labels, learned salience or production retrieval.
import {contextRequest} from './core.mjs';
import {eligibleInput, selectForChangeReview} from './change-review.mjs';
export const ARMS = ['input-order', 'lexical-overlap', 'all-eligible-reference', 'change-first'];
const tokens = text => new Set(text.toLowerCase().match(/[\p{L}\p{N}]+/gu) ?? []);

export async function prepareCase(input) {
  return {...await eligibleInput(input), review_since:input.review_since};
}

export function orderAssertions(assertions, question, arm) {
  if (!ARMS.includes(arm)) throw new Error('Unknown benchmark arm');
  if (arm !== 'lexical-overlap') return [...assertions];
  const query = tokens(question);
  return assertions.map((assertion, index) => {
    const words = tokens(assertion.proposition);
    return {assertion, index, score:[...query].filter(word => words.has(word)).length};
  }).sort((a,b) => b.score-a.score || a.index-b.index).map(row => row.assertion);
}

export async function selectContext(prepared, arm) {
  if (arm === 'change-first') {
    const result = await selectForChangeReview(prepared, prepared.review_since);
    return {...result.context, change_review:result.review};
  }
  const assertions = orderAssertions(prepared.assertions, prepared.contract.question, arm);
  return contextRequest({sources:prepared.sources, assertions, ...prepared.contract,
    max_bytes:arm === 'all-eligible-reference' ? 250000 : prepared.contract.max_bytes});
}
