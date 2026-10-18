// SPDX-License-Identifier: Apache-2.0
import http from 'node:http';
import {readFile} from 'node:fs/promises';
import {randomBytes} from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {LocalCatalog,importExchange,inspectInputs} from '../tacitus-capsule-catalog/catalog.mjs';
import {passage,key} from '../tacitus-policy-lab/core.mjs';
const root=new URL('./',import.meta.url), MAX=2*1024*1024;
export async function start({port=0,seed=true}={}) {
 const catalog=new LocalCatalog(), token=randomBytes(32).toString('hex');
 if(seed)await catalog.add(await importExchange(await readFile(new URL('../tacitus-capsule-catalog/fixtures/fictional-pilot.exchange.json',root),'utf8')));
 const staticFiles=new Map(await Promise.all([['/','index.html','text/html'],['/app.js','app.js','text/javascript'],['/detail.mjs','detail.mjs','text/javascript'],['/style.css','style.css','text/css']].map(async([url,file,type])=>[url,{bytes:await readFile(new URL(file,root)),type}])));
 let origin;let imports=Promise.resolve();
 const server=http.createServer(async(req,res)=>{
  res.setHeader('Cache-Control','no-store');res.setHeader('X-Content-Type-Options','nosniff');res.setHeader('Referrer-Policy','no-referrer');res.setHeader('Content-Security-Policy',"default-src 'self'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'none'");
  const send=(status,value,type='application/json')=>{res.writeHead(status,{'Content-Type':type});res.end(type==='application/json'?JSON.stringify(value):value);};
  try {
   if(req.headers.host!==new URL(origin).host || (req.headers.origin && req.headers.origin!==origin) || req.headers['sec-fetch-site']==='cross-site')return send(403,{error:'Local same-origin access only'});
   const url=new URL(req.url,origin);
   if(req.method==='GET'&&staticFiles.has(url.pathname)){const f=staticFiles.get(url.pathname);return send(200,f.bytes,f.type);}
   if(url.pathname==='/api/session'&&req.method==='GET')return send(200,{token,mode:'ephemeral-local',synthetic:true});
   if(req.headers['x-catalog-token']!==token)return send(403,{error:'Local session required'});
   if(req.method==='GET'&&url.pathname==='/api/catalog')return send(200,catalog.search(url.searchParams.get('q')||''));
   if(req.method==='GET'&&url.pathname==='/api/detail'){
    const text=await catalog.export(url.searchParams.get('id'));const {inputs,content_digest}=JSON.parse(text);
    const {sources}=await inspectInputs(inputs);
    const citations=Object.fromEntries(inputs.data.assertions.map(a=>[a.id,a.evidence.map(span=>({source:span.source,start:span.start,end:span.end,source_valid:inputs.data.sources.find(s=>key(s)===span.source).valid,source_available:inputs.data.sources.find(s=>key(s)===span.source).available,quote:sources.length?passage(span,sources.find(s=>key(s)===span.source)):null}))]));
    return send(200,{citations,manifest:inputs.manifest,sources:inputs.data.sources,assertions:inputs.data.assertions,methods:inputs.data.methods,content_digest});
   }
   if(req.method==='GET'&&url.pathname==='/api/export')return send(200,await catalog.export(url.searchParams.get('id')),'application/octet-stream');
   if(req.method!=='POST'||!['/api/import','/api/attach'].includes(url.pathname))return send(404,{error:'Unknown operation'});
   if(req.headers['content-type']!=='application/json')return send(415,{error:'JSON required'});
   const chunks=[];let size=0;for await(const chunk of req){size+=chunk.length;if(size>MAX)return send(413,{error:'File exceeds 2 MiB'});chunks.push(chunk);}
   const raw=new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(Buffer.concat(chunks));
   if(url.pathname==='/api/import'){
    const inputs=await importExchange(raw);
    const commit=imports.then(async()=>{const records=catalog.search();if(records.length>=100&&!records.some(r=>r.id===`${inputs.manifest.id}@${inputs.manifest.package_version}`))throw Error('Local catalog limit reached');return catalog.add(inputs);});
    imports=commit.catch(()=>{});return send(200,await commit);
   }
   const body=JSON.parse(raw);if(Object.keys(body).sort().join(',')!=='allowEvidence,id'||body.allowEvidence!==true||typeof body.id!=='string')return send(400,{error:'Explicit evidence inclusion required'});
   return send(200,await catalog.attach(body.id,{at:Math.floor(Date.now()/1000),allowEvidence:true}));
  }catch(error){send(400,{error:error.message});}
 });
 server.requestTimeout=15000;server.headersTimeout=10000;
 await new Promise(resolve=>server.listen(port,'127.0.0.1',resolve));origin=`http://127.0.0.1:${server.address().port}`;
 return {server,origin,catalog,close:()=>new Promise(resolve=>server.close(resolve))};
}
if(process.argv[1]===fileURLToPath(import.meta.url)){const app=await start({port:Number(process.argv[2]||8787)});console.log(`TACITUS local catalog: ${app.origin}\nTemporary session; imported files disappear when this process stops.`);}
