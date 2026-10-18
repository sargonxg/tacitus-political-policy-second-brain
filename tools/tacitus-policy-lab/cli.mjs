// SPDX-License-Identifier: Apache-2.0
import {createServer} from 'node:http';
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {resolve,join,dirname} from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
import {demo, demoOutput} from './demo.mjs';
import {briefPreparation} from './core.mjs';
const here=dirname(fileURLToPath(import.meta.url));
const allowed=new Map([['/',['index.html','text/html']],['/ui.mjs',['ui.mjs','text/javascript']],['/core.mjs',['core.mjs','text/javascript']],['/ontology.mjs',['ontology.mjs','text/javascript']],['/demo.mjs',['demo.mjs','text/javascript']],['/style.css',['style.css','text/css']]]);
export function server(port=4317) {
  if (!Number.isInteger(port)||port<0||port>65535) throw new Error('Invalid port');
  const app=createServer(async(req,res)=>{
    const actual=app.address().port;
    const security={'X-Content-Type-Options':'nosniff','Cache-Control':'no-store','Referrer-Policy':'no-referrer',
      'Content-Security-Policy':"default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self' data:; base-uri 'none'; frame-ancestors 'none'; form-action 'none'",
      'Cross-Origin-Resource-Policy':'same-origin'};
    const reply=(code,body,type='text/plain')=>{res.writeHead(code,{...security,'Content-Type':`${type}; charset=utf-8`});res.end(req.method==='HEAD'?'':body);};
    if(![`127.0.0.1:${actual}`,`localhost:${actual}`].includes(req.headers.host))return reply(403,'Unapproved Host');
    if(!['GET','HEAD'].includes(req.method))return reply(405,'Read-only local development server');
    const path=req.url?.split('?')[0];
    if(path==='/health')return reply(200,JSON.stringify({status:'ok',mode:'deterministic-policy-lab',model:'not-connected',storage:'browser-session-only'}),'application/json');
    const entry=allowed.get(path);
    if(!entry)return reply(404,'Not found');
    try {return reply(200,await readFile(join(here,entry[0])),entry[1]);}catch{return reply(500,'Local asset unavailable');}
  });
  return app;
}
async function main(args) {
  const command=args[0]??'help';
  if(command==='review-demo'){
    if(args.length!==1 && !(args.length===3 && args[1]==='--max-bytes' && /^[1-9][0-9]*$/.test(args[2]))) throw new Error('Usage: review-demo [--max-bytes 2400]');
    const max_bytes=args.length===3 ? Number(args[2]) : 2400;
    if(!Number.isInteger(max_bytes)||max_bytes<512||max_bytes>250000) throw new Error('max-bytes must be 512–250000');
    const {changeReviewContext,changeReviewBrief}=await import('./change-review.mjs');
    const session=await demo(), since=session.contract.known_as_of;
    session.contract={...session.contract,known_as_of:since+86400,valid_at:since+86400,max_bytes};
    const result=await changeReviewContext(session,since);
    console.log(changeReviewBrief(result));return;
  }
  if(command==='benchmark'){
    const {benchmarkMain}=await import('./benchmark.mjs');
    return benchmarkMain(args.slice(1));
  }
  if(command==='serve'){
    const index=args.indexOf('--port');const port=index>=0?Number(args[index+1]):4317;
    const app=server(port);app.on('error',e=>{console.error(`Cannot start policy lab: ${e.code??e.message}`);process.exitCode=1;});
    app.listen(port,'127.0.0.1',()=>console.log(`TACITUS policy lab: http://127.0.0.1:${app.address().port}\nDeterministic tools, not the full EdgeQuake app. No model calls. Ctrl+C stops the server.`));
    const close=()=>app.close();process.on('SIGINT',close);process.on('SIGTERM',close);return;
  }
  if(command==='doctor'){
    const has=cmd=>{const r=spawnSync(cmd,['--version'],{timeout:3000,stdio:'ignore'});return !r.error&&r.status===0;};
    console.log(JSON.stringify({node:process.version,platform:process.platform,lab_requirement:'Node >=22; no npm installation required',node_requirement_met:Number(process.versions.node.split('.')[0])>=22,
      cargo:has('cargo'),docker:has('docker'),ollama:has('ollama'),live_inference:'NOT RUN',native_capsule_round_trip:'NOT RUN'},null,2));return;
  }
  if(command==='demo'){
    const result=await demoOutput();const index=args.indexOf('--out');
    if(index<0){console.log(briefPreparation(result.context));console.log('\nOmissions:',JSON.stringify(result.context.receipt.omissions));return;}
    if(!args[index+1])throw new Error('--out needs a NEW output directory');
    const dest=resolve(args[index+1]);
    await mkdir(dirname(dest),{recursive:true});
    await mkdir(dest,{recursive:false}); // Refuse existing directories, never overwrite private data.
    await mkdir(join(dest,'evidence'));
    const files={'manifest.json':result.inputs.manifest,'data.json':result.inputs.data,'request.json':result.context,'session.json':result.session};
    for(const [name,data] of Object.entries(files))await writeFile(join(dest,name),JSON.stringify(data,null,2)+'\n',{flag:'wx',mode:0o600});
    for(const [name,text] of Object.entries(result.inputs.evidence))await writeFile(join(dest,name),text,{flag:'wx',mode:0o600});
    await writeFile(join(dest,'brief.md'),briefPreparation(result.context),{flag:'wx',mode:0o600});
    console.log(`Created synthetic native pack inputs in ${dest}.\nRun the existing tacitus-capsules Rust preview/pack/validate commands before calling this a validated capsule.`);return;
  }
  console.log('Usage: node tools/tacitus-policy-lab/cli.mjs doctor | serve [--port 4317] | demo [--out NEW_DIRECTORY] | review-demo [--max-bytes 2400] | benchmark [--iterations 25] [--out NEW_FILE]\nTests: node --test tools/tacitus-policy-lab/test.mjs');
  if(command!=='help')process.exitCode=2;
}
if(process.argv[1]&&resolve(process.argv[1])===fileURLToPath(import.meta.url))main(process.argv.slice(2)).catch(e=>{console.error(e.message);process.exitCode=1;});
