import pathlib,subprocess,json,sys
root=pathlib.Path(__file__).parent
binary=pathlib.Path(sys.argv[1]) if len(sys.argv)>1 else root/'target/debug/examples/compact_review'
rows=[]
for case in ['heat','compute','access']:
 for budget in [8192,16384,32768]:
  for mode in ['json','text']:
   req=(root/f'outputs/{case}-{budget}.request.json').read_bytes()
   p=subprocess.run([str(binary),mode,str(root/f'outputs/{case}.native-v2.zip'),str(root/f'outputs/{case}.mapping.json')],input=req,capture_output=True)
   if budget==8192:assert p.returncode!=0 and not p.stdout
   else:
    assert p.returncode==0 and len(p.stdout)<=budget
    status='omission_only' if budget==16384 else 'selected_evidence'
    assert f'TACITUS_EVIDENCE_STATUS={status} FINDINGS={0 if budget==16384 else 3}' in p.stderr.decode()
    if mode=='json':assert json.loads(p.stdout)['evidence_status']==status
    elif budget==16384:assert p.stdout.startswith(b'OMISSION RECEIPT - NO EVIDENCE SELECTED\n')
    else:assert p.stdout.startswith(b'SELECTED EVIDENCE REVIEW\n')
   rows.append(dict(case=case,budget=budget,mode=mode,exit=p.returncode,bytes=len(p.stdout),diagnostic=p.stderr.decode().strip()))
(root/'status-cli-receipts.json').write_text(json.dumps(rows,indent=2)+'\n');print(len(rows),'CLI checks pass')
