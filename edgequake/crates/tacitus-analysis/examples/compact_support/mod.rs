use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use tacitus_analysis::{compact_brief::*, layers};
use tacitus_capsules::policy::PolicyPackage;
use tacitus_core::digest;
pub fn package(raw: &str) -> PolicyPackage {
    let v: Value = serde_json::from_str(raw).unwrap();
    PolicyPackage {
        manifest: serde_json::from_value(v["manifest"].clone()).unwrap(),
        data: serde_json::from_value(v["data"].clone()).unwrap(),
        evidence: v["evidence"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.as_str().unwrap().as_bytes().to_vec()))
            .collect(),
    }
}
pub fn mapping(p: &PolicyPackage, archive: &[u8]) -> Mapping {
    let fingerprints = p
        .data
        .assertions
        .iter()
        .map(|a| (a.id.clone(), a.fingerprint()))
        .collect::<BTreeMap<_, _>>();
    let mut items = vec![];
    for (key, text) in &p.data.artifacts {
        let value: Value = serde_json::from_str(text).unwrap();
        if key == "tacitus.native-adaptation" {
            items.push(Item {
                artifact: key.clone(),
                pointer: "".into(),
                value_hash: digest(&serde_json::to_vec(&value).unwrap()),
                role: Role::Global,
                claims: BTreeSet::new(),
                reason: "Native representation caveats always accompany evidence".into(),
            });
            continue;
        }
        for (field, v) in value.as_object().unwrap() {
            let pointer = format!("/{}", field.replace('~', "~0").replace('/', "~1"));
            if field == "sources" {
                for (i, source) in v.as_array().unwrap().iter().enumerate() {
                    let id = source["id"].as_str().unwrap();
                    let claims = value["claims"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .filter(|c| {
                            c["sourceIds"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .any(|s| s.as_str() == Some(id))
                        })
                        .map(|c| c["id"].as_str().unwrap().to_string())
                        .collect::<BTreeSet<_>>();
                    items.push(Item{artifact:key.clone(),pointer:format!("{pointer}/{i}"),value_hash:digest(&serde_json::to_vec(source).unwrap()),role:if claims.is_empty(){Role::Supplement}else{Role::Attribution},claims,reason:"Official source title, locator, publisher date and rights accompany every referencing claim".into()});
                }
            } else if field == "claims" || field == "summaryRecords" {
                for (i, v) in v.as_array().unwrap().iter().enumerate() {
                    let id = v[if field == "claims" { "id" } else { "claimId" }]
                        .as_str()
                        .unwrap();
                    items.push(Item{artifact:key.clone(),pointer:format!("{pointer}/{i}"),value_hash:digest(&serde_json::to_vec(v).unwrap()),role:Role::Claim,claims:BTreeSet::from([id.into()]),reason:"Exact authored claim record and all qualifications accompany the linked assertion".into()});
                }
            } else {
                let global = [
                    "limitations",
                    "rightsNote",
                    "contextPolicy",
                    "temporalContract",
                    "reviewStatus",
                    "constraints",
                    "updates",
                ]
                .contains(&field.as_str());
                items.push(Item{artifact:key.clone(),pointer,value_hash:digest(&serde_json::to_vec(v).unwrap()),role:if global{Role::Global}else{Role::Supplement},claims:BTreeSet::new(),reason:if global{"Global limits, rights and temporal interpretation"}else{"Supplemental dossier context is explicitly not included in this selected-evidence view"}.into()});
            }
        }
    }
    let research: Value =
        serde_json::from_str(&p.data.artifacts["tacitus.policy-research"]).unwrap();
    let mut links = vec![];
    for claim in research["claims"].as_array().unwrap() {
        for dep in claim["analyticalDependencies"].as_array().unwrap() {
            links.push(layers::Link {
                from: claim["id"].as_str().unwrap().into(),
                to: dep.as_str().unwrap().into(),
                kind: layers::LinkKind::DependsOn,
                rationale:
                    "Explicit authored analytical dependency, preserved as a closed evidence group"
                        .into(),
            });
        }
    }
    Mapping {
        profile_version: "tacitus-policy-research-r2/1".into(),
        format: "tacitus-scoped-artifacts/1".into(),
        archive_digest: digest(archive),
        available_at: p.manifest.created_at,
        artifact_hashes: p
            .data
            .artifacts
            .iter()
            .map(|(k, v)| (k.clone(), digest(v.as_bytes())))
            .collect(),
        claim_fingerprints: fingerprints.clone(),
        items,
        graph: layers::Graph {
            version: "tacitus-policy-graph-sidecar/1".into(),
            available_at: p.manifest.created_at,
            claim_fingerprints: fingerprints,
            nodes: BTreeMap::new(),
            links,
            knowledge: BTreeMap::new(),
        },
    }
}
