#!/usr/bin/env python3
"""Synthetic collection/shard mechanics only; not a production index or quality benchmark."""
import argparse,copy,hashlib,json,pathlib,resource,subprocess,tempfile,time
from compare import tokens,encoded,baseline,coverage
FIELDS=['id','version','workspace','speaker','subject','predicate','object','proposition','epistemic_type','modality','conditions','exceptions','evidence','created_at']
def sha(b):return hashlib.sha256(b).hexdigest()
def fixture(template,n):
 c=copy.deepcopy(template);c['documents']=[];c['assertions']=[];c['graph']['nodes']={};c['graph']['links']=[];c['graph']['knowledge']={};c['graph']['claim_fingerprints']={};c['observed_at']={}
 c['manifest']['id']=f'synthetic-collection-{n[0]:04d}';c['manifest']['title']='Synthetic collection shard';c['extractor']='Explicit synthetic annotations; no inference'
 for i in n:
  subject=f'Institution{i:04d}';programme=f'programme{i:04d}'
  first=f'{subject} conditionally funds {programme}, except during unsafe weather.\n';second=f'{subject} withdraws funding for {programme}; the unsafe-weather exception remains.\n';text=first+second
  d=copy.deepcopy(template['documents'][0]);d.update(id=f'doc{i:04d}',document_id=f'doc{i:04d}',title=f'Synthetic {programme}',text=text,text_hash=sha(text.encode()),original_hash=sha(text.encode()),source_family=f'synthetic:{i}',author='Synthetic fixture generator')
  c['documents'].append(d);ids=[]
  for j,(line,start) in enumerate([(first,0),(second,len(first.encode()))]):
   a=copy.deepcopy(template['assertions'][0 if j==0 else 3]);a.update(id=f'claim{i:04d}-{j}',speaker='Synthetic fixture author',subject=subject,proposition=line.rstrip(),conditions=['Synthetic record; no real funding commitment.'],exceptions=['except during unsafe weather'],created_at=100+j*100)
   a['evidence']=[{'source':f'doc{i:04d}@1','start':start,'end':start+len(line.rstrip().encode()),'text_hash':d['text_hash'],'original_language':'en','locator':None,'translation':None}]
   a['evidence']=[{k:sp[k] for k in ['source','text_hash','start','end','locator','original_language','translation']} for sp in a['evidence']];a={k:a[k] for k in FIELDS};c['assertions'].append(a);c['graph']['claim_fingerprints'][a['id']]=sha(encoded(a));c['observed_at'][a['id']]=None;ids.append(a['id'])
  c['graph']['nodes'][subject]={'id':subject,'kind':'institution','label':subject,'claims':ids,'event_time':None}
  c['graph']['links'].append({'from':ids[1],'to':ids[0],'kind':'supersedes','rationale':'Explicit synthetic withdrawal; not a real-world authority claim.'})
 return c

def main():
 ap=argparse.ArgumentParser();ap.add_argument('--binary',required=True);ap.add_argument('--template',required=True);ap.add_argument('--output',required=True);a=ap.parse_args();out=pathlib.Path(a.output)
 if out.exists() or out.is_symlink():raise SystemExit('fresh output required')
 template=json.loads(pathlib.Path(a.template).read_text());runs=[]
 for count in [100,300]:
  with tempfile.TemporaryDirectory() as td:
   start=time.perf_counter_ns();index=[];manifest=[];shards=[]
   for startid in range(0,count,25):
    cap=fixture(template,range(startid,min(startid+25,count)));p=pathlib.Path(td)/f'{startid}.json';p.write_bytes(encoded(cap));z=p.with_suffix('.zip');subprocess.run([a.binary,'pack',str(p),str(z)],check=True,capture_output=True)
    shard_id=len(shards);shards.append((cap,z));manifest.append({'shard':shard_id,'archive_sha256':sha(z.read_bytes()),'source_count':len(cap['documents'])})
    for d in cap['documents']:
     offset=0
     for line in d['text'].splitlines(keepends=True):
      index.append({'source':d['id']+'@1','source_hash':d['text_hash'],'shard':shard_id,'start':offset,'end':offset+len(line.encode()),'text':line,'terms':tokens(line)});offset+=len(line.encode())
   build_ns=time.perf_counter_ns()-start
   query=f'programme{count-1:04d}';start=time.perf_counter_ns();ranked=sorted(index,key=lambda x:(-len(tokens(query)&x['terms']),x['source'],x['start']));matches=[x for x in ranked if tokens(query)&x['terms']];selected=sorted({x['shard']for x in matches});lookup_ns=time.perf_counter_ns()-start
   q={'question':query,'known_as_of':300,'valid_at':300,'time_mode':'disclose_unknown','max_bytes':32768,'revoked_sources':[]}
   start=time.perf_counter_ns();contexts=[]
   for sid in selected:
    r=subprocess.run([a.binary,'reuse',str(shards[sid][1])],input=encoded(q),capture_output=True,check=True);contexts.append(json.loads(r.stdout))
   reuse_ns=time.perf_counter_ns()-start
   packet={'format':'synthetic-collection-query/1','selected_shards':selected,'contexts':contexts}
   payload=encoded(packet);assert len(payload)<=32768
   claims=[cid for c in contexts for cid in c['context']['claims']];assert set(claims)=={f'claim{count-1:04d}-0',f'claim{count-1:04d}-1'}
   assert len(index)==2*count and all(m['source_count']<=25 for m in manifest)
   flat=copy.deepcopy(template);flat['documents']=[d for cap,_ in shards for d in cap['documents']];flat['assertions']=[x for cap,_ in shards for x in cap['assertions']]
   baselines={}
   for method in ['whole_document','line_chunk']:
    t=time.perf_counter_ns();b=baseline(flat,q,chunked=method=='line_chunk');elapsed=time.perf_counter_ns()-t
    covered=coverage(flat,b,method);assert set(covered)==set(claims)
    baselines[method]={'bytes':len(b),'covered_claim_evidence':covered,'elapsed_ns':elapsed,'payload_sha256':sha(b)}
   revised=copy.deepcopy(shards[-1][0]);doc=revised['documents'][-1];oldkey=doc['id']+'@1';doc['version']=2;doc['available_at']=301;doc['retrieved_at']=301;doc['text']+='Synthetic revision metadata: reviewed again.\n';doc['text_hash']=sha(doc['text'].encode());doc['original_hash']=doc['text_hash']
   for claim in revised['assertions']:
    if any(e['source']==oldkey for e in claim['evidence']):
     claim['version']=2;claim['created_at']=301
     for e in claim['evidence']: e['source']=doc['id']+'@2';e['text_hash']=doc['text_hash']
     revised['graph']['claim_fingerprints'][claim['id']]=sha(encoded(claim))
   revised['manifest']['package_version']=2;revised['manifest']['created_at']=301;revised['graph']['available_at']=301
   rp=pathlib.Path(td)/'revision.json';rz=pathlib.Path(td)/'revision.zip';rp.write_bytes(encoded(revised));t=time.perf_counter_ns();subprocess.run([a.binary,'pack',str(rp),str(rz)],capture_output=True,check=True);update_ns=time.perf_counter_ns()-t
   newmanifest=copy.deepcopy(manifest);newmanifest[-1]['previous_archive_sha256']=manifest[-1]['archive_sha256'];newmanifest[-1]['archive_sha256']=sha(rz.read_bytes());changed=[i for i,(old,new) in enumerate(zip(manifest,newmanifest)) if old!=new];assert changed==[len(shards)-1]
   index=[x for x in index if x['source']!=oldkey];offset=0
   for line in doc['text'].splitlines(keepends=True):
    index.append({'source':doc['id']+'@2','source_hash':doc['text_hash'],'shard':len(shards)-1,'start':offset,'end':offset+len(line.encode()),'text':line,'terms':tokens(line)});offset+=len(line.encode())
   newmatches=[x for x in index if tokens(query)&x['terms']];newselected=sorted({x['shard']for x in newmatches});assert newselected==selected and all(x['source']!=oldkey for x in newmatches)
   stale=subprocess.run([a.binary,'reuse',str(rz)],input=encoded(q),capture_output=True);assert stale.returncode!=0 and not stale.stdout
   newer=dict(q);newer['known_as_of']=301;newer['valid_at']=301
   current_paths={i:z for i,(_,z) in enumerate(shards)};current_paths[newselected[0]]=rz
   assert sha(current_paths[newselected[0]].read_bytes())==newmanifest[newselected[0]]['archive_sha256']
   updated=subprocess.run([a.binary,'reuse',str(current_paths[newselected[0]])],input=encoded(newer),capture_output=True,check=True);uo=json.loads(updated.stdout);assert all(v['assertion']['version']==2 for v in uo['context']['claims'].values());assert oldkey not in uo['source_metadata']
   runs.append({'baselines':baselines,'incremental_update':{'changed_shards':changed,'new_manifest_sha256':sha(encoded(newmanifest)),'repack_ns':update_ns,'source_revision':doc['id']+'@2','old_archive_preserved':shards[-1][1].exists(),'new_snapshot_checkpoint':301,'earlier_checkpoint_rejected':True,'refreshed_index_routes_revision':True,'updated_context_sha256':sha(updated.stdout)},'documents':count,'chunks':2*count,'updated_chunks':len(index),'shards':len(shards),'shard_manifest':manifest,'manifest_sha256':sha(encoded(manifest)),'query':query,'selected_shards':selected,'claims':claims,'output_bytes':len(payload),'payload_sha256':sha(payload),'build_pack_ns':build_ns,'collection_lookup_ns':lookup_ns,'reuse_subprocess_ns':reuse_ns,'index_serialized_bytes':len(encoded([{k:v for k,v in x.items() if k!='terms'}for x in index]))})
 report={'scope':'synthetic collection-wide lexical routing to independently validated <=25-source native shards','runs':runs,'python_peak_rss_kib':resource.getrusage(resource.RUSAGE_SELF).ru_maxrss,'limitations':['Synthetic throughput and mechanics only; no real collection extraction quality.','RSS is whole Python process peak; child Rust process memory not measured.','Timing includes process startup and is environment-specific; no model cost.','One synthetic revision repacks one shard; no persisted production registry, atomic update service or authentication.','This exact-key query selects one shard; broad multi-shard queries are not established.','The final wrapper is measured against32KiB; per-shard budget allocation for arbitrary queries is not implemented.']}
 with out.open('x') as f:json.dump(report,f,indent=2)
if __name__=='__main__':main()
