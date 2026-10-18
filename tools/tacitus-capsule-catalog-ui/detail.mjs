// SPDX-License-Identifier: Apache-2.0
// Presentation values only. The UI renders every label/value with textContent.
export function qualificationRows(detail){
 const list=values=>values.length?values.join('; '):'None declared; not proof that none apply';
 return [
  ...detail.assertions.map(a=>({title:`Claim qualifications: ${a.id}`,fields:[['Conditions',list(a.conditions)],['Exceptions',list(a.exceptions)],['Modality',a.modality||'Not supplied'],['Valid time',JSON.stringify(a.valid??null)],['Available time',JSON.stringify(a.available??null)],...(detail.citations?.[a.id]||[]).flatMap(c=>[[`Quoted evidence ${c.source} [${c.start}:${c.end}]`,c.quote===null?'Evidence not included; excerpt not verified':c.quote],[`Source valid time ${c.source}`,JSON.stringify(c.source_valid)],[`Source available time ${c.source}`,JSON.stringify(c.source_available)]])]})),
  ...detail.methods.map(m=>({title:`Cognitive method: ${m.id}@${m.version}`,fields:[['Description',m.description],['Scope',m.scope],['Failure conditions',list(m.failure_conditions)],['Provenance',list(m.provenance)],['Review status',m.review_state]]}))
 ];
}
