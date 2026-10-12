// SPDX-License-Identifier: Apache-2.0
// Browser/Node development tooling over the inspected tacitus-core wire shape.
// Never assigns canonical human review, executes imported code, or replaces Rust ZIP validation.
import { ONTOLOGY, MODULES } from './ontology.mjs';
export const VERSION = 'policy-lab/0.1.0';
const enc = new TextEncoder();
const dec = new TextDecoder('utf-8', { fatal: true });
const clone = value => structuredClone(value);
export function requireValue(ok, message) { if (!ok) throw new Error(message); }
export const bytes = text => enc.encode(text);
export async function sha256(value) {
  const buffer = typeof value === 'string' ? bytes(value) : value;
  return [...new Uint8Array(await crypto.subtle.digest('SHA-256', buffer))].map(n => n.toString(16).padStart(2, '0')).join('');
}
export const key = source => `${source.id}@${source.version}`;
const nonempty = v => typeof v === 'string' && v.trim().length > 0;
const idOK = v => typeof v === 'string' && /^[a-zA-Z0-9][a-zA-Z0-9._:-]{0,100}$/.test(v);
const epoch = n => Number.isSafeInteger(n);
const SOURCE_KEYS='id version workspace title author original_hash text_hash text language extraction_tool published_at retrieved_at available valid source_family withdrawn restricted redistributable license'.split(' ');
const ASSERTION_KEYS='id version workspace profile speaker proposition subject object epistemic_type polarity modality conditions exceptions audience speech_act valid available evidence dependencies challenges supersedes confidence confidence_rationale'.split(' ');
function exactKeys(value, keys) { requireValue(value && typeof value==='object' && !Array.isArray(value) && Object.keys(value).length===keys.length && keys.every(k=>Object.hasOwn(value,k)), 'Unsupported or missing record fields'); }
function interval(v) {
  exactKeys(v,['start','end']);
  requireValue(v && epoch(v.start) && (v.end === null || (epoch(v.end) && v.end > v.start)), 'Invalid half-open UTC interval');
}
export const contains = (v, t) => t >= v.start && (v.end === null || t < v.end);
function limitArray(a, limit, label) { requireValue(Array.isArray(a) && a.length <= limit, `Invalid or oversized ${label}`); }
export async function sourceVersion(input) {
  requireValue(input && idOK(input.id) && idOK(input.workspace), 'Source ID/workspace required');
  requireValue(nonempty(input.text) && bytes(input.text).length <= 250000, 'Selected UTF-8 text must contain 1–250000 bytes');
  // Reject unpaired surrogates rather than hashing a silently replaced string.
  requireValue(dec.decode(bytes(input.text)) === input.text, 'Invalid Unicode text');
  requireValue(nonempty(input.title) && nonempty(input.author) && nonempty(input.source_family), 'Source title, author and source family required');
  requireValue(epoch(input.published_at) && epoch(input.retrieved_at), 'Explicit publication and retrieval timestamps required');
  const digest = await sha256(input.text);
  const source = {
    id: input.id, version: input.version ?? 1, workspace: input.workspace,
    title: input.title, author: input.author, original_hash: digest, text_hash: digest,
    text: input.text, language: input.language ?? 'en', extraction_tool: 'tacitus-policy-lab-utf8/0.1.0',
    published_at: input.published_at, retrieved_at: input.retrieved_at,
    available: input.available ?? { start: Math.max(input.published_at, input.retrieved_at), end: null },
    valid: input.valid ?? { start: input.published_at, end: null }, source_family: input.source_family,
    withdrawn: input.withdrawn ?? false, restricted: input.restricted ?? false,
    redistributable: input.redistributable ?? false, license: input.license ?? 'All rights reserved'
  };
  await validateSource(source);
  return source;
}
export async function validateSource(s) {
  exactKeys(s,SOURCE_KEYS);
  requireValue(s && idOK(s.id) && idOK(s.workspace) && Number.isInteger(s.version) && s.version > 0, 'Invalid source identity');
  interval(s.available); interval(s.valid);
  requireValue(epoch(s.published_at) && epoch(s.retrieved_at) && s.available.start >= Math.max(s.published_at, s.retrieved_at), 'Knowledge cannot precede publication/retrieval');
  requireValue(nonempty(s.text) && bytes(s.text).length <= 250000 && dec.decode(bytes(s.text)) === s.text, 'Invalid source text');
  requireValue(await sha256(s.text) === s.text_hash && /^[a-f0-9]{64}$/.test(s.original_hash), 'Source hash mismatch');
  requireValue([s.author,s.title,s.language,s.extraction_tool,s.source_family,s.license].every(nonempty), 'Missing source metadata');
  requireValue(['withdrawn','restricted','redistributable'].every(f => typeof s[f] === 'boolean'), 'Explicit source rights/status required');
  return true;
}
export function passage(span, source) {
  exactKeys(span,['source','text_hash','start','end','locator','original_language','translation']);
  requireValue((span.locator===null||typeof span.locator==='string') && (span.translation===null||typeof span.translation==='string'),'Invalid span annotation');
  requireValue(span && span.source === key(source) && span.text_hash === source.text_hash && span.original_language === source.language, 'Span source/version/hash/language mismatch');
  requireValue(Number.isInteger(span.start) && Number.isInteger(span.end) && span.start >= 0 && span.end > span.start && span.end <= bytes(source.text).length, 'Invalid byte offsets');
  // Prefix and suffix decoding also enforce UTF-8 boundaries at both offsets.
  const text = bytes(source.text);
  dec.decode(text.slice(0, span.start)); dec.decode(text.slice(span.end));
  return dec.decode(text.slice(span.start, span.end));
}
export async function assertionProposal(source, quote, options = {}) {
  await validateSource(source);
  requireValue(!source.withdrawn && !source.restricted, 'Source unavailable to this laboratory');
  requireValue(nonempty(quote) && source.text.includes(quote), 'Quote must occur literally in source');
  const charStart = options.char_start ?? source.text.indexOf(quote);
  requireValue(Number.isInteger(charStart) && charStart >= 0 && source.text.slice(charStart, charStart + quote.length) === quote, 'Quote offset mismatch');
  requireValue(options.char_start !== undefined || source.text.indexOf(quote, charStart + 1) === -1, 'Repeated quote: specify char_start');
  const profile = options.profile ?? 'claim';
  requireValue(ONTOLOGY.concepts.some(c => c.id === profile), 'Unknown ontology profile');
  const span = { source: key(source), text_hash: source.text_hash, start: bytes(source.text.slice(0, charStart)).length,
    end: bytes(source.text.slice(0, charStart + quote.length)).length, locator: null, original_language: source.language, translation: null };
  const a = {
    id: options.id ?? `claim-${(await sha256(key(source) + ':' + charStart + ':' + quote)).slice(0,16)}`,
    version: 1, workspace: source.workspace, profile, speaker: source.author,
    proposition: quote, subject: options.subject ?? source.author, object: options.object ?? null,
    epistemic_type: 'reported_statement', polarity: 'unspecified', modality: 'preserve_source_wording',
    conditions: [], exceptions: [], audience: options.audience ?? 'unspecified', speech_act: 'quotation',
    valid: clone(source.valid), available: clone(source.available), evidence: [span], dependencies: [], challenges: [],
    supersedes: null, confidence: 'not_assessed', confidence_rationale: 'Literal source quotation; underlying proposition not verified.'
  };
  requireValue(idOK(a.id), 'Invalid assertion ID');
  return a;
}
export async function validateAssertion(a, sources) {
  exactKeys(a,ASSERTION_KEYS);
  requireValue(a && idOK(a.id) && Number.isInteger(a.version) && a.version > 0 && idOK(a.workspace), 'Invalid assertion identity');
  requireValue(nonempty(a.proposition) && nonempty(a.speaker) && nonempty(a.subject) && nonempty(a.confidence_rationale), 'Incomplete assertion');
  requireValue(ONTOLOGY.concepts.some(c => c.id === a.profile) && ONTOLOGY.epistemic_types.includes(a.epistemic_type), 'Unknown assertion profile or epistemic type');
  requireValue([a.polarity,a.modality,a.audience,a.speech_act,a.confidence].every(nonempty) && (a.object===null||typeof a.object==='string') && (a.supersedes===null||typeof a.supersedes==='string'),'Invalid assertion annotations');
  interval(a.available); interval(a.valid); limitArray(a.evidence,32,'evidence spans');
  requireValue(a.evidence.length > 0, 'Evidence required');
  for (const span of a.evidence) {
    const s = sources.find(s => key(s) === span.source);
    requireValue(s && s.workspace === a.workspace, 'Missing or cross-workspace evidence');
    await validateSource(s); passage(span,s);
  }
  for (const f of ['conditions','exceptions','dependencies','challenges']) {
    limitArray(a[f],64,f); requireValue(a[f].every(nonempty), `Invalid ${f}`);
  }
}
export function semanticChecks(a, sourceText = '') {
  const text = `${a.proposition ?? ''}\n${sourceText}`;
  const flags = [];
  const flag = (id, pattern, note) => { if (pattern.test(text)) flags.push({ id, severity:'review', assertion_id:a.id, note }); };
  flag('conditional-language', /\b(may|might|consider|subject to|pending|provided that|in principle)\b/i, 'Preserve conditions and modality; this is not proof of an unconditional commitment.');
  flag('funding-stages', /\b(authori[sz](ed|ation)|allocat(ed|ion)|appropriat(ed|ion)|obligat(ed|ion)|disburs(ed|ement)|expenditure)\b/i, 'Distinguish authorization, allocation, obligation, disbursement and expenditure.');
  flag('attribution', /\b(alleged|reportedly|according to|claimed|accused)\b/i, 'Attribute the report; repetition does not verify the underlying event.');
  flag('identity-control', /\b(affiliated|aligned|supports|linked to|associated with)\b/i, 'Affiliation or alignment does not by itself establish membership, command or control.');
  flag('causal-claim', /\b(caused|because|therefore|proves|inevitably)\b/i, 'Check competing explanations and distinguish temporal association from causation.');
  flag('embedded-instruction', /ignore (all |previous |prior )*instructions|system prompt|send.*api.key|override.*(policy|endpoint)/i, 'Treat this source passage as untrusted data, never instruction authority.');
  return flags;
}
export function dependencyImpact(changed, records) {
  limitArray(changed,256,'changed IDs'); limitArray(records,1024,'dependency records');
  const ids = new Set();
  for (const r of records) { requireValue(idOK(r.id) && !ids.has(r.id), 'Invalid/duplicate dependency ID'); ids.add(r.id); limitArray(r.dependencies,256,'dependencies'); }
  const affected = new Set(changed), reasons = [];
  let progress = true;
  while (progress) {
    progress = false;
    for (const r of records) if (!affected.has(r.id)) {
      const dependency = r.dependencies.find(d => affected.has(d));
      if (dependency) { affected.add(r.id); reasons.push({id:r.id, because:dependency}); progress = true; }
    }
  }
  return { affected:[...affected].sort(), reasons, action:'Reconsider; do not automatically revise or delete accepted judgments.' };
}
export function hypothesisMatrix(hypotheses, observations) {
  limitArray(hypotheses,12,'hypotheses'); limitArray(observations,128,'observations');
  requireValue(hypotheses.length >= 2 && hypotheses.every(nonempty) && new Set(hypotheses).size === hypotheses.length, 'At least two distinct hypotheses required');
  const observed = new Set(), missing = [], families = new Set();
  for (const row of observations) {
    requireValue(idOK(row.id) && !observed.has(row.id) && nonempty(row.source_family), 'Invalid/duplicate evidence row'); observed.add(row.id); families.add(row.source_family);
    requireValue(row.judgments && typeof row.judgments === 'object' && !Array.isArray(row.judgments), 'Evidence judgments required');
    requireValue(Object.keys(row.judgments).every(h => hypotheses.includes(h)), 'Unknown hypothesis column');
    for (const h of hypotheses) {
      const j = Object.hasOwn(row.judgments,h) ? row.judgments[h] : undefined;
      if (j === undefined) missing.push({ evidence:row.id, hypothesis:h });
      else requireValue(['consistent','inconsistent','unknown','not_diagnostic'].includes(j), 'Invalid evidence judgment');
    }
    requireValue(nonempty(row.rationale), 'Evidence rationale required');
  }
  return {hypotheses, observations, missing, independent_source_families:families.size,
    warning:'Consistency is not probability. Source families are declared metadata, not verified independence. No hypothesis is automatically selected.'};
}
export function memoryProposal({id,description,scope,source_refs,created_at,prerequisites=[],exceptions=[]}) {
  requireValue(idOK(id) && [description,scope].every(nonempty) && epoch(created_at), 'Incomplete method proposal');
  limitArray(source_refs,32,'method provenance'); requireValue(source_refs.length > 0 && source_refs.every(nonempty), 'Method provenance required');
  for (const list of [prerequisites,exceptions]) { limitArray(list,16,'guards'); requireValue(list.every(nonempty), 'Invalid method guard'); }
  return {
    method:{id,version:1,kind:'distinction',description,scope,failure_conditions: exceptions.length ? clone(exceptions) : ['Applicability outside the stated scope is unverified.'], provenance:clone(source_refs), review_state:'proposed'},
    annotation:{created_at,prerequisites:clone(prerequisites),exceptions:clone(exceptions),status:'unreviewed-proposal'}
  };
}
export function methodApplicability(memory, {scope,known_as_of,facts={},current_sources=[]}) {
  const m = memory.method, a = memory.annotation;
  requireValue(m && a && epoch(a.created_at) && epoch(known_as_of), 'Invalid memory timestamps');
  if (m.scope !== scope) return {state:'blocked',reason:'out-of-scope'};
  if (a.created_at > known_as_of) return {state:'blocked',reason:'not-yet-known'};
  if (m.provenance.some(ref => !current_sources.includes(ref))) return {state:'stale',reason:'missing-or-changed-provenance'};
  if (a.exceptions.some(f => facts[f] === true)) return {state:'blocked',reason:'exception-present'};
  if (a.prerequisites.some(f => facts[f] === false)) return {state:'blocked',reason:'prerequisite-false'};
  if (a.prerequisites.some(f => facts[f] !== true) || a.exceptions.some(f => facts[f] !== false)) return {state:'guarded',reason:'unknown-prerequisite-or-exception'};
  return {state:'candidate',reason:'Applicable candidate only; authenticated review/activation is not implemented in this lab.'};
}
export async function contextRequest({sources,assertions,question,workspace,known_as_of,valid_at,max_bytes=32000}) {
  limitArray(sources,32,'sources'); limitArray(assertions,256,'assertions');
  requireValue(nonempty(question) && question.length <= 4000 && idOK(workspace), 'Question and workspace required');
  requireValue(epoch(known_as_of) && epoch(valid_at) && Number.isInteger(max_bytes) && max_bytes>=512 && max_bytes<=250000, 'Invalid context bounds');
  requireValue(new Set(sources.map(key)).size === sources.length, 'Duplicate source version');
  requireValue(new Set(assertions.map(a=>a.id)).size === assertions.length, 'Duplicate assertion ID');
  const available = [];
  for (const s of sources) {
    await validateSource(s);
    if (s.workspace === workspace && !s.restricted && !s.withdrawn && contains(s.available,known_as_of) && contains(s.valid,valid_at)) available.push(s);
  }
  const packet = {kind:'selected-evidence-request',compiler:VERSION,question,workspace,known_as_of,valid_at,
    evidence:[], methods_status:'No methods automatically applied', limits:{max_bytes,unit:'UTF-8 bytes; not model tokens'},
    warning:'No retrieval, truth verification, human review or LLM analysis has occurred. Use the native compiler for production context.'};
  const omissions=[];
  for (const a of assertions) {
    await validateAssertion(a,sources);
    if (a.workspace !== workspace || !contains(a.available,known_as_of) || !contains(a.valid,valid_at) || a.evidence.some(e=>!available.some(s=>key(s)===e.source))) {
      omissions.push({id:a.id,reason:'access-or-time'}); continue;
    }
    const record = {assertion_id:a.id,proposition:a.proposition,epistemic_type:a.epistemic_type,profile:a.profile,
      citations:a.evidence.map(e=>{const s=available.find(s=>key(s)===e.source);return {source:e.source,span:clone(e),quote:passage(e,s),source_family:s.source_family};}),
      checks:semanticChecks(a,a.evidence.map(e=>passage(e,available.find(s=>key(s)===e.source))).join('\n'))};
    packet.evidence.push(record);
    if (bytes(JSON.stringify(packet)).length > max_bytes) {packet.evidence.pop();omissions.push({id:a.id,reason:'byte-budget'});}
  }
  requireValue(bytes(JSON.stringify(packet)).length<=max_bytes,'Budget cannot fit the request envelope');
  return {packet,receipt:{digest:await sha256(JSON.stringify(packet)),bytes:bytes(JSON.stringify(packet)).length,selected:packet.evidence.map(e=>e.assertion_id),omissions,
    method:'Explicit selection only; not a salience or model-performance benchmark.'}};
}
export function briefPreparation(context) {
  const p=context.packet;
  return ['# Decision brief preparation — UNREVIEWED', '', p.question, '',
    '## Attributed source statements', ...p.evidence.map(e=>`- ${e.proposition} [${e.citations.map(c=>c.source).join(', ')}]`), '',
    '## Questions that need review', ...[...new Set(p.evidence.flatMap(e=>e.checks.map(c=>c.note)))].map(n=>`- ${n}`), '',
    '## Alternatives, implementation, affected groups and missing evidence',
    'Not assessed by this extractive tool. An analyst must complete these sections.', '',
    `Context receipt: ${context.receipt.digest}`, 'No model was called; quotations are not verified facts.' ].join('\n');
}
export async function nativePackInputs({id,title,author,workspace,sources,assertions,memories=[],mode='references_only',created_at}) {
  requireValue(idOK(id) && [title,author].every(nonempty) && idOK(workspace) && epoch(created_at), 'Explicit capsule identity/author/time required');
  requireValue(['references_only','evidence_included'].includes(mode),'Invalid evidence mode');
  limitArray(sources,32,'sources'); limitArray(assertions,256,'assertions'); limitArray(memories,32,'methods');
  requireValue(new Set(sources.map(key)).size===sources.length && new Set(assertions.map(a=>a.id)).size===assertions.length,'Duplicate source/assertion');
  const evidence={};
  for (const s of sources) {
    await validateSource(s);
    requireValue(s.workspace===workspace && !s.withdrawn && !s.restricted, 'Export denied for unavailable/cross-workspace source');
    if (mode==='evidence_included') {requireValue(s.redistributable && nonempty(s.license),'Evidence redistribution not authorized');evidence[`evidence/${await sha256(key(s))}.txt`]=s.text;}
  }
  for (const a of assertions) {await validateAssertion(a,sources);requireValue(a.workspace===workspace,'Cross-workspace assertion');}
  requireValue(new Set(memories.map(m=>m.method?.id)).size===memories.length,'Duplicate memory ID');
  for (const m of memories) {
    requireValue(m.method && m.annotation && idOK(m.method.id) && m.method.version===1 && m.method.kind==='distinction' && nonempty(m.method.description) && nonempty(m.method.scope), 'Invalid method proposal');
    requireValue(Array.isArray(m.method.provenance) && m.method.provenance.length>0 && m.method.provenance.every(ref=>sources.some(s=>key(s)===ref)), 'Missing method source provenance');
    requireValue(Array.isArray(m.method.failure_conditions) && m.method.failure_conditions.length>0 && m.method.failure_conditions.every(nonempty), 'Method failure conditions required');
  }
  const safeMemories=memories.map(m=>memoryProposal({id:m.method.id,description:m.method.description,scope:m.method.scope,source_refs:m.method.provenance,created_at:m.annotation.created_at,prerequisites:m.annotation.prerequisites,exceptions:m.annotation.exceptions}));
  const data={assertions:clone(assertions),sources:sources.map(s=>({...clone(s),text:''})),methods:safeMemories.map(m=>m.method),reviews:[],
    artifacts:{'tacitus.policy-lab':JSON.stringify({ontology:ONTOLOGY.id,status:'unreviewed-proposal',memory_annotations:safeMemories.map(m=>({id:m.method.id,...clone(m.annotation)}))})}};
  const manifest={schema_version:1,package_version:1,id,author,title,domain:'political-policy-analysis',scope:workspace,purpose:'Unreviewed evidence and method proposals for native Rust validation',
    profile:memories.length && !assertions.length?'cognitive_method':'context',created_at,valid_until:null,supersedes:null,dependencies:[],license:'License per source; no new evidence rights granted',
    confidentiality:'private',sharing_policy:'Local only; review every exported field before sharing',evidence_mode:mode,
    limitations:[mode==='references_only'?'Evidence text is absent; literal/semantic support cannot be rechecked without authorized source versions.':'Evidence included only where explicitly marked redistributable.',
      'Provisional bridge to inspected Rust wire format. Full Rust round-trip not executed in this environment. No canonical Praxis compatibility claim.'],files:{}};
  return {manifest,data,evidence};
}

/** Typed relation proposals; never graph entailment or canonical promotion. */
export function relationProposal({predicate,subject,object,assertion_id,epistemic_type}) {
  const definition=ONTOLOGY.relations.find(r=>r.id===predicate);
  requireValue(definition && subject && object && idOK(subject.id) && idOK(object.id) && idOK(assertion_id),'Known relation, entities and source assertion required');
  for (const entity of [subject,object]) requireValue(ONTOLOGY.concepts.some(c=>c.id===entity.type),'Unknown entity type');
  requireValue((definition.subject==='*'||definition.subject===subject.type) && (definition.object==='*'||definition.object===object.type),'Ontology domain/range mismatch');
  requireValue(ONTOLOGY.epistemic_types.includes(epistemic_type),'Epistemic type required');
  return {predicate,subject:{id:subject.id,type:subject.type},object:{id:object.id,type:object.type},assertion_id,epistemic_type,status:'unreviewed-relation-proposal',ontology:ONTOLOGY.id};
}
/** Trusted declarative module dispatch. No import(), eval(), shell, or network tools. */
export async function runTool(id,input) {
  const module=MODULES.find(m=>m.id===id);
  requireValue(module && input && typeof input==='object','Unknown module or missing input');
  let result;
  switch(id) {
    case 'semantic-checks': requireValue(input.assertion && typeof input.assertion.proposition==='string','Assertion required'); result=semanticChecks(input.assertion,input.source_text??''); break;
    case 'decision-brief': result=briefPreparation(await contextRequest(input)); break;
    case 'competing-hypotheses': result=hypothesisMatrix(input.hypotheses,input.observations); break;
    case 'change-impact': result=dependencyImpact(input.changed,input.records); break;
    case 'cognitive-memory': result=memoryProposal(input); break;
    case 'native-capsule-input': result=await nativePackInputs(input); break;
    default: throw new Error('No trusted handler');
  }
  return {module:id,version:VERSION,network_calls:0,canonical_writes:0,result};
}
