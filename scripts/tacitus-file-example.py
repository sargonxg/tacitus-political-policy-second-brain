"""Real local file round-trip of intentional synthetic data; no model/API/UI acceptance."""
import datetime
import hashlib
import json
import pathlib
import subprocess
import sys
import zipfile
import runpy

SCHEMA = runpy.run_path(str(pathlib.Path(__file__).with_name("generate-tacitus-schema.py")))["SCHEMA"]

ROOT = pathlib.Path(__file__).resolve().parent.parent

def write(path, value):
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")

def main():
    # Optional developer-only schema validator, not an application dependency.
    import jsonschema
    jsonschema.Draft202012Validator.check_schema(SCHEMA)
    root = ROOT / ".tacitus" / "file-example"
    root.mkdir(parents=True, exist_ok=True)
    evidence_root = root / "selected"
    (evidence_root / "evidence").mkdir(parents=True, exist_ok=True)
    fixture = json.loads((ROOT / "fixtures/tacitus/public-synthetic.json").read_text(encoding="utf-8"))
    assert fixture["fictional"] is True and sum(len(d["documents"]) for d in fixture["dossiers"]) == 15
    selected = [d for d in fixture["dossiers"][0]["documents"] if d["id"] in {"alder-statement", "authority", "budget"}]
    sources = []
    for doc in selected:
        text = doc["text"].encode("utf-8")
        sha = hashlib.sha256(text).hexdigest()
        timestamp = int(datetime.datetime.fromisoformat(doc["published"]).replace(tzinfo=datetime.timezone.utc).timestamp())
        source = dict(id=doc["id"], version=1, workspace="synthetic-original", title=doc["title"], author=doc["author"],
            original_hash=sha, text_hash=sha, text="", language=doc["language"], extraction_tool="selected-synthetic-utf8/1",
            published_at=timestamp, retrieved_at=timestamp, available=dict(start=timestamp,end=None), valid=dict(start=timestamp,end=None),
            source_family=doc["family"], withdrawn=False, restricted=False, redistributable=True, license="CC0-1.0")
        name = hashlib.sha256((doc["id"]+"@1").encode()).hexdigest()+".txt"
        (evidence_root / "evidence" / name).write_bytes(text)
        sources.append(source)
    spans = [dict(source=s["id"]+"@1",text_hash=s["text_hash"],start=0,end=len(d["text"].encode()),
                  locator="paragraph 1",original_language=s["language"],translation=None) for s,d in zip(sources,selected)]
    assertion = dict(id="synthetic-conditional-participation",version=1,workspace="synthetic-original",profile="position",
        speaker=sources[0]["author"],proposition="The delegation stated conditional willingness to discuss, not a participation commitment.",
        subject="Fictional Alder delegation",object="fictional pilot",epistemic_type="reported_statement",polarity="positive",
        modality="may consider",conditions=["Council approval", "funded implementation plan"],exceptions=[],audience="public",
        speech_act="statement",valid=sources[0]["valid"],available=sources[0]["available"],evidence=[spans[0]],dependencies=[],challenges=[],
        supersedes=None,confidence="wording explicit",confidence_rationale="Establishes attributed statement only; not implementation.")
    data = dict(assertions=[assertion],sources=sources,methods=[],reviews=[],artifacts={})
    manifest = dict(schema_version=1,package_version=1,id="fictional-pilot-context",author="Synthetic example author",
        title="Fictional conditional pilot",domain="policy",scope="fictional pilot",purpose="developer file regression",profile="context",
        created_at=1768435200,valid_until=None,supersedes=None,dependencies=[],license="CC0-1.0",confidentiality="public",
        sharing_policy="Intentional public synthetic fixtures only",evidence_mode="evidence_included",limitations=[],files={})
    jsonschema.validate(dict(manifest=manifest,data=data),SCHEMA)
    rejected = json.loads(json.dumps(dict(manifest=manifest,data=data)))
    rejected["manifest"]["schema_version"] = 2
    try:
        jsonschema.validate(rejected, SCHEMA)
    except jsonschema.ValidationError:
        pass
    else:
        raise AssertionError("Unsupported schema accepted")
    write(root / "manifest.json",manifest)
    write(root / "data.json",data)
    contract = dict(workspace="synthetic-fresh",question="Has the delegation committed and can the pilot proceed?",
        decision_maker="fictional Council",audience="analysts",mandate="pilot approval",objectives=["feasible pilot"],
        constraints=["appropriation and written consent"],time_horizon="one month",known_as_of=1768435200,valid_at=1768435200,
        selected_sources=[s["id"]+"@1" for s in sources],selected_capsules=[],permitted_tools=[],output_format="brief",
        context_tokens=8192,output_tokens=512,overhead_tokens=1024)
    write(root / "contract.json",contract)
    base = ["cargo","run","--quiet","--offline","--locked","--manifest-path",str(ROOT / "edgequake/Cargo.toml"),"-p","tacitus-capsules","--"]
    # Unique output paths; the CLI never overwrites an existing bundle or packet.
    nonce = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%S%f")
    bundle = root / (nonce+".capsule")
    packet_path = root / (nonce+"-packet.json")
    subprocess.run(base+["preview",str(root/"manifest.json"),str(root/"data.json"),str(evidence_root)],check=True,stdout=subprocess.DEVNULL)
    subprocess.run(base+["pack",str(root/"manifest.json"),str(root/"data.json"),str(evidence_root),str(bundle)],check=True)
    subprocess.run(base+["validate",str(bundle)],check=True)
    subprocess.run(base+["compile",str(bundle),"synthetic-fresh",str(root/"contract.json"),str(packet_path)],check=True)
    with zipfile.ZipFile(bundle) as z:
        exported = dict(manifest=json.loads(z.read("manifest.json")),data=json.loads(z.read("capsule.json")))
        jsonschema.validate(exported,SCHEMA)
        assert exported["data"]["assertions"][0]["conditions"] == assertion["conditions"]
    packet = json.loads(packet_path.read_text())
    assert len(packet["sources"]) == 3 and len(packet["citations"]) == 3
    assert any("may consider" in text for text in packet["sources"].values())
    assert any("no payment" in text.lower() for text in packet["sources"].values())
    assert any("lexical retrieval ONLY" in warning for warning in packet["warnings"])
    print("PASS: 15 fictional documents; 3-source file export/import/compile; conditions/text/schema/citations preserved.")
    print("No local inference, embeddings, graph retrieval, browser, API or persistence test ran.")
    print("Artifacts: " + str(root))

if __name__ == "__main__":
    main()
