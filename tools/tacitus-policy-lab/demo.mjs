// SPDX-License-Identifier: Apache-2.0
// All names and documents below are intentionally fictional, newly authored demo fixtures.
import {sourceVersion, assertionProposal, memoryProposal, contextRequest, nativePackInputs} from './core.mjs';
export const DEMO_TIME = 1780315200;
export async function demo() {
  const docs = [
    ['delegation','Official statement','Lydian delegation','The delegation may consider participating in the regional pilot, subject to domestic approval.','official-delegation','commitment',0],
    ['budget','Budget note','Regional budget office','The assembly authorized a ceiling of 10 million units. No expenditure has been recorded.','budget-office','funding',0],
    ['press','Dispatch','Harbor Press','According to a delegate, participation is agreed in principle; the cabinet has not approved it.','dispatch-A','position',0],
    ['reprint','Reprinted dispatch','Evening Ledger','According to a delegate, participation is agreed in principle; the cabinet has not approved it.','dispatch-A','position',0],
    ['correction','Later correction','Lydian delegation','The cabinet declined approval and the delegation withdrew the proposal.','official-delegation','commitment',86400]
  ];
  const sources=[], assertions=[];
  for (const [id,title,author,text,source_family,profile,offset] of docs) {
    const s=await sourceVersion({id,workspace:'fictional-pilot',title,author,text,source_family,published_at:DEMO_TIME+offset,retrieved_at:DEMO_TIME+offset,redistributable:true,license:'CC0-1.0 (synthetic fixture)'});
    sources.push(s);assertions.push(await assertionProposal(s,s.text,{id:`claim-${id}`,profile}));
  }
  const memories=[memoryProposal({id:'conditional-is-not-final',description:'Do not transform willingness to consider participation into an unconditional commitment. Check who can authorize the act.',scope:'fictional-pilot',source_refs:['delegation@1'],created_at:DEMO_TIME,prerequisites:['same_decision_context'],exceptions:['source_superseded']})];
  const contract={question:'Can the regional pilot proceed on the basis of these commitments and funding?',workspace:'fictional-pilot',known_as_of:DEMO_TIME,valid_at:DEMO_TIME,max_bytes:32000};
  return {format:'tacitus-policy-lab-session/1',synthetic:true,sources,assertions,memories,contract};
}
export async function demoOutput() {
  const session=await demo();
  const context=await contextRequest({...session,...session.contract});
  const selected=new Set(context.packet.evidence.map(e=>e.assertion_id));
  const assertions=session.assertions.filter(a=>selected.has(a.id));
  const refs=new Set(assertions.flatMap(a=>a.evidence.map(e=>e.source)));
  const inputs=await nativePackInputs({id:'fictional-pilot-capsule',title:'Fictional pilot dossier',author:'Synthetic example author',workspace:session.contract.workspace,
    sources:session.sources.filter(s=>refs.has(`${s.id}@${s.version}`)),assertions,memories:session.memories,mode:'evidence_included',created_at:DEMO_TIME});
  return {session,context,inputs};
}
