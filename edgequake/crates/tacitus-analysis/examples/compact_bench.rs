mod compact_support;
use serde_json::json;
use tacitus_analysis::{compact_brief::*, v2_workflow as v2};
fn main() {
    let mut rows = vec![];
    let output = std::env::args().nth(1).map(std::path::PathBuf::from);
    if let Some(path) = &output {
        std::fs::create_dir_all(path).unwrap();
    }
    for (name, raw) in [
        (
            "heat",
            include_str!("../tests/fixtures/v2/nyc-heat-resilience.native-v2-input.json"),
        ),
        (
            "compute",
            include_str!(
                "../tests/fixtures/v2/compute-infrastructure-governance.native-v2-input.json"
            ),
        ),
        (
            "access",
            include_str!("../tests/fixtures/v2/humanitarian-access-analysis.native-v2-input.json"),
        ),
    ] {
        let p = compact_support::package(raw);
        let archive = p.export().unwrap();
        let m = compact_support::mapping(&p, &archive);
        if let Some(path) = &output {
            std::fs::write(path.join(format!("{name}.native-v2.zip")), &archive).unwrap();
            std::fs::write(
                path.join(format!("{name}.mapping.json")),
                serde_json::to_vec_pretty(&m).unwrap(),
            )
            .unwrap();
        }
        for budget in [8192, 16384, 32768] {
            let request = Request {
                review: v2::Request {
                    question: "Which evidence can be inspected within this byte budget?".into(),
                    known_as_of: p.manifest.created_at,
                    valid_at: 0,
                    time_mode: v2::TimeMode::DiscloseUnknown,
                    max_bytes: budget,
                },
                priority: p.data.assertions.iter().map(|a| a.id.clone()).collect(),
                revoked_sources: Default::default(),
                package_revoked: false,
            };
            if let Some(path) = &output {
                std::fs::write(
                    path.join(format!("{name}-{budget}.request.json")),
                    serde_json::to_vec_pretty(&request).unwrap(),
                )
                .unwrap();
            }
            let mut c = v2::LocalCatalog::default();
            let d = c.import_native_v2(&archive).unwrap();
            let baseline = c.export_review(&d.archive_digest, &request.review);
            let compact = compile(&archive, &m, &request);
            if let Ok(x) = &compact {
                assert!(x.bytes <= budget);
                assert_eq!(x.digest, compile(&archive, &m, &request).unwrap().digest);
                if let Some(path) = &output {
                    std::fs::write(
                        path.join(format!("{name}-{budget}.inspection-envelope.json")),
                        serde_json::to_vec_pretty(x).unwrap(),
                    )
                    .unwrap();
                    std::fs::write(
                        path.join(format!("{name}-{budget}.payload.json")),
                        serde_json::to_vec(&x.document).unwrap(),
                    )
                    .unwrap();
                    if let Ok(text) = render_text(x, budget) {
                        std::fs::write(path.join(format!("{name}-{budget}.review.txt")), text)
                            .unwrap();
                    }
                }
            }
            rows.push(json!({"case":name,"budget":budget,"atomic_baseline_claims":baseline.as_ref().map(|x|x.document.layered_context.packet.claims.len()).unwrap_or(0),"atomic_baseline_error":baseline.err(),"compact_claims":compact.as_ref().map(|x|x.document.claims.len()).unwrap_or(0),"compact_bytes":compact.as_ref().map(|x|x.bytes).ok(),"same_budget_text_accepted":compact.as_ref().map(|x|render_text(x,budget).is_ok()).unwrap_or(false),"rendered_text_bytes":compact.as_ref().map(|x|render_text(x,100000).unwrap().len()).ok(),"compact_claim_ids":compact.as_ref().map(|x|x.document.claims.keys().collect::<Vec<_>>()).ok(),"compact_digest":compact.as_ref().map(|x|x.digest.clone()).ok(),"compact_error":compact.err()}));
        }
    }
    println!("{}", serde_json::to_string_pretty(&rows).unwrap());
}
