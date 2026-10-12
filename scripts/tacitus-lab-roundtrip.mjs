// Native Rust is the only capsule implementation used by this rehearsal.
import {mkdtemp,writeFile,readFile} from 'node:fs/promises';
import {resolve,join,dirname} from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
import {mkdir} from 'node:fs/promises';
import assert from 'node:assert/strict';

const root=resolve(dirname(fileURLToPath(import.meta.url)),'..');
function run(command,args,quiet=false) {
  const result=spawnSync(command,args,{cwd:root,encoding:'utf8',maxBuffer:8*1024*1024});
  if(result.error||result.status!==0) throw new Error(`${command} failed: ${result.error?.message??result.stderr}`);
  if(!quiet) process.stdout.write(result.stdout);
  return result.stdout;
}
async function main() {
  await mkdir(join(root,'.tacitus'),{recursive:true});
  const dir=await mkdtemp(join(root,'.tacitus','lab-native-'));
  const selected=join(dir,'selected');
  run(process.execPath,['tools/tacitus-policy-lab/cli.mjs','demo','--out',selected]);
  const data=JSON.parse(await readFile(join(selected,'data.json'),'utf8'));
  const session=JSON.parse(await readFile(join(selected,'session.json'),'utf8'));
  const workspace='fresh-policy-rehearsal';
  const contract={workspace,question:session.contract.question,decision_maker:'Fictional regional council',audience:'Analysts',mandate:'Assess fictional pilot prerequisites',
    objectives:['Preserve authorization and funding qualifications'],constraints:['Approval and implementation resources'],time_horizon:'One month',
    known_as_of:session.contract.known_as_of,valid_at:session.contract.valid_at,selected_sources:data.sources.map(s=>`${s.id}@${s.version}`),selected_capsules:[],
    permitted_tools:[],output_format:'decision_brief',context_tokens:32000,output_tokens:2048,overhead_tokens:1024};
  const contractPath=join(dir,'contract.json'); await writeFile(contractPath,JSON.stringify(contract,null,2),{flag:'wx'});
  const base=['run','--offline','--locked','--quiet','--manifest-path','edgequake/Cargo.toml','-p','tacitus-capsules','--'];
  const manifest=join(selected,'manifest.json'),dataPath=join(selected,'data.json'),bundle=join(dir,'rehearsal.capsule'),packetPath=join(dir,'packet.json');
  const preview=run('cargo',[...base,'preview',manifest,dataPath,selected],true);
  assert.ok(preview.includes('FILE capsule.json')); assert.ok(preview.includes('FILE manifest.json'));
  run('cargo',[...base,'pack',manifest,dataPath,selected,bundle]);
  run('cargo',[...base,'validate',bundle]);
  run('cargo',[...base,'compile',bundle,workspace,contractPath,packetPath]);
  const packet=JSON.parse(await readFile(packetPath,'utf8'));
  assert.equal(Object.keys(packet.sources).length,4); assert.equal(Object.keys(packet.citations).length,4);
  for(const [id,span] of Object.entries(packet.citations)) {
    const source=data.sources.find(s=>`${s.id}@${s.version}`===span.source); assert.ok(source,id);
    assert.equal(source.text_hash,span.text_hash); assert.equal(span.start,0);
    assert.equal(span.end,Buffer.byteLength(packet.sources[span.source]));
    const original=session.sources.find(s=>`${s.id}@${s.version}`===span.source);
    assert.equal(packet.sources[span.source],original.text);
  }
  assert.ok(Object.values(packet.sources).some(text=>text.includes('subject to domestic approval')));
  assert.ok(Object.values(packet.sources).some(text=>text.includes('No expenditure has been recorded')));
  assert.equal(packet.source_families['dispatch-A'].length,2);
  assert.ok(packet.warnings.some(w=>w.includes('lexical retrieval ONLY')));
  console.log('PASS: actual lab -> native preview/pack/validate/import/compile in a fresh workspace; 4 exact source versions/citations, qualifications and common-origin provenance preserved.');
  console.log('No model, database persistence or authenticated application workflow ran. Artifacts: '+dir);
}
main().catch(e=>{console.error(e.message);process.exitCode=1;});
