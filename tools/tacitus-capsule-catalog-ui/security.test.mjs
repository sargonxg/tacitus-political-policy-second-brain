// SPDX-License-Identifier: Apache-2.0
// Independent HTTP-boundary regression checks. Loopback only; no external services.
import test from 'node:test';
import assert from 'node:assert/strict';
import http from 'node:http';
import {readFile} from 'node:fs/promises';
import {start} from './server.mjs';
import {exportExchange,importExchange} from '../tacitus-capsule-catalog/catalog.mjs';
const fixtureText=await readFile(new URL('../tacitus-capsule-catalog/fixtures/fictional-pilot.exchange.json',import.meta.url),'utf8');
const fixture=()=>JSON.parse(fixtureText).inputs;
const request=(app,path='/',{method='GET',headers={},body}={})=>new Promise((resolve,reject)=>{
 const req=http.request(app.origin,{path,method,headers,agent:false},res=>{
  const chunks=[];res.on('data',c=>chunks.push(c));res.on('error',reject);res.on('end',()=>resolve({status:res.statusCode,headers:res.headers,text:Buffer.concat(chunks).toString('utf8')}));
 });req.on('error',reject);if(body!==undefined)req.write(body);req.end();
});
async function session(t,{seed=false}={}){
 const app=await start({seed});t.after(()=>app.close());
 const r=await request(app,'/api/session');assert.equal(r.status,200);
 app.headers={'X-Catalog-Token':JSON.parse(r.text).token};return app;
}
const post=(app,path,body,headers={})=>request(app,path,{method:'POST',body,headers:{...app.headers,'Content-Type':'application/json',...headers}});

test('server binds only IPv4 loopback and returns restrictive browser headers',async t=>{
 const app=await session(t),r=await request(app);
 assert.equal(app.server.address().address,'127.0.0.1');assert.equal(r.status,200);
 assert.equal(r.headers['cache-control'],'no-store');assert.equal(r.headers['x-content-type-options'],'nosniff');
 assert.equal(r.headers['referrer-policy'],'no-referrer');assert.match(r.headers['content-security-policy'],/frame-ancestors 'none'/);
 assert.match(r.headers['content-security-policy'],/object-src 'none'/);assert.match(r.headers['content-security-policy'],/form-action 'none'/);
 assert.equal(r.headers['access-control-allow-origin'],undefined);
});
test('remote Host, mismatched Origin, cross-site and null Origin cannot obtain session',async t=>{
 const app=await session(t);
 for(const headers of [{Host:'evil.example'},{Host:'localhost:'+app.server.address().port},{Origin:'https://evil.example'},{Origin:'null'},{Origin:app.origin+'/'},{'Sec-Fetch-Site':'cross-site'}]){
  const r=await request(app,'/api/session',{headers});assert.equal(r.status,403,JSON.stringify(headers));assert.doesNotMatch(r.text,new RegExp(app.headers['X-Catalog-Token']));
 }
 assert.equal((await request(app,'/api/session',{headers:{Origin:app.origin,'Sec-Fetch-Site':'same-origin'}})).status,200);
});
test('every data route requires the per-process custom-header token',async t=>{
 const app=await session(t,{seed:true}),id='fictional-pilot-capsule@1';
 for(const path of ['/api/catalog','/api/detail?id='+id,'/api/export?id='+id])for(const headers of [{},{'X-Catalog-Token':'wrong'}])assert.equal((await request(app,path,{headers})).status,403);
 for(const path of ['/api/import','/api/attach'])assert.equal((await request(app,path,{method:'POST',body:fixtureText,headers:{'Content-Type':'application/json'}})).status,403);
 const other=await session(t);assert.notEqual(app.headers['X-Catalog-Token'],other.headers['X-Catalog-Token']);
 assert.equal((await request(other,'/api/catalog',{headers:app.headers})).status,403);
 assert.equal((await request(app,'/api/import',{method:'OPTIONS',headers:{Origin:'https://evil.example','Access-Control-Request-Method':'POST'}})).status,403);
});
test('invalid routes and traversal cannot read local files',async t=>{
 const app=await session(t);
 for(const path of ['/server.mjs','/../server.mjs','/%2e%2e/server.mjs','/../../etc/passwd','/%2fetc%2fpasswd','/api/import?path=/etc/passwd']){
  const r=await request(app,path,{headers:app.headers});assert.equal(r.status,404,path);assert.doesNotMatch(r.text,/root:|createServer/);
 }
 const r=await post(app,'/api/import',JSON.stringify({path:'/etc/passwd'}));assert.equal(r.status,400);
});
test('mutation requires JSON and rejects malformed UTF-8 and oversized bodies',async t=>{
 const app=await session(t);
 assert.equal((await post(app,'/api/import',fixtureText,{'Content-Type':'text/plain'})).status,415);
 assert.equal((await post(app,'/api/import',Buffer.from([0xff,0xfe]))).status,400);
 assert.equal((await post(app,'/api/import','x'.repeat(2*1024*1024+1))).status,413);
 assert.equal(app.catalog.search().length,0);
});
test('BOM-prefixed import bytes remain noncanonical and are rejected',async t=>{
 const app=await session(t),r=await post(app,'/api/import','\ufeff'+fixtureText);
 assert.equal(r.status,400);assert.equal(app.catalog.search().length,0);
});
test('import/export preserve exact validated exchange and unreviewed status',async t=>{
 const app=await session(t),r=await post(app,'/api/import',fixtureText);assert.equal(r.status,200);
 const id=JSON.parse(r.text).id,search=await request(app,'/api/catalog',{headers:app.headers});
 assert.equal(JSON.parse(search.text)[0].review_status,'imported-unreviewed');
 const exported=await request(app,'/api/export?id='+encodeURIComponent(id),{headers:app.headers});
 assert.equal(exported.status,200);assert.equal(exported.text,fixtureText);assert.deepEqual(await importExchange(exported.text),fixture());
 const duplicate=await post(app,'/api/import',fixtureText);assert.equal(JSON.parse(duplicate.text).duplicate,true);assert.equal(app.catalog.search().length,1);
});
test('attachment requires exact consent and uses trusted server time',async t=>{
 const app=await session(t,{seed:true}),id='fictional-pilot-capsule@1';
 for(const body of [{id},{id,allowEvidence:false},{id,allowEvidence:'true'},{id,allowEvidence:true,at:1780315201},null,[]])assert.equal((await post(app,'/api/attach',JSON.stringify(body))).status,400);
 const before=Math.floor(Date.now()/1000),r=await post(app,'/api/attach',JSON.stringify({id,allowEvidence:true}));assert.equal(r.status,200);
 const a=JSON.parse(r.text);assert.ok(a.attached_at>=before&&a.attached_at<=Math.floor(Date.now()/1000));assert.equal(a.trust,'untrusted-data');assert.equal(a.review_status,'imported-unreviewed');
});
test('attachment fails closed for expired, withdrawn, references-only and future records',async t=>{
 const app=await session(t);let n=0;
 for(const change of [f=>{f.manifest.valid_until=Math.floor(Date.now()/1000)-1;},f=>{f.manifest.created_at=Math.floor(Date.now()/1000)+60;},f=>{f.manifest.evidence_mode='references_only';f.evidence={};},f=>{f.data.sources[0].valid.end=Math.floor(Date.now()/1000)-1;},f=>{f.data.assertions[0].available.end=Math.floor(Date.now()/1000)-1;}]){
  const f=fixture();f.manifest.id='fail-closed-'+(++n);change(f);const imported=await post(app,'/api/import',await exportExchange(f));assert.equal(imported.status,200);
  const id=JSON.parse(imported.text).id;assert.equal((await post(app,'/api/attach',JSON.stringify({id,allowEvidence:true}))).status,400);
 }
 const r=await app.catalog.add(fixture());app.catalog.transition(r.id,r.digest,'revoked','Independent regression test',Math.floor(Date.now()/1000));
 assert.equal((await post(app,'/api/attach',JSON.stringify({id:r.id,allowEvidence:true}))).status,400);
 assert.equal((await request(app,'/api/export?id='+r.id,{headers:app.headers})).status,400);
});
test('untrusted HTML strings stay JSON data and UI has no HTML injection sink',async t=>{
 const app=await session(t),f=fixture();f.manifest.title='<img src=x onerror=alert(1)>';f.manifest.limitations=['</script><script>alert(1)</script>'];
 const imported=await post(app,'/api/import',await exportExchange(f));assert.equal(imported.status,200);
 const detail=await request(app,'/api/detail?id='+JSON.parse(imported.text).id,{headers:app.headers});assert.equal(JSON.parse(detail.text).manifest.title,f.manifest.title);
 assert.equal(detail.headers['content-type'],'application/json');
 const shell=await request(app);assert.doesNotMatch(shell.text,/<img src=x/);
 const script=await request(app,'/app.js');assert.match(script.text,/textContent/);assert.doesNotMatch(script.text,/innerHTML|outerHTML|insertAdjacentHTML|document\.write|\beval\s*\(|new Function\s*\(/);
});
test('concurrent imports cannot bypass the 100-entry catalog bound',async t=>{
 const app=await session(t);
 for(let i=0;i<99;i++){const f=fixture();f.manifest.id='seed-'+i;await app.catalog.add(f);}
 const payloads=await Promise.all(Array.from({length:6},async(_,i)=>{const f=fixture();f.manifest.id='parallel-'+i;return exportExchange(f);}));
 const results=await Promise.all(payloads.map(p=>post(app,'/api/import',p)));
 assert.ok(app.catalog.search().length<=100,`Catalog contains ${app.catalog.search().length} records after concurrent import`);
 assert.equal(results.filter(r=>r.status===200).length,1);
 const existing=fixture();existing.manifest.id='seed-0';
 const duplicate=await post(app,'/api/import',await exportExchange(existing));
 assert.equal(duplicate.status,200);assert.equal(JSON.parse(duplicate.text).duplicate,true);
 existing.manifest.title+=' changed';const collision=await post(app,'/api/import',await exportExchange(existing));
 assert.equal(collision.status,400);assert.match(collision.text,/collision/i);assert.equal(app.catalog.search().length,100);
});
