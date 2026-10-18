// SPDX-License-Identifier: Apache-2.0
import {readFile} from 'node:fs/promises';
import {sha256, key, validateSource, validateAssertion, contains} from '../tacitus-policy-lab/core.mjs';
import {validateSchema} from './schema.mjs';
const schemaText = await readFile(new URL('../../schemas/tacitus/capsule-v1.schema.json',import.meta.url),'utf8');
if (await sha256(schemaText) !== '003d06f1591f43b3b2d78cfb993732213860c1f7930fd2acc64ef36001dc1dbf') throw new Error('Native v1 schema changed: review adapter before use');
const schema = JSON.parse(schemaText);
export const FORMAT = 'tacitus-local-catalog-exchange/1';
const MAX = 2 * 1024 * 1024;
const assert = (condition,message) => { if (!condition) throw new Error(message); };
const clone = v => structuredClone(v);
const exact = (v,keys) => assert(v && typeof v==='object' && !Array.isArray(v) && Object.keys(v).length===keys.length && keys.every(k=>Object.hasOwn(v,k)),'Invalid exchange fields');
export function canonical(value, depth=0) {
  assert(depth < 48,'Nesting limit exceeded');
  if (value === null || typeof value === 'boolean') return JSON.stringify(value);
  if (typeof value === 'number') { assert(Number.isSafeInteger(value),'Only safe integers supported');return JSON.stringify(value); }
  if (typeof value === 'string') { assert(value.isWellFormed(),'Invalid Unicode');return JSON.stringify(value); }
  if (Array.isArray(value)) {
    assert(Object.keys(value).length===value.length && Array.from({length:value.length},(_,i)=>Object.hasOwn(value,i)).every(Boolean),'Sparse or decorated arrays unsupported');
    return '['+value.map(v=>canonical(v,depth+1)).join(',')+']';
  }
  assert(value && Object.getPrototypeOf(value)===Object.prototype,'Plain JSON data required');
  assert(Object.keys(value).every(k=>k.isWellFormed()),'Invalid Unicode object key');
  assert(!Object.keys(value).some(k=>['__proto__','constructor','prototype'].includes(k)),'Unsafe object key');
  return '{'+Object.keys(value).sort().map(k=>JSON.stringify(k)+':'+canonical(value[k],depth+1)).join(',')+'}';
}
export async function inspectInputs(inputs) {
  inputs=clone(inputs);
  exact(inputs,['manifest','data','evidence']);
  const encoded=canonical(inputs); assert(Buffer.byteLength(encoded)<=MAX,'Exchange exceeds 2 MiB');
  validateSchema({manifest:inputs.manifest,data:inputs.data},schema);
  const {manifest:m,data:d,evidence}=inputs;
  assert(m.schema_version===1,'Native v1 only; native v2 and DIALECTICA v3 need explicit adapters');
  assert(/^[a-zA-Z0-9][a-zA-Z0-9._:-]{0,100}$/.test(m.id) && [m.title,m.author,m.scope,m.license,m.sharing_policy,m.domain,m.purpose].every(x=>typeof x==='string' && x.trim()),'Missing or invalid identity or rights');
  assert(m.package_version<=4294967295,'Version exceeds native u32');
  assert([...d.sources,...d.assertions,...d.methods].every(x=>x.version<=4294967295),'Version exceeds native u32');
  const nonblank=x=>typeof x==='string' && x.trim().length>0;
  const interval=v=>assert(v.end===null || v.end>v.start,'Invalid half-open interval');
  for(const s of d.sources){interval(s.available);interval(s.valid);assert([s.id,s.title,s.author,s.language,s.extraction_tool,s.source_family,s.license].every(nonblank),'Incomplete source metadata');assert(s.available.start>=Math.max(s.published_at,s.retrieved_at),'Knowledge precedes publication/retrieval');}
  for(const a of d.assertions){interval(a.available);interval(a.valid);assert([a.id,a.proposition,a.speaker,a.confidence_rationale].every(nonblank)&&a.evidence.length>0,'Incomplete assertion evidence');}
  for(const method of d.methods)assert([method.id,method.scope,method.description].every(nonblank)&&method.failure_conditions.length>0&&method.failure_conditions.every(nonblank),'Invalid cognitive method');
  assert(m.valid_until===null || m.valid_until>m.created_at,'Invalid capsule validity interval');
  assert(Object.keys(m.files).length===0,'Pack inputs only; native archive file hashes require Rust validation');
  assert(m.dependencies.length===0,'Unresolved dependencies cannot be attached');
  assert(d.assertions.every(a=>a.dependencies.length===0),'Assertion dependencies require a verified closure adapter');
  assert(d.reviews.length===0 && d.methods.every(x=>x.review_state==='proposed'),'Imported reviews cannot grant authority');
  assert(new Set(d.sources.map(key)).size===d.sources.length,'Duplicate source version');
  assert(new Set(d.assertions.map(x=>x.id)).size===d.assertions.length,'Duplicate assertion');
  assert(new Set(d.methods.map(x=>x.id)).size===d.methods.length,'Duplicate method');
  assert(evidence && !Array.isArray(evidence) && typeof evidence==='object','Invalid evidence');
  const sources=[]; const expected=[];
  for (const source of d.sources) {
    assert(source.workspace===m.scope && !source.withdrawn && !source.restricted,'Unavailable or cross-workspace source');
    const path=`evidence/${await sha256(key(source))}.txt`;
    if (m.evidence_mode==='evidence_included') {
      assert(source.redistributable && source.license?.trim(),'No evidence redistribution permission');
      assert(Object.hasOwn(evidence,path),'Missing evidence file'); expected.push(path);
      const restored={...source,text:evidence[path]};await validateSource(restored);sources.push(restored);
    }
  }
  assert(Object.keys(evidence).length===expected.length && Object.keys(evidence).every(p=>expected.includes(p)),'Unexpected evidence path');
  if (m.evidence_mode==='references_only') assert(m.limitations.length>0,'References-only limitations required');
  if (sources.length) for (const a of d.assertions) await validateAssertion(a,sources);
  for (const a of d.assertions) {
    assert(a.workspace===m.scope,'Cross-workspace assertion');
    for (const span of a.evidence) assert(span.start<span.end && d.sources.some(s=>key(s)===span.source && s.text_hash===span.text_hash && s.language===span.original_language),'Missing or invalid pinned provenance');
  }
  for (const method of d.methods) assert(method.provenance.length>0 && method.provenance.every(ref=>d.sources.some(s=>key(s)===ref)),'Missing method provenance');
  return {digest:await sha256(encoded),sources,manifest:clone(m)};
}
export async function exportExchange(inputs) {
  inputs=clone(inputs);
  const {digest}=await inspectInputs(inputs);
  const text=canonical({format:FORMAT,content_digest:digest,inputs});
  assert(Buffer.byteLength(text)<=MAX,'Exchange envelope exceeds 2 MiB');return text;
}
export async function importExchange(text) {
  assert(typeof text==='string' && Buffer.byteLength(text)<=MAX,'Exchange exceeds 2 MiB');
  const x=JSON.parse(text);exact(x,['format','content_digest','inputs']);assert(x.format===FORMAT,'Unknown exchange format');
  const result=await inspectInputs(x.inputs);assert(result.digest===x.content_digest,'Content digest mismatch');
  // Canonical serialization rejects duplicate keys, whitespace variants, and ambiguous encodings.
  assert(canonical(x)===text,'Use canonical exchange serialization');
  return clone(x.inputs);
}
export class LocalCatalog {
  #records=new Map(); #events=[];
  async add(inputs) {
    inputs=clone(inputs);
    const checked=await inspectInputs(inputs); const m=checked.manifest;const id=`${m.id}@${m.package_version}`;
    const prior=this.#records.get(id);assert(!prior || prior.digest===checked.digest,'Version collision: immutable revision has different content');
    if (!prior) this.#records.set(id,{inputs:clone(inputs),digest:checked.digest,state:'active',generation:0});
    return {id,digest:checked.digest,duplicate:!!prior};
  }
  search(query='') {
    assert(typeof query==='string' && query.length<=500,'Invalid query');const needle=query.toLowerCase();
    return [...this.#records.entries()].filter(([,r])=>[r.inputs.manifest.title,r.inputs.manifest.domain,r.inputs.manifest.scope,r.inputs.manifest.profile].join(' ').toLowerCase().includes(needle))
      .map(([id,r])=>({id,content_digest:r.digest,title:r.inputs.manifest.title,profile:r.inputs.manifest.profile,state:r.state,review_status:'imported-unreviewed'})).sort((a,b)=>a.id.localeCompare(b.id));
  }
  transition(id,digest,state,reason,at) {
    const r=this.#records.get(id);assert(r && r.digest===digest,'Unknown pinned revision');
    assert(['active','retracted','revoked'].includes(state) && state!==r.state,'Invalid status transition');
    assert(typeof reason==='string' && reason.trim() && reason.length<=2000 && Number.isSafeInteger(at),'Explicit reason/time required');
    assert(!this.#events.length || at>=this.#events.at(-1).at,'Events must be chronological');
    const event={sequence:this.#events.length+1,id,content_digest:digest,from:r.state,to:state,reason,at};this.#events.push(event);r.state=state;r.generation++;return clone(event);
  }
  history() {return clone(this.#events);}
  async export(id) {const r=this.#records.get(id);assert(r?.state==='active','Inactive revision');const generation=r.generation;const text=await exportExchange(r.inputs);assert(r.state==='active' && r.generation===generation,'Revision changed during export');return text;}
  async attach(id,{at,allowEvidence=false}={}) {
    const r=this.#records.get(id);assert(r?.state==='active','Inactive revision');assert(Number.isSafeInteger(at),'Explicit attach time required');
    const generation=r.generation;const {manifest:m,sources}=await inspectInputs(r.inputs);
    assert(at>=m.created_at && (m.valid_until===null || at<m.valid_until),'Capsule not current');
    assert(m.evidence_mode==='evidence_included' && allowEvidence===true,'Evidence and explicit local inclusion approval required');
    assert(sources.length>0 && (r.inputs.data.assertions.length>0 || r.inputs.data.methods.length>0),'Empty capsule cannot attach');
    assert(sources.every(s=>contains(s.available,at)&&contains(s.valid,at)),'Source unavailable or expired');
    assert(r.inputs.data.assertions.every(a=>contains(a.available,at)&&contains(a.valid,at)),'Assertion unavailable or expired');
    assert(r.state==='active' && r.generation===generation,'Revision changed during attachment');
    return {format:'tacitus-praxis-attachment-proposal/1',capsule_id:m.id,package_version:m.package_version,content_digest:r.digest,profile:m.profile,title:m.title,
      review_status:'imported-unreviewed',trust:'untrusted-data',rights:{license:m.license,confidentiality:m.confidentiality,sharing_policy:m.sharing_policy},valid_until:m.valid_until,attached_at:at,source_refs:sources.map(key),limitations:clone(m.limitations),native_schema_version:1};
  }
}
