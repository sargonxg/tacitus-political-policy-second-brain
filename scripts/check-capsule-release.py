#!/usr/bin/env python3
"""Reproduce capsule developer checks and real/synthetic documents in a NEW directory."""
import argparse, hashlib, importlib.util, json, os, pathlib, subprocess, sys
ROOT=pathlib.Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('prepare',ROOT/'scripts/prepare-capsule-workspace.py');module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
def run(args,cwd,log,input=None,ok=True):
    r=subprocess.run([str(x) for x in args],cwd=cwd,input=input,capture_output=True)
    log.write_bytes(r.stdout+r.stderr)
    if (r.returncode==0)!=ok:raise RuntimeError('unexpected result: '+' '.join(map(str,args))+'; see '+str(log))
    return r
p=argparse.ArgumentParser();p.add_argument('output',help='new directory; never overwrites');p.add_argument('--offline',action='store_true');a=p.parse_args()
out=pathlib.Path(a.output)
if out.is_symlink():raise SystemExit("output symlink refused")
out=out.resolve()
if out.exists():raise SystemExit('output must not exist; choose a new path')
# Check tools before creating outputs. No implicit installs.
for command in [['cargo','+1.95.0','--version'],['node','--version']]:
 r=subprocess.run(command,capture_output=True,check=True);print(r.stdout.decode().strip())
 if command[0]=='node' and int(r.stdout.decode().lstrip('v').split('.')[0])<22:raise SystemExit('Node22+ required')
out.mkdir(parents=True);ws=module.prepare(out/'workspace');logs=out/'logs';logs.mkdir()
run([sys.executable,ROOT/'scripts/test-capsule-workspace.py'],ROOT,logs/'workspace-tests.log')
base=['cargo','+1.95.0'];flags=['--locked']+(['--offline'] if a.offline else [])
run(base+['test',*flags,'--all-targets'],ws,logs/'rust-tests.log')
run(base+['clippy',*flags,'--all-targets','--','-D','warnings'],ws,logs/'clippy.log')
run(base+['fmt','--all','--check'],ws,logs/'fmt.log')
run(base+['build',*flags,'--examples'],ws,logs/'build.log')
node=['tools/tacitus-policy-lab/test.mjs','tools/tacitus-policy-lab/benchmark.test.mjs','tools/tacitus-policy-lab/change-review.test.mjs','tools/tacitus-capsule-catalog/test.mjs','tools/tacitus-capsule-catalog-ui/test.mjs','tools/tacitus-capsule-catalog-ui/security.test.mjs']
run(['node','--test',*node],ROOT,logs/'node-tests.log')
run([sys.executable,'scripts/generate-tacitus-schema.py','--check'],ROOT,logs/'schema.log')
# Force this workspace's binaries even if caller has a custom target directory.
target=pathlib.Path(os.environ.get('CARGO_TARGET_DIR',str(ws/'target')))
if not target.is_absolute(): target=ws/target
bins=target/'debug/examples'
run([sys.executable,ws/'check_authoring.py',bins],ws,logs/'synthetic.log')
real=out/'real-capsules';real.mkdir();cases=[]
for inp in sorted((ws/'crates/tacitus-analysis/tests/fixtures/v2').glob('*.native-v2-input.json')):
 name=inp.name.removesuffix('.native-v2-input.json');archive=real/(name+'.zip');v=json.loads(inp.read_text())
 run([bins/'author_native_v2','--confirm-original-evidence',inp,archive],ws,logs/(name+'-create.log'))
 r=run([bins/'v2_workflow','inspect',archive],ws,logs/(name+'-inspect.log'));(real/(name+'-inspect.json')).write_bytes(r.stdout)
 request={'question':'What do these source-linked authored records say, with their qualifications?','known_as_of':v['manifest']['created_at'],'valid_at':0,'time_mode':'disclose_unknown','max_bytes':200000}
 raw=json.dumps(request).encode();(real/(name+'-request.json')).write_bytes(raw)
 for mode,ext in [('review','json'),('review-text','txt')]:
  r=run([bins/'v2_workflow',mode,archive],ws,logs/(name+'-'+mode+'.log'),input=raw);(real/(name+'-review.'+ext)).write_bytes(r.stdout)
  if mode=='review':assert len(json.loads(r.stdout)['document']['layered_context']['packet']['claims'])==5
 request['time_mode']='require_declared_interval';r=run([bins/'v2_workflow','review',archive],ws,logs/(name+'-strict.log'),input=json.dumps(request).encode(),ok=False);assert not r.stdout
 cases.append({'case':name,'archive_sha256':hashlib.sha256(archive.read_bytes()).hexdigest(),'claims':5,'strict_date_refused':True})
compact=out/'compact';r=run([bins/'compact_bench',compact],ws,logs/'compact.log');(out/'compact-results.json').write_bytes(r.stdout)
receipt={'status':'capsule developer workflow passed','real_cases':cases,'synthetic':json.loads((ws/'evidence/authoring-receipt.json').read_text()),'scope':'Local capsule/Node developer tools only. No full application, hosted marketplace, model, DB, browser accessibility or live PRAXIS acceptance.'}
(out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(receipt,indent=2))
