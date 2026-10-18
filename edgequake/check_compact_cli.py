"""Exercise actual CLI stdout bytes; no network or model calls."""
import json, subprocess, pathlib, hashlib, sys
root=pathlib.Path(__file__).parent
out=pathlib.Path(sys.argv[1]) if len(sys.argv)>1 else root/'outputs'
binary=root/'target/debug/examples/compact_review'
rows=[]
for short,slug in [('heat','nyc-heat-resilience'),('compute','compute-infrastructure-governance'),('access','humanitarian-access-analysis')]:
 archive=out/(short+'.native-v2.zip')
 for budget in [8192,16384,32768]:
  req=json.loads((out/f'{short}-{budget}.request.json').read_text())
  cmd=[str(binary),'json',str(archive),str(out/f'{short}.mapping.json')]
  run=subprocess.run(cmd,input=json.dumps(req).encode(),capture_output=True)
  if run.returncode==0:
   assert len(run.stdout)<=budget
   expected=(out/f'{short}-{budget}.payload.json').read_bytes()
   assert run.stdout==expected
  else: assert not run.stdout
  rows.append({'case':short,'budget':budget,'exit':run.returncode,'stdout_bytes':len(run.stdout),'stdout_sha256':hashlib.sha256(run.stdout).hexdigest() if run.stdout else None,'error':run.stderr.decode().strip()})
  if budget==32768:
   for mode in ['strict_time','revoked']:
    changed=json.loads(json.dumps(req))
    if mode=='strict_time':changed['review']['time_mode']='require_declared_interval'
    else:changed['package_revoked']=True
    bad=subprocess.run(cmd,input=json.dumps(changed).encode(),capture_output=True)
    assert bad.returncode!=0 and not bad.stdout
   text=subprocess.run([str(binary),'text',str(archive),str(out/f'{short}.mapping.json')],input=json.dumps(req).encode(),capture_output=True)
   if text.returncode==0:assert len(text.stdout)<=budget
   else:assert not text.stdout
   rows[-1]['text_exit']=text.returncode;rows[-1]['text_stdout_bytes']=len(text.stdout)
(root/'cli-receipts.json').write_text(json.dumps(rows,indent=2)+'\n')
print(json.dumps(rows,indent=2))
