#!/usr/bin/env python3
"""Copy exact public capsule sources into a bounded, repository-locked workspace.
No downloads, application build scripts, credentials, or source changes.
"""
import argparse, hashlib, json, pathlib, shutil, tomllib
ROOT = pathlib.Path(__file__).resolve().parents[1]
CRATES = ('tacitus-core','tacitus-capsules','tacitus-analysis','tacitus-evals')
def prepare(destination):
    destination=pathlib.Path(destination)
    if destination.is_symlink(): raise ValueError("output symlink refused")
    destination=destination.resolve()
    if destination.exists(): raise ValueError('output must not already exist')
    raw=(ROOT/'edgequake/Cargo.lock').read_text()
    repository=tomllib.loads(raw)['package']
    subset_raw=(ROOT/'scripts/capsule-workspace.lock').read_text()
    selected=tomllib.loads(subset_raw)['package']
    identities={(p['name'],p['version'],p.get('source'),p.get('checksum')) for p in repository}
    for p in selected:
        if (p['name'],p['version'],p.get('source'),p.get('checksum')) not in identities:
            raise ValueError('capsule dependency differs from repository lock: '+p['name'])
        if p.get('source') not in (None,'registry+https://github.com/rust-lang/crates.io-index'):
            raise ValueError('unsupported dependency source')
    destination.mkdir(parents=True)
    for crate in CRATES:shutil.copytree(ROOT/'edgequake/crates'/crate,destination/'crates'/crate)
    shutil.copytree(ROOT/'edgequake/authoring-demo',destination/'authoring-demo')
    shutil.copy2(ROOT/'edgequake/check_authoring.py',destination/'check_authoring.py')
    (destination/'Cargo.toml').write_text('[workspace]\nresolver = "2"\nmembers = '+json.dumps(['crates/'+x for x in CRATES])+'\n')
    (destination/'Cargo.lock').write_text(subset_raw)
    inventory={str(p.relative_to(destination)):hashlib.sha256(p.read_bytes()).hexdigest() for p in destination.rglob('*') if p.is_file()}
    receipt={'source_repository':'https://github.com/sargonxg/tacitus-political-policy-second-brain','repository_lock_sha256':hashlib.sha256(raw.encode()).hexdigest(),'locked_packages':len(selected),'source_sha256':inventory,'scope':'Four capsule crates only; full application not built.'}
    (destination/'source-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
    return destination
if __name__=='__main__':
    a=argparse.ArgumentParser();a.add_argument('output');args=a.parse_args()
    print(prepare(args.output))
