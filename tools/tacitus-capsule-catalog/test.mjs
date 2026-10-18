// SPDX-License-Identifier: Apache-2.0
import test from 'node:test';
import assert from 'node:assert/strict';
import {demoOutput, DEMO_TIME} from '../tacitus-policy-lab/demo.mjs';
import {canonical,inspectInputs,exportExchange,importExchange,LocalCatalog} from './catalog.mjs';
const fixture=async()=>structuredClone((await demoOutput()).inputs);
const now=DEMO_TIME+1;
test('native v1 evidence pack inputs round-trip losslessly with pinned digest',async()=>{
 const f=await fixture();const txt=await exportExchange(f);assert.deepEqual(await importExchange(txt),f);
 assert.equal((await inspectInputs(f)).digest,JSON.parse(txt).content_digest);
});
test('catalog discovery import and attachment remain unreviewed',async()=>{
 const c=new LocalCatalog(), f=await fixture(), {id,digest}=await c.add(f);
 assert.equal(c.search('pilot').length,1);assert.equal(c.search('nothing').length,0);
 const a=await c.attach(id,{at:now,allowEvidence:true});assert.equal(a.content_digest,digest);assert.equal(a.review_status,'imported-unreviewed');assert.equal(a.trust,'untrusted-data');assert.equal(a.rights.confidentiality,'private');
 assert.equal((await c.add(f)).duplicate,true);assert.equal(c.search().length,1);
 const copy=await importExchange(await c.export(id));assert.deepEqual(copy,f);
});
test('immutable revision cannot be replaced by different payload',async()=>{
 const c=new LocalCatalog(),f=await fixture();await c.add(f);f.manifest.title+=' altered';await assert.rejects(c.add(f),/collision/);
});
test('returned records and input objects cannot mutate catalog',async()=>{
 const c=new LocalCatalog(),f=await fixture();const {id}=await c.add(f);f.manifest.title='mutated';c.search()[0].title='mutated';assert.notEqual(c.search()[0].title,'mutated');assert.ok(await c.attach(id,{at:now,allowEvidence:true}));
});
test('digest-pinned retraction, restoration, revocation and audit trail',async()=>{
 const c=new LocalCatalog(),{id,digest}=await c.add(await fixture());
 assert.throws(()=>c.transition(id,'0'.repeat(64),'retracted','correction',now),/pinned/);
 c.transition(id,digest,'retracted','Correction requires review',now);
 await assert.rejects(c.attach(id,{at:now,allowEvidence:true}),/Inactive/);await assert.rejects(c.export(id),/Inactive/);
 c.transition(id,digest,'active','Mistaken retirement reversed by local operator',now+1);
 assert.ok(await c.attach(id,{at:now+1,allowEvidence:true}));
 c.transition(id,digest,'revoked','Local access withdrawn',now+2);await assert.rejects(c.attach(id,{at:now+2,allowEvidence:true}),/Inactive/);
 assert.equal(c.history().length,3);c.history()[0].reason='mutated';assert.notEqual(c.history()[0].reason,'mutated');
 assert.throws(()=>c.transition(id,digest,'active','bad time',now),/chronological/);
});
test('old import cannot resurrect locally retracted exact version',async()=>{
 const c=new LocalCatalog(),f=await fixture(),{id,digest}=await c.add(f);c.transition(id,digest,'retracted','changed',now);
 await c.add(await importExchange(await exportExchange(f)));assert.equal(c.search()[0].state,'retracted');
});
test('unrelated revision and other source details do not supersede anything automatically',async()=>{
 const c=new LocalCatalog(),f=await fixture();const one=await c.add(f);f.manifest.package_version=2;f.manifest.supersedes=one.id;const two=await c.add(f);
 assert.equal(c.search().length,2);assert.ok(await c.attach(one.id,{at:now,allowEvidence:true}));assert.ok(await c.attach(two.id,{at:now,allowEvidence:true}));
});
test('explicit local evidence inclusion consent required every attach',async()=>{
 const c=new LocalCatalog(),{id}=await c.add(await fixture());await assert.rejects(c.attach(id,{at:now}),/approval/);await assert.rejects(c.attach(id,{at:now,allowEvidence:'true'}),/approval/);
});
for (const [label,modify,regex] of [
 ['tampered evidence',f=>{const p=Object.keys(f.evidence)[0];f.evidence[p]+=' changed';},/hash/],
 ['source rights absent',f=>{f.data.sources[0].redistributable=false;},/permission/],
 ['source restricted',f=>{f.data.sources[0].restricted=true;},/constant|Unavailable/],
 ['source withdrawn',f=>{f.data.sources[0].withdrawn=true;},/constant|Unavailable/],
 ['cross workspace',f=>{f.data.sources[0].workspace='other';},/workspace/],
 ['duplicate source',f=>f.data.sources.push(f.data.sources[0]),/Duplicate/],
 ['missing provenance',f=>{f.data.assertions[0].evidence[0].source='missing@1';},/provenance|Span|source|evidence/i],
 ['invalid byte span',f=>{f.data.assertions[0].evidence[0].end=999999;},/offset/],
 ['extra path',f=>{f.evidence['../escape']='x';},/path/],
 ['missing evidence',f=>{delete f.evidence[Object.keys(f.evidence)[0]];},/Missing evidence/],
 ['native v2 rejected',f=>{f.manifest.schema_version=2;},/constant/],
 ['DIALECTICA v3 rejected',f=>{f.manifest.schema_version=3;},/constant/],
 ['unresolved dependency',f=>{f.manifest.dependencies=[{id:'other',version:1,digest:'0'.repeat(64)}];},/dependencies/],
 ['unknown field',f=>{f.manifest.execute='evil';},/Unknown/],
 ['native archive masquerading as pack input',f=>{f.manifest.files={'capsule.json':'0'.repeat(64)};},/Pack inputs/],
 ['trusted method imported',f=>{f.data.methods[0].review_state='accepted';},/authority/],
]) test(label,async()=>{const f=await fixture();modify(f);await assert.rejects(inspectInputs(f),regex);});
test('expired and future capsule rejected at attachment, boundary exclusive',async()=>{
 for(const change of [f=>{f.manifest.valid_until=now;},f=>{f.manifest.created_at=now+1;},f=>{f.data.sources[0].valid.end=now;},f=>{f.data.assertions[0].available.end=now;}]){
  const f=await fixture();change(f);const c=new LocalCatalog(),{id}=await c.add(f);await assert.rejects(c.attach(id,{at:now,allowEvidence:true}),/current|expired/);
 }
});
test('references-only may be inspected but cannot attach as evidence',async()=>{
 const f=await fixture();f.manifest.evidence_mode='references_only';f.evidence={};const c=new LocalCatalog(),{id}=await c.add(f);await assert.rejects(c.attach(id,{at:now,allowEvidence:true}),/Evidence/);
});
test('foreign integrity digest, extra fields and noncanonical JSON rejected',async()=>{
 const txt=await exportExchange(await fixture());const x=JSON.parse(txt);x.content_digest='0'.repeat(64);await assert.rejects(importExchange(canonical(x)),/digest/);
 await assert.rejects(importExchange(txt+' '),/canonical/);x.extra=true;await assert.rejects(importExchange(canonical(x)),/fields/);
 await assert.rejects(importExchange('x'.repeat(2*1024*1024+1)),/2 MiB/);
});
test('malicious JSON shapes, duplicate keys, nonfinite numbers and deep nesting rejected',async()=>{
 assert.throws(()=>canonical(JSON.parse('{"__proto__":{}}')),/Unsafe/);assert.throws(()=>canonical(NaN),/integers/);assert.throws(()=>canonical('\ud800'),/Unicode/);
 let x={};for(let i=0;i<50;i++)x={x};assert.throws(()=>canonical(x),/Nesting/);
 const txt=await exportExchange(await fixture());await assert.rejects(importExchange(txt.replace('"format":','"format":"x","format":')),/canonical/);
});

import {mkdtemp,rm,writeFile,stat} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {readBounded,readCatalog,updateCatalog} from './store.mjs';
test('local restart preserves data, revocation and duplicate-import state',async()=>{
 const dir=await mkdtemp(join(tmpdir(),'tacitus-catalog-'));
 try {
  const txt=await exportExchange(await fixture());const {id,digest}=await updateCatalog(dir,api=>api.import(txt));
  await updateCatalog(dir,({catalog})=>catalog.transition(id,digest,'revoked','Local access revoked',now));
  await updateCatalog(dir,api=>api.import(txt));const reopened=await readCatalog(dir);
  assert.equal(reopened.search()[0].state,'revoked');await assert.rejects(reopened.attach(id,{at:now,allowEvidence:true}),/Inactive/);
 }finally{await rm(dir,{recursive:true,force:true});}
});
test('concurrent writer lock and invalid event chain fail closed',async()=>{
 const dir=await mkdtemp(join(tmpdir(),'tacitus-catalog-'));
 try {
  await writeFile(join(dir,'.catalog.lock'),'');await assert.rejects(updateCatalog(dir,()=>null),/EEXIST/);await rm(join(dir,'.catalog.lock'));
  await writeFile(join(dir,'catalog.json'),JSON.stringify({format:'tacitus-local-catalog-store/1',exchanges:[],events:[{id:'missing'}]}));
  await assert.rejects(readCatalog(dir),/pinned/);
 }finally{await rm(dir,{recursive:true,force:true});}
});
test('cognitive methods retain scope, failures and provenance without execution',async()=>{
 const f=await fixture();f.data.assertions=[];f.manifest.profile='cognitive_method';f.manifest.id='conditional-method';const c=new LocalCatalog(),{id}=await c.add(f);
 assert.equal((await c.attach(id,{at:now,allowEvidence:true})).profile,'cognitive_method');
 assert.deepEqual((await importExchange(await c.export(id))).data.methods,f.data.methods);
});
test('empty capsules and invalid identifiers fail closed',async()=>{
 const f=await fixture();f.data.assertions=[];f.data.methods=[];const c=new LocalCatalog(),{id}=await c.add(f);await assert.rejects(c.attach(id,{at:now,allowEvidence:true}),/Empty/);
 f.manifest.id='../../escape';await assert.rejects(inspectInputs(f),/identity/);
});

test('revocation during asynchronous validation cancels attachment and export',async()=>{
 for(const action of ['attach','export']){
  const c=new LocalCatalog(),{id,digest}=await c.add(await fixture());
  const pending=action==='attach'?c.attach(id,{at:now,allowEvidence:true}):c.export(id);
  c.transition(id,digest,'revoked','Access withdrawn during validation',now);
  await assert.rejects(pending,/Inactive|revoked|changed/i);
 }
});
test('input snapshots keep hashes and content consistent during asynchronous work',async()=>{
 const f=await fixture(),original=structuredClone(f),expected=(await inspectInputs(original)).digest;
 const c=new LocalCatalog(),pending=c.add(f);f.manifest.title='Caller mutation while add awaits hashing';
 const {id,digest}=await pending;assert.equal(digest,expected);
 assert.deepEqual(await importExchange(await c.export(id)),original);
 const g=await fixture(),initial=structuredClone(g),exporting=exportExchange(g);
 g.manifest.title='Caller mutation while export awaits hashing';
 assert.deepEqual(await importExchange(await exporting),initial);
});
test('canonical JSON rejects malformed Unicode object keys and sparse arrays',async()=>{
 assert.throws(()=>canonical({'\ud800':'value'}),/Unicode/);
 assert.throws(()=>canonical(new Array(2)),/array|JSON|sparse/i);
 const f=await fixture();f.data.artifacts['\ud800']='value';
 await assert.rejects(inspectInputs(f),/Unicode/);
});
for(const [label,modify] of [
 ['empty native manifest purpose',f=>{f.manifest.purpose='';}],
 ['empty native manifest domain',f=>{f.manifest.domain='';}],
 ['empty method identity',f=>{f.data.methods[0].id='';}],
 ['empty method scope',f=>{f.data.methods[0].scope='';}],
 ['blank method description',f=>{f.data.methods[0].description=' ';}],
 ['missing method failure conditions',f=>{f.data.methods[0].failure_conditions=[];}],
 ['blank method failure condition',f=>{f.data.methods[0].failure_conditions=[' '];}],
 ['package version outside native u32',f=>{f.manifest.package_version=2**32;}],
 ['assertion version outside native u32',f=>{f.data.assertions[0].version=2**32;}],
 ['method version outside native u32',f=>{f.data.methods[0].version=2**32;}],
])test(`native semantic validation: ${label}`,async()=>{
 const f=await fixture();modify(f);await assert.rejects(inspectInputs(f));
});
for(const [label,modify] of [
 ['backward source interval',f=>{f.data.sources[0].valid.end=f.data.sources[0].valid.start;}],
 ['backward assertion interval',f=>{f.data.assertions[0].valid.end=f.data.assertions[0].valid.start;}],
 ['assertion without evidence',f=>{f.data.assertions[0].evidence=[];}],
 ['blank assertion proposition',f=>{f.data.assertions[0].proposition=' ';}],
 ['empty assertion speaker',f=>{f.data.assertions[0].speaker='';}],
 ['empty assertion rationale',f=>{f.data.assertions[0].confidence_rationale='';}],
 ['reversed evidence span',f=>{f.data.assertions[0].evidence[0].start=1000;}],
 ['mismatched evidence language',f=>{f.data.assertions[0].evidence[0].original_language='different';}],
])test(`references-only semantic validation: ${label}`,async()=>{
 const f=await fixture();f.manifest.evidence_mode='references_only';f.evidence={};modify(f);
 await assert.rejects(inspectInputs(f));
});
test('new persistent stores restrict directory and file access',async()=>{
 const base=await mkdtemp(join(tmpdir(),'tacitus-catalog-private-')),dir=join(base,'new-store');
 try{
  const txt=await exportExchange(await fixture());await updateCatalog(dir,api=>api.import(txt));
  assert.equal((await stat(dir)).mode&0o077,0);
  assert.equal((await stat(join(dir,'catalog.json'))).mode&0o077,0);
 }finally{await rm(base,{recursive:true,force:true});}
});
test('complete exchange envelope respects the same byte bound on export and import',async()=>{
 const f=await fixture();f.data.artifacts.padding='';
 const envelopeBytes=Buffer.byteLength(await exportExchange(f))-Buffer.byteLength(canonical(f));
 f.data.artifacts.padding='x'.repeat(2*1024*1024-envelopeBytes-Buffer.byteLength(canonical(f)));
 const txt=await exportExchange(f);assert.equal(Buffer.byteLength(txt),2*1024*1024);
 assert.deepEqual(await importExchange(txt),f);
 f.data.artifacts.padding+='x';await assert.rejects(exportExchange(f),/2 MiB|bound|large|size/i);
});
test('bounded local reads reject malformed UTF-8 without silently replacing bytes',async()=>{
 const dir=await mkdtemp(join(tmpdir(),'tacitus-catalog-utf8-'));
 try{
  const path=join(dir,'input.json');await writeFile(path,Buffer.from([0x7b,0x22,0x78,0x22,0x3a,0x22,0xff,0x22,0x7d]));
  await assert.rejects(readBounded(path),/UTF|encoded|encoding/i);
 }finally{await rm(dir,{recursive:true,force:true});}
});
test('BOM is preserved so exchange parser rejects noncanonical bytes',async()=>{
 const dir=await mkdtemp(join(tmpdir(),'tacitus-bom-'));
 try{const file=join(dir,'bom.json');await writeFile(file,'\ufeff'+await exportExchange(await fixture()));const {readBounded}=await import('./store.mjs');const text=await readBounded(file);assert.equal(text.charCodeAt(0),0xfeff);await assert.rejects(importExchange(text));}
 finally{await rm(dir,{recursive:true,force:true});}
});
test('assertion dependencies fail closed until closure resolution is implemented',async()=>{
 const f=await fixture();f.data.assertions[0].dependencies=['missing-assertion'];
 await assert.rejects(inspectInputs(f),/Assertion dependencies/);await assert.rejects(new LocalCatalog().add(f),/Assertion dependencies/);
 f.data.assertions[0].dependencies=[f.data.assertions[1].id];await assert.rejects(exportExchange(f),/Assertion dependencies/);
});
