// SPDX-License-Identifier: Apache-2.0
// Newly authored fictional text below is CC0-1.0. Never real policy evidence.
import {sourceVersion, assertionProposal} from './core.mjs';
import {demo, DEMO_TIME} from './demo.mjs';

export async function benchmarkInputs() {
  const pilot = await demo();
  const sources = [], assertions = [];
  const documents = [
    ['notice', 'Civic notice', 'Civic press office', 'The river transit launch campaign will celebrate service, investment and regional cooperation.', 'press-office', 'position', 0],
    ['authority', 'Authority circular', 'River ministry', 'Only the ministry may authorize operations; the advisory board can recommend but cannot grant permission.', 'ministry', 'authority', 0],
    ['safeguard', 'Operations condition', 'River inspectorate', 'Service may begin only after an independent safety inspection. No inspection certificate has been issued.', 'inspectorate', 'constraint', 0],
    ['translation', 'Original-language notice', 'River inspectorate', 'Avis : le départ reste soumis à une inspection indépendante.', 'inspectorate', 'position', 0],
    ['withdrawal', 'Later suspension', 'River ministry', 'The ministry suspended the provisional authorization. Operations must not start pending a new decision.', 'ministry', 'authority', 86400]
  ];
  for (const [id, title, author, text, source_family, profile, offset] of documents) {
    const source = await sourceVersion({id, title, author, text, source_family, workspace:'fictional-river',
      published_at:DEMO_TIME + offset, retrieved_at:DEMO_TIME + offset,
      language:id === 'translation' ? 'fr' : 'en',
      redistributable:true, license:'CC0-1.0 (synthetic fixture)'});
    sources.push(source);
    assertions.push(await assertionProposal(source, text, {id:`claim-${id}`, profile}));
  }
  const river = {sources, assertions, contract:{question:'Can the river transit launch proceed, and who can permit operations?',
    workspace:'fictional-river', known_as_of:DEMO_TIME, valid_at:DEMO_TIME, max_bytes:2400}};
  return [
    {id:'pilot-before', sources:pilot.sources, assertions:pilot.assertions, contract:{...pilot.contract, max_bytes:2400}},
    {id:'pilot-after', sources:pilot.sources, assertions:pilot.assertions,
      contract:{...pilot.contract, known_as_of:DEMO_TIME+86400, valid_at:DEMO_TIME+86400, max_bytes:2400}},
    {id:'river-before', ...river},
    {id:'river-after', ...river, contract:{...river.contract, known_as_of:DEMO_TIME+86400, valid_at:DEMO_TIME+86400}}
  ].map(input => ({...input, review_since:DEMO_TIME}));
}

// Declared dependency links are fixture inputs, not discovered relationships.
export const CHANGE_INPUT = {
  changed:['delegation@1'],
  records:[
    {id:'claim-delegation', dependencies:['delegation@1']},
    {id:'participation-assumption', dependencies:['claim-delegation']},
    {id:'pilot-option', dependencies:['participation-assumption']},
    {id:'pilot-brief', dependencies:['pilot-option','funding-note']},
    {id:'funding-note', dependencies:['budget@1']},
    {id:'river-brief', dependencies:['river-note']},
    {id:'river-note', dependencies:['authority@1']},
    {id:'archived-glossary', dependencies:[]}
  ]
};
