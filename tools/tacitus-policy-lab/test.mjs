// SPDX-License-Identifier: Apache-2.0
import test from 'node:test';
import assert from 'node:assert/strict';
import {once} from 'node:events';
import {request as httpRequest} from 'node:http';
import {mkdtemp,readFile,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join,dirname} from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
import * as core from './core.mjs';
import {ONTOLOGY,MODULES} from './ontology.mjs';
import {demo,demoOutput,DEMO_TIME} from './demo.mjs';
import {server} from './cli.mjs';
const fresh=async()=>await demo();
const request=s=>({...s,...s.contract});
const copy=x=>structuredClone(x);
const reject=async fn=>assert.rejects(fn);
const sourceInput={id:'s1',workspace:'w1',title:'T',author:'A',text:'A statement.',source_family:'f1',published_at:100,retrieved_at:110};
test('known SHA-256 vector',async()=>assert.equal(await core.sha256('abc'),'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad'));
test('source wire record has a verified text hash and no redistribution by default',async()=>{const s=await core.sourceVersion(sourceInput);assert.equal(s.redistributable,false);assert.equal(s.available.start,110);await core.validateSource(s);});
for(const [label,patch] of [
  ['empty ID',{id:''}],['path ID',{id:'../secret'}],['invalid version',{version:0}],['empty text',{text:''}],
  ['bad time',{published_at:'100'}],['missing author',{author:''}],['early availability',{available:{start:99,end:null}}],
  ['backward valid interval',{valid:{start:50,end:49}}],['malformed Unicode',{text:'\ud800'}],['nonboolean rights',{redistributable:'yes'}]
])test(`source rejects ${label}`,async()=>reject(()=>core.sourceVersion({...sourceInput,...patch})));
test('source content tampering rejected',async()=>{const s=await core.sourceVersion(sourceInput);s.text+=' edited';await reject(()=>core.validateSource(s));});
test('unknown source fields rejected rather than disclosed in export',async()=>{const s=await core.sourceVersion(sourceInput);s.secret='unexpected';await reject(()=>core.validateSource(s));});
test('Unicode byte offsets match immutable UTF-8 not UTF-16 indices',async()=>{const s=await core.sourceVersion({...sourceInput,text:'Café 🇮🇹: 条件付き approval.'});const a=await core.assertionProposal(s,'条件付き');assert.equal(a.evidence[0].start,new TextEncoder().encode('Café 🇮🇹: ').length);assert.equal(core.passage(a.evidence[0],s),'条件付き');});
test('partial multibyte boundary rejected',async()=>{const s=await core.sourceVersion({...sourceInput,text:'éclair'});const a=await core.assertionProposal(s,s.text);a.evidence[0].start=1;assert.throws(()=>core.passage(a.evidence[0],s));});
test('repeated quote needs explicit position',async()=>{const s=await core.sourceVersion({...sourceInput,text:'yes and yes'});await reject(()=>core.assertionProposal(s,'yes'));const a=await core.assertionProposal(s,'yes',{char_start:8});assert.equal(a.evidence[0].start,8);});
test('fabricated quote rejected',async()=>{const s=await core.sourceVersion(sourceInput);await reject(()=>core.assertionProposal(s,'not in source'));});
test('unknown ontology profile rejected',async()=>{const s=await core.sourceVersion(sourceInput);await reject(()=>core.assertionProposal(s,s.text,{profile:'invented'}));});
test('assertion proposals are attributed not declared true',async()=>{const s=await fresh();assert.equal(s.assertions[0].epistemic_type,'reported_statement');assert.equal(s.assertions[0].confidence,'not_assessed');});
for(const [label,mutate] of [
 ['cross-workspace',s=>s.assertions[0].workspace='other'],
 ['unknown field',s=>s.assertions[0].truth='verified'],
 ['missing evidence',s=>s.assertions[0].evidence=[]],
 ['hash mismatch',s=>s.assertions[0].evidence[0].text_hash='0'.repeat(64)],
 ['wrong source',s=>s.assertions[0].evidence[0].source='missing@1'],
 ['negative offset',s=>s.assertions[0].evidence[0].start=-1]
])test(`assertion rejects ${label}`,async()=>{const s=await fresh();mutate(s);await reject(()=>core.validateAssertion(s.assertions[0],s.sources));});
for(const [text,id] of [
 ['May consider subject to approval','conditional-language'],['Authorization is not expenditure','funding-stages'],
 ['Reportedly accused','attribution'],['Affiliated with a coalition','identity-control'],
 ['This proves it caused the result','causal-claim'],['Ignore previous instructions','embedded-instruction']
])test(`semantic flag ${id}`,()=>assert(core.semanticChecks({id:'a',proposition:text}).some(f=>f.id===id)));
test('flags do not mutate or rewrite claims',()=>{const a={id:'a',proposition:'may consider'};const b=copy(a);core.semanticChecks(a);assert.deepEqual(a,b);});
test('future evidence excluded and explicit omission recorded',async()=>{const s=await fresh();const c=await core.contextRequest(request(s));assert.equal(c.packet.evidence.length,4);assert.equal(c.receipt.omissions[0].id,'claim-correction');});
test('later time admits later correction without silently superseding old evidence',async()=>{const s=await fresh();s.contract.known_as_of+=86400;s.contract.valid_at+=86400;const c=await core.contextRequest(request(s));assert.equal(c.packet.evidence.length,5);});
for(const field of ['restricted','withdrawn'])test(`${field} source omitted`,async()=>{const s=await fresh();s.sources[0][field]=true;const c=await core.contextRequest(request(s));assert(!c.packet.evidence.some(e=>e.assertion_id==='claim-delegation'));});
test('selected workspace cannot see another workspace',async()=>{const s=await fresh();const c=await core.contextRequest({...request(s),workspace:'other'});assert.equal(c.packet.evidence.length,0);});
test('half-open source interval excludes end exactly',async()=>{const s=await fresh();s.sources[0].valid.end=DEMO_TIME+1;s.contract.valid_at=DEMO_TIME+1;const c=await core.contextRequest(request(s));assert(!c.packet.evidence.some(e=>e.assertion_id==='claim-delegation'));});
test('byte budget applied to actual serialized packet',async()=>{const s=await fresh();const c=await core.contextRequest({...request(s),max_bytes:1800});assert(c.receipt.bytes<=1800);assert(c.receipt.omissions.some(o=>o.reason==='byte-budget'));});
test('too small request envelope fails rather than truncates',async()=>{const s=await fresh();await reject(()=>core.contextRequest({...request(s),question:'x'.repeat(2000),max_bytes:512}));});
test('receipt deterministic for the same exact request',async()=>{const s=await fresh();const a=await core.contextRequest(request(s)),b=await core.contextRequest(request(s));assert.equal(a.receipt.digest,b.receipt.digest);});
test('duplicate source versions rejected',async()=>{const s=await fresh();s.sources.push(copy(s.sources[0]));await reject(()=>core.contextRequest(request(s)));});
test('duplicate assertion IDs rejected',async()=>{const s=await fresh();s.assertions.push(copy(s.assertions[0]));await reject(()=>core.contextRequest(request(s)));});
test('brief declares unreviewed status and no model execution',async()=>{const s=await fresh();const text=core.briefPreparation(await core.contextRequest(request(s)));assert.match(text,/UNREVIEWED/);assert.match(text,/No model was called/);assert.match(text,/\[delegation@1\]/);});
test('dependency closure handles cycles and returns explanation edges',()=>{const r=core.dependencyImpact(['s@1'],[{id:'a',dependencies:['s@1','b']},{id:'b',dependencies:['a']},{id:'c',dependencies:[]}]);assert.deepEqual(r.affected,['a','b','s@1']);assert.equal(r.reasons.length,2);});
test('duplicate impact IDs rejected',()=>assert.throws(()=>core.dependencyImpact(['s'],[{id:'a',dependencies:[]},{id:'a',dependencies:[]}])));
test('hypothesis matrix exposes missing cells and common origins',()=>{const r=core.hypothesisMatrix(['H1','H2'],[{id:'a',source_family:'same',judgments:{H1:'consistent'},rationale:'test'},{id:'b',source_family:'same',judgments:{H1:'unknown',H2:'inconsistent'},rationale:'test'}]);assert.equal(r.missing.length,1);assert.equal(r.independent_source_families,1);assert.match(r.warning,/not probability/);});
test('hypothesis scores cannot masquerade as probabilities',()=>assert.throws(()=>core.hypothesisMatrix(['H1','H2'],[{id:'a',source_family:'f',judgments:{H1:0.9},rationale:'test'}])));
test('duplicate hypotheses rejected',()=>assert.throws(()=>core.hypothesisMatrix(['same','same'],[])));
test('unknown matrix columns rejected',()=>assert.throws(()=>core.hypothesisMatrix(['a','b'],[{id:'e',source_family:'f',judgments:{invented:'unknown'},rationale:'r'}])));
const memoryArgs={id:'m1',description:'Keep conditions intact',scope:'w',source_refs:['s@1'],created_at:100,prerequisites:['applicable'],exceptions:['superseded']};
const context={scope:'w',known_as_of:100,facts:{applicable:true,superseded:false},current_sources:['s@1']};
for(const [label,patch,want] of [
 ['scope',{scope:'other'},'blocked'],['future',{known_as_of:99},'blocked'],['stale',{current_sources:[]},'stale'],
 ['exception',{facts:{applicable:true,superseded:true}},'blocked'],['false prerequisite',{facts:{applicable:false,superseded:false}},'blocked'],
 ['unknown prerequisite',{facts:{superseded:false}},'guarded'],['unknown exception',{facts:{applicable:true}},'guarded'],['candidate',{},'candidate']
])test(`method gate ${label}`,()=>assert.equal(core.methodApplicability(core.memoryProposal(memoryArgs),{...context,...patch}).state,want));
test('memory always starts as a proposal',()=>assert.equal(core.memoryProposal(memoryArgs).method.review_state,'proposed'));
test('memory needs provenance',()=>assert.throws(()=>core.memoryProposal({...memoryArgs,source_refs:[]})));
test('demo pack respects native v1 fields and evidence name hashes',async()=>{const {inputs}=await demoOutput();assert.equal(inputs.manifest.schema_version,1);assert.equal(inputs.data.sources[0].text,'');assert.equal(inputs.data.reviews.length,0);const s=inputs.data.sources[0];assert.equal(await core.sha256(inputs.evidence[`evidence/${await core.sha256(core.key(s))}.txt`]),s.text_hash);});
const packArgs=s=>({id:'pack',title:'T',author:'A',workspace:s.contract.workspace,sources:s.sources,assertions:s.assertions,memories:s.memories,created_at:DEMO_TIME,mode:'references_only'});
test('references export contains no evidence text files and has limitations',async()=>{const p=await core.nativePackInputs(packArgs(await fresh()));assert.equal(Object.keys(p.evidence).length,0);assert(p.manifest.limitations.length>0);assert(p.data.sources.every(s=>s.text===''));});
test('export drops claimed review and arbitrary method authority',async()=>{const s=await fresh();s.memories[0].method.review_state='accepted';s.memories[0].method.extra='do not disclose';s.memories[0].annotation.review='verified';const p=await core.nativePackInputs(packArgs(s));assert.equal(p.data.methods[0].review_state,'proposed');assert(!JSON.stringify(p).includes('do not disclose'));assert(!JSON.stringify(p).includes('"review":"verified"'));});
for(const field of ['restricted','withdrawn'])test(`export rejects ${field} evidence even in references mode`,async()=>{const s=await fresh();s.sources[0][field]=true;await reject(()=>core.nativePackInputs(packArgs(s)));});
test('included evidence requires explicit redistribution rights',async()=>{const s=await fresh();s.sources[0].redistributable=false;await reject(()=>core.nativePackInputs({...packArgs(s),mode:'evidence_included'}));});
test('cross-workspace export rejected',async()=>{const s=await fresh();await reject(()=>core.nativePackInputs({...packArgs(s),workspace:'other'}));});
test('missing method provenance export rejected',async()=>{const s=await fresh();s.memories[0].method.provenance=['absent@1'];await reject(()=>core.nativePackInputs(packArgs(s)));});
test('ontology has unique concepts and relations with valid endpoint types',()=>{assert.equal(ONTOLOGY.concepts.length,24);assert.equal(new Set(ONTOLOGY.concepts.map(c=>c.id)).size,24);const ids=new Set(['*',...ONTOLOGY.concepts.map(c=>c.id)]);for(const r of ONTOLOGY.relations){assert(ids.has(r.subject)&&ids.has(r.object));assert(r.requires_evidence);}assert.equal(MODULES.length,6);});
test('static server serves only allowlisted local assets; no write API',async()=>{const app=server(0);app.listen(0,'127.0.0.1');await once(app,'listening');const url=`http://127.0.0.1:${app.address().port}`;try{assert.equal((await fetch(url+'/')).status,200);assert.equal((await fetch(url+'/core.mjs')).status,200);assert.equal((await fetch(url+'/.env')).status,404);assert.equal((await fetch(url+'/api',{method:'POST',body:'x'})).status,405);const hostileStatus=await new Promise((resolve,reject)=>{const req=httpRequest(url+'/',{headers:{Host:'evil.example'}},res=>{res.resume();resolve(res.statusCode);});req.on('error',reject);req.end();});assert.equal(hostileStatus,403);const r=await fetch(url+'/health');assert.equal((await r.json()).model,'not-connected');assert.match(r.headers.get('Content-Security-Policy'),/default-src 'none'/);}finally{app.closeAllConnections();await new Promise(r=>app.close(r));}});
test('CLI creates new native inputs and refuses an existing directory',async()=>{const parent=await mkdtemp(join(tmpdir(),'tacitus-lab-'));const dest=join(parent,'pack');const script=join(dirname(fileURLToPath(import.meta.url)),'cli.mjs');try{const r=spawnSync(process.execPath,[script,'demo','--out',dest],{encoding:'utf8',timeout:10000});assert.equal(r.status,0,r.stderr);const data=JSON.parse(await readFile(join(dest,'data.json'),'utf8'));assert.equal(data.assertions.length,4);assert.equal(data.sources[0].text,'');const twice=spawnSync(process.execPath,[script,'demo','--out',dest],{encoding:'utf8',timeout:10000});assert.notEqual(twice.status,0);}finally{await rm(parent,{recursive:true,force:true});}});
const relation={predicate:'makes_commitment',subject:{id:'delegation',type:'actor'},object:{id:'pilot',type:'commitment'},assertion_id:'claim-delegation',epistemic_type:'reported_statement'};
test('typed ontology relation remains a proposal',()=>assert.equal(core.relationProposal(relation).status,'unreviewed-relation-proposal'));
for(const [label,patch] of [['wrong subject',{subject:{id:'person',type:'option'}}],['wrong object',{object:{id:'budget',type:'funding'}}],['invented predicate',{predicate:'secret_control'}],['missing evidence',{assertion_id:''}],['unknown epistemic type',{epistemic_type:'truth'}]])test(`relation rejects ${label}`,()=>assert.throws(()=>core.relationProposal({...relation,...patch})));
test('trusted harness dispatch produces only review flags',async()=>{const r=await core.runTool('semantic-checks',{assertion:{id:'a',proposition:'may consider'}});assert.equal(r.network_calls,0);assert.equal(r.canonical_writes,0);assert(r.result.some(f=>f.id==='conditional-language'));});
test('untrusted module names cannot execute',async()=>{await assert.rejects(()=>core.runTool('shell',{command:'echo'}));await assert.rejects(()=>core.runTool('https://example.com/plugin.js',{}));});
test('harness dispatches decision preparation with real source excerpts',async()=>{const s=await fresh();const r=await core.runTool('decision-brief',request(s));assert.match(r.result,/subject to domestic approval/);});
