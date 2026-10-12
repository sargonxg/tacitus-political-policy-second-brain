"""Emit the provisional JSON package contract. Rust import validation adds semantic checks."""
import json
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
S = {"type": "string"}
I = {"type": "integer"}
V = {"type": "integer", "minimum": 1}
H = {"type": "string", "pattern": "^[a-f0-9]{64}$"}
B = {"type": "boolean"}
STRINGS = {"type": "array", "items": S}

def ref(name):
    return {"$ref": "#/$defs/" + name}

def optional(value):
    return {"anyOf": [value, {"type": "null"}]}

def array(value, maximum=None):
    result = {"type": "array", "items": value}
    if maximum is not None:
        result["maxItems"] = maximum
    return result

def obj(fields):
    return {"type": "object", "additionalProperties": False,
            "properties": fields, "required": list(fields)}

def enum(*values):
    return {"enum": list(values)}

defs = {}
defs["interval"] = obj({"start": I, "end": optional(I)})
defs["span"] = obj({"source": S, "text_hash": H, "start": {"type":"integer","minimum":0},
    "end": {"type":"integer","minimum":1}, "locator": optional(S), "original_language": S,
    "translation": optional(S)})
defs["source"] = obj({"id":S,"version":V,"workspace":S,"title":S,"author":S,
    "original_hash":H,"text_hash":H,"text":{"const":""},"language":S,"extraction_tool":S,
    "published_at":I,"retrieved_at":I,"available":ref("interval"),"valid":ref("interval"),
    "source_family":S,"withdrawn":{"const":False},"restricted":{"const":False},
    "redistributable":B,"license":S})
defs["assertion"] = obj({"id":S,"version":V,"workspace":S,"profile":S,"speaker":S,
    "proposition":S,"subject":S,"object":optional(S),
    "epistemic_type":enum("reported_statement","observation","allegation","inference","hypothesis","forecast","value_judgment","recommendation"),
    "polarity":S,"modality":S,"conditions":STRINGS,"exceptions":STRINGS,"audience":S,"speech_act":S,
    "valid":ref("interval"),"available":ref("interval"),"evidence":array(ref("span")),
    "dependencies":STRINGS,"challenges":STRINGS,"supersedes":optional(S),"confidence":S,"confidence_rationale":S})
defs["review"] = obj({"assertion_id":S,"assertion_fingerprint":H,
    "evidence_hashes":{"type":"object","additionalProperties":H},"reviewer":S,"rationale":S,"reviewed_at":I,
    "decision":enum("accept","reject","caveat"),"origin":enum("local_human","imported_attestation","model_suggestion","synthetic_test")})
defs["method"] = obj({"id":S,"version":V,
    "kind":enum("distinction","cue","heuristic","pattern","exception","red_flag","language_rule","procedure","case","unknown","revision_trigger"),
    "description":S,"scope":S,"failure_conditions":STRINGS,"provenance":STRINGS,"review_state":S})
defs["dependency"] = obj({"id":S,"version":V,"digest":H})
defs["manifest"] = obj({"schema_version":{"const":1},"package_version":V,"id":S,"author":S,
    "title":S,"domain":S,"scope":S,"purpose":S,"profile":enum("context","cognitive_method","output_artifact"),
    "created_at":I,"valid_until":optional(I),"supersedes":optional(S),"dependencies":array(ref("dependency"),16),
    "license":S,"confidentiality":enum("public","private","restricted"),"sharing_policy":S,
    "evidence_mode":enum("references_only","evidence_included"),"limitations":STRINGS,
    "files":{"type":"object","additionalProperties":H}})
defs["data"] = obj({"assertions":array(ref("assertion"),256),"sources":array(ref("source"),32),
    "methods":array(ref("method"),32),"reviews":array(ref("review")),"artifacts":{"type":"object","additionalProperties":S}})
SCHEMA = {"$schema":"https://json-schema.org/draft/2020-12/schema",
    "$id":"urn:tacitus:capsule:v1", "title":"Provisional Tacitus capsule v1",
    **obj({"manifest":ref("manifest"),"data":ref("data")}),"$defs":defs}

if __name__ == "__main__":
    destination = ROOT / "schemas/tacitus/capsule-v1.schema.json"
    expected = (json.dumps(SCHEMA, indent=2, ensure_ascii=False) + "\n").encode()
    if "--check" in sys.argv:
        if not destination.exists() or destination.read_bytes() != expected:
            raise SystemExit("Schema artifact drift: run scripts/generate-tacitus-schema.py")
        print("Schema artifact matches generator; Rust semantic validation remains separate.")
    else:
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(expected)
