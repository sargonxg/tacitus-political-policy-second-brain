// SPDX-License-Identifier: Apache-2.0
// Explicit local-only storage. Never resolves URLs or imports executables.
import {mkdir,rename,open,unlink} from 'node:fs/promises';
import {join} from 'node:path';
import {constants} from 'node:fs';
import {LocalCatalog,canonical,importExchange,exportExchange} from './catalog.mjs';
const assert=(ok,message)=>{if(!ok)throw new Error(message);};
const MAX=16*1024*1024;
export async function readBounded(path,max=MAX) {
 assert(Number.isSafeInteger(max)&&max>0&&max<=MAX,'Invalid file size limit');
 assert(typeof constants.O_NOFOLLOW==='number','Safe local-file opening unsupported on this platform');
 const file=await open(path,constants.O_RDONLY|constants.O_NOFOLLOW|constants.O_NONBLOCK);
 try {
  const stat=await file.stat();assert(stat.isFile()&&stat.size<=max,'Regular bounded local file required');
  const chunks=[];let total=0;
  while(total<=max){const buf=Buffer.alloc(Math.min(65536,max+1-total));const {bytesRead}=await file.read(buf,0,buf.length,null);if(!bytesRead)break;chunks.push(buf.subarray(0,bytesRead));total+=bytesRead;}
  assert(total<=max,'Local file exceeds byte limit');
  return new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(Buffer.concat(chunks,total));
 } finally {await file.close();}
}
async function load(directory) {
 let saved;try{saved=JSON.parse(await readBounded(join(directory,'catalog.json')));}catch(e){if(e.code!=='ENOENT')throw e;saved={format:'tacitus-local-catalog-store/1',exchanges:[],events:[]};}
 assert(saved.format==='tacitus-local-catalog-store/1'&&Array.isArray(saved.exchanges)&&saved.exchanges.length<=100&&Array.isArray(saved.events)&&saved.events.length<=1000,'Invalid local catalog store');
 const catalog=new LocalCatalog();for(const text of saved.exchanges)await catalog.add(await importExchange(text));
 for(const event of saved.events){const replay=catalog.transition(event.id,event.content_digest,event.to,event.reason,event.at);assert(canonical(replay)===canonical(event),'Invalid local event chain');}
 return {catalog,saved};
}
export async function readCatalog(directory){return (await load(directory)).catalog;}
export async function updateCatalog(directory,operation){
 await mkdir(directory,{recursive:true,mode:0o700});
 const lock=await open(join(directory,'.catalog.lock'),'wx',0o600);
 try {
  const {catalog,saved}=await load(directory);
  const api={catalog,async import(text){const inputs=await importExchange(text);const result=await catalog.add(inputs);if(!result.duplicate)saved.exchanges.push(await exportExchange(inputs));return result;}};
  const result=await operation(api);saved.events=catalog.history();const text=canonical(saved);
  assert(saved.exchanges.length<=100&&saved.events.length<=1000&&Buffer.byteLength(text)<=MAX,'Local store limit exceeded');
  const temporary=join(directory,'.catalog.next');const tmp=await open(temporary,'wx',0o600);
  try{await tmp.writeFile(text);await tmp.sync();}finally{await tmp.close();}
  await rename(temporary,join(directory,'catalog.json'));return result;
 } finally{await lock.close();await unlink(join(directory,'.catalog.lock'));}
}
