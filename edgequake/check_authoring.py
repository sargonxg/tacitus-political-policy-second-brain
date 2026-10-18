#!/usr/bin/env python3
"""Run after building examples. Uses a fresh temporary directory; no network/overwrites."""
import json,pathlib,subprocess,tempfile,sys,hashlib
root=pathlib.Path(__file__).resolve().parent
(root/'evidence').mkdir(exist_ok=True)
bins=pathlib.Path(sys.argv[1]) if len(sys.argv)>1 else root/'target/debug/examples'
def call(args,**kw):
 return subprocess.run([str(x) for x in args],capture_output=True,**kw)
with tempfile.TemporaryDirectory(prefix='tacitus-author-') as tmp:
 out=pathlib.Path(tmp)/'new.zip'; inp=root/'authoring-demo/synthetic-input.json'
 a=call([bins/'author_native_v2','--confirm-original-evidence',inp,out]);assert a.returncode==0,a.stderr
 original=out.read_bytes();again=call([bins/'author_native_v2','--confirm-original-evidence',inp,out]);assert again.returncode!=0 and out.read_bytes()==original
 inspect=call([bins/'v2_workflow','inspect',out]);assert inspect.returncode==0,inspect.stderr
 req=json.loads((root/'authoring-demo/request.json').read_text()); raw=json.dumps(req).encode()
 review=call([bins/'v2_workflow','review',out],input=raw);assert review.returncode==0,review.stderr
 doc=json.loads(review.stdout)['document'];assert len(doc['layered_context']['packet']['claims'])==1
 text=call([bins/'v2_workflow','review-text',out],input=raw);assert text.returncode==0,text.stderr
 assert b'UNKNOWN WORLD VALIDITY' in text.stdout and b'SYNTHETIC' in text.stdout and b'staffing' in text.stdout
 req['time_mode']='require_declared_interval'; strict=call([bins/'v2_workflow','review',out],input=json.dumps(req).encode());assert strict.returncode!=0 and not strict.stdout
 for name,payload in [('created.zip',original),('inspect.json',inspect.stdout),('review.json',review.stdout),('review.txt',text.stdout)]: (root/'evidence'/name).write_bytes(payload)
 receipt={'archive_sha256':hashlib.sha256(original).hexdigest(),'created_inspected_reviewed':True,'claims':1,'unknown_world_validity_preserved':True,'synthetic_and_exception_retained':True,'verified_date_request_rejected':True,'existing_output_preserved':True,'json_bytes':len(review.stdout),'text_bytes':len(text.stdout),'limitation':'Structural workflow only; no source authenticity, truth or rights adjudication.'}
 (root/'evidence/authoring-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(receipt,indent=2))
