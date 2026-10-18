// SPDX-License-Identifier: Apache-2.0
import test from 'node:test';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {contextRequest, sourceVersion, assertionProposal} from './core.mjs';
import {demo, DEMO_TIME} from './demo.mjs';
import {eligibleInput, changeOrder, changeReviewContext, changeReviewBrief} from './change-review.mjs';
const later = async () => {const input=await demo();input.contract={...input.contract,known_as_of:DEMO_TIME+86400,valid_at:DEMO_TIME+86400,max_bytes:2400};return input;};

test('newly available correction survives the identical budget that omits it in input order',async()=>{
  const input=await later(), before=structuredClone(input);
  const baseline=await contextRequest({...input,...input.contract}), result=await changeReviewContext(input,DEMO_TIME);
  assert(!baseline.receipt.selected.includes('claim-correction'));
  assert(result.context.receipt.selected.includes('claim-correction'));
  assert.equal(result.context.packet.limits.max_bytes,baseline.packet.limits.max_bytes);
  assert.equal(result.review.status,'needs-human-review');
  assert(result.review.omitted_existing_ids.includes('claim-budget'));
  assert.deepEqual(input,before);
  assert.match(changeReviewBrief(result),/cabinet declined approval/);
  assert.match(changeReviewBrief(result),/Existing evidence omitted: claim-budget/);
});
test('new evidence that cannot fit blocks brief output rather than hiding an omission',async()=>{
  const input=await later();input.contract.max_bytes=700;
  const result=await changeReviewContext(input,DEMO_TIME);
  assert.equal(result.review.status,'blocked-new-evidence-omitted');
  assert.deepEqual(result.review.omitted_new_ids,['claim-correction']);
  assert.throws(()=>changeReviewBrief(result),/Brief blocked/);
});
test('partial admission of two new records still blocks brief rendering',async()=>{
  const input=await later();
  const source=await sourceVersion({id:'large-update',workspace:input.contract.workspace,title:'Fictional update',author:'Demo office',text:'A lengthy newly available report. '.repeat(100),source_family:'office',published_at:DEMO_TIME+86400,retrieved_at:DEMO_TIME+86400});
  input.sources.push(source);input.assertions.push(await assertionProposal(source,source.text,{id:'claim-large-update'}));
  const result=await changeReviewContext(input,DEMO_TIME);
  assert.deepEqual(result.review.selected_new_ids,['claim-correction']);
  assert.deepEqual(result.review.omitted_new_ids,['claim-large-update']);
  assert.equal(result.review.status,'blocked-new-evidence-omitted');
  assert.throws(()=>changeReviewBrief(result),/Brief blocked/);
});
test('source availability and multiple source spans contribute to newness',async()=>{
  const input=await later();input.sources[0].available.start=DEMO_TIME+500;
  const one=await changeReviewContext(input,DEMO_TIME);
  assert(one.review.newly_available_ids.includes('claim-delegation'));
  input.assertions[0].evidence.push(structuredClone(input.assertions.at(-1).evidence[0]));
  const prepared=await eligibleInput(input);
  assert.deepEqual(changeOrder(prepared,DEMO_TIME+600).incoming,['claim-delegation','claim-correction']);
});
test('with no new evidence original order and exact context receipt remain unchanged',async()=>{
  const input=await demo();input.contract.max_bytes=2400;
  const baseline=await contextRequest({...input,...input.contract}), result=await changeReviewContext(input,DEMO_TIME);
  assert.equal(result.context.receipt.digest,baseline.receipt.digest);
  assert.deepEqual(result.review.newly_available_ids,[]);
  assert.equal(result.review.status,'needs-human-review');
  assert(result.review.unhandled_change_types.includes('withdrawal-or-access-revocation'));
  assert.match(changeReviewBrief(result),/can miss revocation, expiry/);
});
test('later knowledge cannot bypass valid-time or source withdrawal',async()=>{
  for(const kind of ['valid','withdrawn','restricted']) {
    const input=await later();
    if(kind==='valid') input.contract.valid_at=DEMO_TIME;
    else input.sources.find(s=>s.id==='correction')[kind]=true;
    const result=await changeReviewContext(input,DEMO_TIME);
    assert(!result.review.newly_available_ids.includes('claim-correction'));
    assert(!result.context.receipt.selected.includes('claim-correction'));
  }
});
test('checkpoint is explicit, inclusive and never later than current knowledge time',async()=>{
  const input=await later(), prepared=await eligibleInput(input);
  for(const value of [undefined,NaN,Infinity,'yesterday',DEMO_TIME+86401])assert.throws(()=>changeOrder(prepared,value));
  assert.deepEqual(changeOrder(prepared,DEMO_TIME+86400).incoming,[]);
});
test('new assertion availability on old source is included without claiming a correction',async()=>{
  const input=await later();input.assertions[0].available.start=DEMO_TIME+500;
  const result=await changeReviewContext(input,DEMO_TIME);
  assert.deepEqual(result.review.newly_available_ids,['claim-correction','claim-delegation']);
  assert.match(result.review.warning,/does not mean corrected/);
});
test('ties preserve original order; recent irrelevant material is not magically judged useful',async()=>{
  const input=await later();
  const source=await sourceVersion({id:'recent-notice',workspace:input.contract.workspace,title:'Fictional notice',author:'Demo office',text:'Office paint is blue.',source_family:'office',published_at:DEMO_TIME+86400,retrieved_at:DEMO_TIME+86400});
  input.sources.push(source);input.assertions.push(await assertionProposal(source,source.text,{id:'claim-paint'}));
  input.contract.max_bytes=2000;
  const result=await changeReviewContext(input,DEMO_TIME);
  assert.deepEqual(result.review.newly_available_ids,['claim-correction','claim-paint']);
  assert(result.context.receipt.selected.includes('claim-paint'));
  assert(result.review.omitted_existing_ids.includes('claim-delegation'));
});
test('review-demo prints exact correction and omissions, or exits without a brief when blocked',()=>{
  const script=fileURLToPath(new URL('./cli.mjs',import.meta.url));
  const run=args=>spawnSync(process.execPath,[script,'review-demo',...args],{encoding:'utf8',timeout:10000});
  const ok=run([]);assert.equal(ok.status,0,ok.stderr);assert.match(ok.stdout,/claim-correction/);assert.match(ok.stdout,/claim-budget/);
  const blocked=run(['--max-bytes','700']);assert.notEqual(blocked.status,0);assert.equal(blocked.stdout,'');assert.match(blocked.stderr,/Brief blocked/);
  for(const args of [['--bad'],['--max-bytes'],['--max-bytes','NaN'],['--max-bytes','2400','extra'],['--max-bytes','999999']])assert.notEqual(run(args).status,0);
});
