"""Actual CLI ambiguity regressions; errors must not emit partial evidence."""
import json, pathlib, subprocess, tempfile, sys
root=pathlib.Path(__file__).parent
binary=pathlib.Path(sys.argv[1]) if len(sys.argv)>1 else root/'target/debug/examples/compact_review'
out=root/'outputs'
rows=[]
with tempfile.TemporaryDirectory() as t:
 t=pathlib.Path(t)
 for case in ['heat','compute','access']:
  mapping=(out/f'{case}.mapping.json').read_bytes()
  request=(out/f'{case}-32768.request.json').read_bytes()
  for mode in ['valid','duplicate_mapping','duplicate_request','invalid_utf8','deep_request']:
   m=mapping;r=request
   if mode=='duplicate_mapping':m=b'{"format":1,"\\u0066ormat":2}'
   if mode=='duplicate_request':r=b'{"priority":[],"\\u0070riority":[]}'
   if mode=='invalid_utf8':r=b'"\xff"'
   if mode=='deep_request':r=b'['*65+b'0'+b']'*65
   path=t/'mapping.json';path.write_bytes(m)
   p=subprocess.run([str(binary),'json',str(out/f'{case}.native-v2.zip'),str(path)],input=r,capture_output=True)
   if mode=='valid':assert p.returncode==0 and p.stdout==(out/f'{case}-32768.payload.json').read_bytes()
   else:assert p.returncode!=0 and p.stdout==b''
   rows.append(dict(case=case,mode=mode,exit=p.returncode,stdout_bytes=len(p.stdout),error=p.stderr.decode().strip()))
(root/'strict-cli-receipts.json').write_text(json.dumps(rows,indent=2)+'\n')
print(f'{len(rows)} actual CLI cases passed')
