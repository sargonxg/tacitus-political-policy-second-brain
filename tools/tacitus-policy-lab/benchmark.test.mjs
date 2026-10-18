// SPDX-License-Identifier: Apache-2.0
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile, mkdtemp, rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
import {bytes, key} from './core.mjs';
import {benchmarkInputs, CHANGE_INPUT} from './benchmark-fixtures.mjs';
import {ARMS, prepareCase, selectContext, orderAssertions} from './benchmark-selection.mjs';
import {scoreContext, fraction, benchmarkChange} from './benchmark-metrics.mjs';
import {runBenchmark, parseBenchmarkArgs, timingSummary} from './benchmark.mjs';
const gold = JSON.parse(await readFile(new URL('./benchmark-gold.json', import.meta.url), 'utf8'));
const fresh = async () => (await benchmarkInputs())[0];
const score = (context,input) => scoreContext(context,input,gold.cases[input.id]);
const context = async input => selectContext(await prepareCase(input),'all-eligible-reference');
function withoutTiming(report) {
  const copy=structuredClone(report);
  for (const item of copy.cases) {delete item.eligibility_ms;for(const arm of item.arms)delete arm.timing;}
  return copy;
}

test('all four fixtures and gold are coherent; reference fits and preserves qualifications', async () => {
  for (const input of await benchmarkInputs()) {
    const labels=gold.cases[input.id], prepared=await prepareCase(input), result=await context(input);
    assert.deepEqual(prepared.assertions.map(a=>a.id),labels.eligible);
    if(input.id.startsWith('river')) {
      assert.equal(input.sources.find(s=>s.id==='translation').language,'fr');
      assert.equal(input.assertions.find(a=>a.id==='claim-translation').evidence[0].original_language,'fr');
    }
    assert.equal(new Set(labels.critical).size,labels.critical.length);
    for(const id of labels.critical) assert(labels.eligible.includes(id));
    for(const q of labels.qualifications) assert(input.assertions.find(a=>a.id===q.id).proposition.includes(q.text));
    assert(!result.receipt.omissions.some(o=>o.reason==='byte-budget'));
    const metrics=await score(result,input);
    for(const field of ['critical_claim_coverage','qualification_coverage','citation_integrity','literal_claim_support'])assert.equal(metrics[field].rate,1);
    assert.deepEqual(metrics.ineligible_ids,[]);
  }
});
test('bounded arms share eligible inputs, exact limits and real serialized-byte accounting',async()=>{
  for(const input of await benchmarkInputs()) {
    const prepared=await prepareCase(input), snapshot=structuredClone(prepared);
    for(const arm of ARMS.filter(arm=>arm!=='all-eligible-reference')) {
      const result=await selectContext(prepared,arm);
      assert.equal(result.packet.limits.max_bytes,input.contract.max_bytes);
      assert.equal(result.receipt.bytes,bytes(JSON.stringify(result.packet)).length);
      assert(result.receipt.bytes<=input.contract.max_bytes);
      assert(result.receipt.omissions.some(o=>o.reason==='byte-budget'));
      assert.deepEqual(prepared,snapshot);
    }
  }
});
test('lexical overlap uses unique Unicode tokens and stable input-order ties, without gold',()=>{
  const assertions=[{id:'a',proposition:'café café'},{id:'b',proposition:'River CAFÉ'},{id:'c',proposition:'river'}];
  const snapshot=structuredClone(assertions);
  assert.deepEqual(orderAssertions(assertions,'café river','lexical-overlap').map(a=>a.id),['b','a','c']);
  assert.deepEqual(assertions,snapshot);
  assert.throws(()=>orderAssertions(assertions,'q','invented'));
});
test('ranking module cannot import gold or scoring and receives no label fields',async()=>{
  const source=await readFile(new URL('./benchmark-selection.mjs',import.meta.url),'utf8');
  assert(!/import.+benchmark-(gold|metrics)/.test(source));
  const prepared=await prepareCase(await fresh());
  assert.deepEqual(Object.keys(prepared).sort(),['assertions','contract','eligibility_omissions','review_since','sources']);
});
test('known and valid time independently exclude future sources; exact boundary admits correction',async()=>{
  const input=await fresh(), time=input.contract.known_as_of;
  for(const [known,valid,want] of [[time,time+86400,false],[time+86400,time,false],[time+86400,time+86400,true]]) {
    const request={...input,contract:{...input.contract,known_as_of:known,valid_at:valid}};
    assert.equal((await prepareCase(request)).assertions.some(a=>a.id==='claim-correction'),want);
  }
  input.sources[0].valid.end=time+1;
  input.assertions[0].valid.end=time+1;
  input.contract.valid_at=time+1;
  assert(!(await prepareCase(input)).assertions.some(a=>a.id==='claim-delegation'));
});
test('restricted and withdrawn records are removed before ordering',async()=>{
  for(const flag of ['restricted','withdrawn']) {
    const input=await fresh();input.sources[0][flag]=true;
    const prepared=await prepareCase(input);
    assert(!prepared.assertions.some(a=>a.id==='claim-delegation'));
    assert(!prepared.sources.some(s=>key(s)==='delegation@1'));
  }
});
for(const [label,mutate] of [
  ['quote',c=>{c.quote+=' fabricated';}],
  ['hash',c=>{c.span.text_hash='0'.repeat(64);}],
  ['version',c=>{c.source='delegation@2';}],
  ['span',c=>{c.span.end++;}],
  ['language',c=>{c.span.original_language='invented';}],
  ['source family',c=>{c.source_family='invented';}]
])test(`scorer detects corrupt ${label} separately from selection`,async()=>{
  const input=await fresh(), result=await context(input);mutate(result.packet.evidence[0].citations[0]);
  const metrics=await score(result,input);
  assert.equal(metrics.citation_integrity.numerator,3);
  assert.equal(metrics.literal_claim_support.numerator,3);
  assert.equal(metrics.critical_claim_coverage.numerator,1);
});
test('valid citation cannot rescue a rewritten proposition or lost qualification',async()=>{
  const input=await fresh(), result=await context(input);
  result.packet.evidence[0].proposition='The delegation has unconditionally committed.';
  const metrics=await score(result,input);
  assert.equal(metrics.citation_integrity.rate,1);
  assert.equal(metrics.literal_claim_support.numerator,3);
  assert.equal(metrics.qualification_coverage.numerator,1);
});
test('empty output has zero coverage and undefined selected-output integrity',async()=>{
  const input=await fresh(), result=await context(input);result.packet.evidence=[];
  const metrics=await score(result,input);
  assert.equal(metrics.critical_claim_coverage.rate,0);
  assert.equal(metrics.citation_integrity.rate,null);
  assert.equal(metrics.selected_qualification_integrity.rate,null);
  assert.equal(metrics.literal_claim_support.rate,null);
  assert.deepEqual(fraction(0,0),{numerator:0,denominator:0,rate:null});
});
test('duplicate output cannot increase critical coverage; reprints share declared origin',async()=>{
  const input=await fresh(), result=await context(input);
  result.packet.evidence.push(structuredClone(result.packet.evidence[0]));
  const metrics=await score(result,input);
  assert.equal(metrics.critical_claim_coverage.numerator,2);
  assert.equal(metrics.duplicate_record_count,1);
  assert.equal(metrics.declared_source_families,3);
});
test('a clean row cannot hide a corrupted duplicate qualification occurrence',async()=>{
  const input=await fresh(), result=await context(input);
  const duplicate=structuredClone(result.packet.evidence[0]);duplicate.proposition='Unconditional participation.';
  result.packet.evidence.push(duplicate);const metrics=await score(result,input);
  assert.equal(metrics.critical_claim_coverage.rate,1);
  assert.equal(metrics.duplicate_record_count,1);
  assert.deepEqual(metrics.selected_qualification_integrity,fraction(2,3));
  assert.deepEqual(metrics.literal_claim_support,fraction(4,5));
});
test('post-cutoff output scored against earlier task exposes temporal leakage',async()=>{
  const inputs=await benchmarkInputs(), result=await context(inputs[1]);
  const metrics=await score(result,inputs[0]);
  assert.deepEqual(metrics.ineligible_ids,['claim-correction']);
});
test('checkpoint candidate retains both updates without overstating total coverage',async()=>{
  for(const input of await benchmarkInputs()) {
    const prepared=await prepareCase(input);
    const baseline=await score(await selectContext(prepared,'input-order'),input);
    const candidate=await score(await selectContext(prepared,'change-first'),input);
    assert.deepEqual(candidate.critical_claim_coverage,baseline.critical_claim_coverage);
    assert.deepEqual(candidate.ineligible_ids,[]);
    if(input.id.endsWith('after')) {
      assert.deepEqual(baseline.new_evidence_retention,fraction(0,1));
      assert.deepEqual(candidate.new_evidence_retention,fraction(1,1));
      assert.deepEqual(candidate.missing_critical_claims,input.id==='pilot-after' ? ['claim-budget'] : ['claim-safeguard','claim-authority']);
      assert.deepEqual(baseline.missing_critical_claims,input.id==='pilot-after' ? ['claim-correction'] : ['claim-withdrawal','claim-safeguard']);
    } else assert.equal(candidate.new_evidence_retention.rate,null);
  }
});
test('change queue counts only reviewable records and missing link harms recall',()=>{
  const [all,tracked,broken]=benchmarkChange(CHANGE_INPUT,gold.change);
  assert.deepEqual([all.queue_size,tracked.queue_size,broken.queue_size],[8,4,1]);
  assert.equal(all.affected_recall.rate,1);assert.equal(tracked.affected_recall.rate,1);
  assert.equal(broken.affected_recall.rate,0.25);
  assert.deepEqual(broken.missed_affected,['participation-assumption','pilot-brief','pilot-option']);
  assert(!tracked.queue_ids.includes('delegation@1'));
});
test('timing percentiles use nearest rank and do not mutate samples',()=>{
  const samples=[4,1,3,2];assert.deepEqual(timingSummary(samples),{samples:4,min_ms:1,p50_ms:2,p95_ms:4,max_ms:4});
  assert.deepEqual(samples,[4,1,3,2]);
});
test('complete reports repeat exactly excluding observed timings; all baselines retained',async()=>{
  const a=await runBenchmark({iterations:2}), b=await runBenchmark({iterations:2});
  assert.deepEqual(withoutTiming(a),withoutTiming(b));
  assert.equal(a.cases.length,4);
  for(const entry of a.cases) {
    assert.deepEqual(entry.arms.map(a=>a.arm),ARMS);
    for(const arm of entry.arms) {
      assert.equal(arm.timing.samples,2);assert(arm.timing.min_ms>=0);
      assert.equal(arm.timing.p95_ms,arm.timing.max_ms);
    }
  }
  assert.equal(a.unmeasured.analyst_minutes,null);assert.equal(a.network_calls,0);
});
for(const args of [['--iterations','0'],['--iterations','1001'],['--iterations','1.5'],['--iterations'],['--out'],['--out','--iterations'],['--bogus'],['--iterations','1','--iterations','2'],['--out','a','--out','b']]) {
  test(`CLI refuses ${JSON.stringify(args)}`,()=>assert.throws(()=>parseBenchmarkArgs(args)));
}
test('CLI needs no subprocess toolchain and refuses overwrite or a missing parent',async()=>{
  const dir=await mkdtemp(join(tmpdir(),'tacitus-benchmark-'));
  const script=fileURLToPath(new URL('./cli.mjs',import.meta.url)), dest=join(dir,'report.json');
  const run=args=>spawnSync(process.execPath,[script,'benchmark',...args],{encoding:'utf8',timeout:15000,env:{PATH:'',HOME:dir}});
  try {
    const result=run(['--iterations','1','--out',dest]);assert.equal(result.status,0,result.stderr);
    const first=await readFile(dest,'utf8');assert.equal(JSON.parse(first).cases.length,4);
    assert.notEqual(run(['--iterations','1','--out',dest]).status,0);
    assert.equal(await readFile(dest,'utf8'),first);
    assert.notEqual(run(['--iterations','1','--out',join(dir,'missing','report.json')]).status,0);
    const stdout=run(['--iterations','1']);assert.equal(stdout.status,0,stdout.stderr);
    assert.equal(JSON.parse(stdout.stdout).status,'synthetic-developer-regression-only');
  } finally {await rm(dir,{recursive:true,force:true});}
});
