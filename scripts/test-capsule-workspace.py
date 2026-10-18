#!/usr/bin/env python3
"""Regression checks for isolation and pinned dependency identity."""
import importlib.util, json, pathlib, subprocess, sys, tempfile, unittest
ROOT=pathlib.Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('prepare',ROOT/'scripts/prepare-capsule-workspace.py');p=importlib.util.module_from_spec(spec);spec.loader.exec_module(p)
class Preparation(unittest.TestCase):
 def test_existing_output_is_preserved(self):
  with tempfile.TemporaryDirectory() as d:
   marker=pathlib.Path(d)/'keep';marker.write_text('user content')
   with self.assertRaises(ValueError):p.prepare(d)
   self.assertEqual(marker.read_text(),'user content')
 def test_dangling_output_symlink_is_refused(self):
  with tempfile.TemporaryDirectory() as d:
   link=pathlib.Path(d)/'link';target=pathlib.Path(d)/'absent';link.symlink_to(target)
   with self.assertRaises(ValueError):p.prepare(link)
   result=subprocess.run([sys.executable,str(ROOT/'scripts/check-capsule-release.py'),str(link)],capture_output=True)
   self.assertNotEqual(result.returncode,0);self.assertIn(b'output symlink refused',result.stderr)
   self.assertTrue(link.is_symlink());self.assertFalse(target.exists())
 def test_exact_sources_and_repository_pins(self):
  with tempfile.TemporaryDirectory() as d:
   out=p.prepare(pathlib.Path(d)/'workspace');receipt=json.loads((out/'source-receipt.json').read_text())
   self.assertEqual(receipt['locked_packages'],32)
   for path in receipt['source_sha256']:
    if path.startswith('crates/'):
     self.assertEqual((out/path).read_bytes(),(ROOT/'edgequake'/path).read_bytes())
 def test_changed_lock_identity_fails_before_output(self):
  with tempfile.TemporaryDirectory() as d:
   fake=pathlib.Path(d)/'repo';(fake/'edgequake').mkdir(parents=True);(fake/'scripts').mkdir()
   (fake/'edgequake/Cargo.lock').write_bytes((ROOT/'edgequake/Cargo.lock').read_bytes())
   raw=(ROOT/'scripts/capsule-workspace.lock').read_text();raw=raw.replace('version = "1.4.2"','version = "999.0.0"',1)
   self.assertNotEqual(raw,(ROOT/'scripts/capsule-workspace.lock').read_text())
   (fake/'scripts/capsule-workspace.lock').write_text(raw)
   old=p.ROOT;p.ROOT=fake
   try:
    with self.assertRaises(ValueError):p.prepare(pathlib.Path(d)/'output')
    self.assertFalse((pathlib.Path(d)/'output').exists())
   finally:p.ROOT=old
if __name__=='__main__':unittest.main()
