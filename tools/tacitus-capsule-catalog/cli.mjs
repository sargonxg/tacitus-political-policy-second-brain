#!/usr/bin/env node
// SPDX-License-Identifier: Apache-2.0
import {writeFile} from 'node:fs/promises';
import {demoOutput,DEMO_TIME} from '../tacitus-policy-lab/demo.mjs';
import {exportExchange} from './catalog.mjs';
import {readBounded,readCatalog,updateCatalog} from './store.mjs';
const [command,directory,arg,extra,reason]=process.argv.slice(2);
try {
 if(!directory)throw new Error('Usage: node tools/tacitus-capsule-catalog/cli.mjs demo|list|import|export|attach|retract|revoke|restore DIRECTORY [query|FILE|ID] [OUTPUT|EPOCH] [reason]');
 let result;
 if(command==='demo'){
  const {inputs}=await demoOutput();const exchange=await exportExchange(inputs);
  const added=await updateCatalog(directory,api=>api.import(exchange));const c=await readCatalog(directory);
  result={synthetic:true,added,listings:c.search('pilot'),attachment:await c.attach(added.id,{at:DEMO_TIME+1,allowEvidence:true}),limitations:['Local prototype only; no public listing, payment, native ZIP validation or live PRAXIS integration.']};
 }else if(command==='import')result=await updateCatalog(directory,async api=>api.import(await readBounded(arg,2*1024*1024)));
 else if(command==='list')result=(await readCatalog(directory)).search(arg??'');
 else if(command==='export'){
  const text=await(await readCatalog(directory)).export(arg);if(!extra)throw new Error('Explicit new output file required');await writeFile(extra,text,{flag:'wx',mode:0o600});result={exported:extra};
 }else if(command==='attach'){
  // CLI invocation explicitly asks inclusion in this local proposal, not remote use.
  result=await(await readCatalog(directory)).attach(arg,{at:Number(extra),allowEvidence:true});
 }else if(['retract','revoke','restore'].includes(command)){
  result=await updateCatalog(directory,({catalog})=>{const record=catalog.search().find(x=>x.id===arg);if(!record)throw new Error('Unknown capsule');return catalog.transition(arg,record.content_digest,{retract:'retracted',revoke:'revoked',restore:'active'}[command],reason,Number(extra));});
 }else throw new Error('Unknown command');
 console.log(JSON.stringify(result,null,2));
}catch(error){console.error(error.message);process.exitCode=1;}
