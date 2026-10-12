use std::collections::{BTreeMap, BTreeSet};
use std::io::{Cursor, Write};
use tacitus_capsules::{
    compiler::{compile, Candidate, Retrieval},
    evidence_name, CapsuleData, Manifest, Package,
};
use tacitus_core::*;
use zip::{write::SimpleFileOptions, ZipWriter};

fn source() -> SourceVersion {
    let text = "The fictional Alder delegation may discuss participation, subject to Council approval. Budget authorization is not expenditure.".to_string();
    SourceVersion {
        id: "alder-statement".into(),
        version: 1,
        workspace: "first".into(),
        title: "Synthetic conditional statement".into(),
        author: "Fictional delegation".into(),
        original_hash: digest(text.as_bytes()),
        text_hash: digest(text.as_bytes()),
        text,
        language: "en".into(),
        extraction_tool: "synthetic-utf8/1".into(),
        published_at: 10,
        retrieved_at: 12,
        available: Interval {
            start: 12,
            end: None,
        },
        valid: Interval {
            start: 10,
            end: None,
        },
        source_family: "alder-original".into(),
        withdrawn: false,
        restricted: false,
        redistributable: true,
        license: "CC0-1.0".into(),
    }
}
fn assertion(s: &SourceVersion) -> Assertion {
    Assertion {
        id: "claim-1".into(),
        version: 1,
        workspace: s.workspace.clone(),
        profile: "position".into(),
        speaker: s.author.clone(),
        proposition: "Delegation expressed conditional willingness to discuss.".into(),
        subject: "Alder delegation".into(),
        object: None,
        epistemic_type: EpistemicType::ReportedStatement,
        polarity: "positive".into(),
        modality: "may".into(),
        conditions: vec!["Council approval".into()],
        exceptions: vec![],
        audience: "public".into(),
        speech_act: "statement".into(),
        valid: s.valid.clone(),
        available: s.available.clone(),
        evidence: vec![Span {
            source: s.key(),
            text_hash: s.text_hash.clone(),
            start: 0,
            end: s.text.len(),
            locator: Some("paragraph 1".into()),
            original_language: "en".into(),
            translation: None,
        }],
        dependencies: vec![],
        challenges: vec![],
        supersedes: None,
        confidence: "source wording explicit".into(),
        confidence_rationale: "Establishes statement, not delivered participation.".into(),
    }
}
fn package() -> Package {
    let s = source();
    let a = assertion(&s);
    let review = ReviewReceipt {
        assertion_id: a.id.clone(),
        assertion_fingerprint: a.fingerprint(),
        evidence_hashes: BTreeMap::from([(s.key(), s.text_hash.clone())]),
        reviewer: "synthetic-test-reviewer".into(),
        rationale: "Test action only".into(),
        reviewed_at: 20,
        decision: ReviewDecision::Caveat,
        origin: ReviewOrigin::SyntheticTest,
    };
    let evidence = BTreeMap::from([(evidence_name(&s), s.text.as_bytes().to_vec())]);
    let mut metadata = s;
    metadata.text.clear();
    Package {
        manifest: Manifest {
            schema_version: 1,
            package_version: 1,
            id: "synthetic-capsule".into(),
            author: "Fictional analyst".into(),
            title: "Conditional participation".into(),
            domain: "policy".into(),
            scope: "fictional pilot".into(),
            purpose: "regression".into(),
            profile: "context".into(),
            created_at: 20,
            valid_until: None,
            supersedes: None,
            dependencies: vec![],
            license: "CC0-1.0".into(),
            confidentiality: "public".into(),
            sharing_policy: "intentional synthetic test export".into(),
            evidence_mode: "evidence_included".into(),
            limitations: vec![],
            files: BTreeMap::new(),
        },
        data: CapsuleData {
            assertions: vec![a],
            sources: vec![metadata],
            methods: vec![],
            reviews: vec![review],
            artifacts: BTreeMap::from([(
                "brief".into(),
                "Participation is conditional; appropriation is unresolved.".into(),
            )]),
        },
        evidence,
    }
}
fn contract(workspace: &str) -> DecisionContract {
    DecisionContract {
        workspace: workspace.into(),
        question: "Has the delegation committed?".into(),
        decision_maker: "fictional council".into(),
        audience: "analysts".into(),
        mandate: "pilot approval".into(),
        objectives: vec!["feasible pilot".into()],
        constraints: vec!["funding".into()],
        time_horizon: "one month".into(),
        known_as_of: 20,
        valid_at: 20,
        selected_sources: BTreeSet::from(["alder-statement@1".into()]),
        selected_capsules: BTreeSet::new(),
        permitted_tools: BTreeSet::new(),
        output_format: "brief".into(),
        context_tokens: 8192,
        output_tokens: 512,
        overhead_tokens: 1024,
    }
}
struct Allowed;
impl Retrieval for Allowed {
    fn retrieve(&self, _: &str, allowed: &BTreeSet<String>) -> Result<Vec<Candidate>> {
        Ok(allowed
            .iter()
            .map(|s| Candidate {
                source: s.clone(),
                semantic_relevance: 0.5,
            })
            .collect())
    }
}
#[test]
fn selected_text_ingestion_preserves_bytes_hashes_and_rejects_malformed() {
    let bytes = "SYNTHETIC. élan\r\nNo committed participation."
        .as_bytes()
        .to_vec();
    let doc = SelectedTextDocument::ingest(bytes.clone(), source()).unwrap();
    assert_eq!(doc.original, bytes);
    assert_eq!(doc.source.original_hash, digest(&bytes));
    assert_eq!(doc.source.text_hash, digest(doc.source.text.as_bytes()));
    assert!(doc.source.text.contains("\r\n"));
    assert!(SelectedTextDocument::ingest(vec![0xff], source()).is_err());
    assert!(SelectedTextDocument::ingest(b" \n".to_vec(), source()).is_err());
}
#[test]
fn substantive_fresh_workspace_roundtrip() {
    let p = package();
    let bytes = p.export().unwrap();
    assert_eq!(
        bytes,
        p.export().unwrap(),
        "serialization/ZIP must be deterministic"
    );
    let imported = Package::import(&bytes).unwrap();
    assert_eq!(
        imported.data.assertions[0].conditions,
        vec!["Council approval"]
    );
    assert_eq!(imported.data.artifacts, p.data.artifacts);
    assert_eq!(
        imported.data.reviews[0].origin,
        ReviewOrigin::ImportedAttestation
    );
    let sources = imported.sources_for_workspace("fresh").unwrap();
    let proposals = imported.proposals_for_workspace("fresh").unwrap();
    assert_eq!(proposals[0].workspace, "fresh");
    assert!(!imported.data.reviews[0].is_current_human_review(&proposals[0], &sources));
    let packet = compile(&contract("fresh"), &sources, &Allowed).unwrap();
    assert!(packet.sources["alder-statement@1"].contains("may discuss"));
    assert_eq!(
        packet.citations["E1"]
            .passage(&sources["alder-statement@1"])
            .unwrap(),
        source().text
    );
}
#[test]
fn references_only_cannot_fabricate_evidence() {
    let mut p = package();
    p.evidence.clear();
    p.manifest.evidence_mode = "references_only".into();
    assert!(p.export().is_err());
    p.manifest
        .limitations
        .push("Evidence unavailable; citation entailment cannot be verified.".into());
    let imported = Package::import(&p.export().unwrap()).unwrap();
    assert!(imported.sources_for_workspace("fresh").is_err());
}
#[test]
fn disclosure_rights_and_withdrawal_are_enforced() {
    for change in 0..4 {
        let mut p = package();
        match change {
            0 => p.data.sources[0].redistributable = false,
            1 => p.data.sources[0].restricted = true,
            2 => p.data.sources[0].withdrawn = true,
            _ => p.data.sources[0].text = "hidden evidence in metadata".into(),
        }
        assert!(p.export().is_err());
    }
}
fn archive_entry(name: &str, content: &[u8]) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    writer
        .start_file(name, SimpleFileOptions::default())
        .unwrap();
    writer.write_all(content).unwrap();
    writer.finish().unwrap().into_inner()
}
#[test]
fn archive_traversal_and_executables_rejected() {
    for name in [
        "../stolen",
        "/absolute",
        "C:\\escape",
        "evidence/../../escape.txt",
        "evidence/run.js",
        "evidence/run.sh",
        "evidence/a\\b.txt",
    ] {
        assert!(
            Package::import(&archive_entry(name, b"malicious")).is_err(),
            "{name}"
        );
    }
}
#[test]
fn symlink_and_oversized_entries_rejected() {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    writer
        .add_symlink(
            "evidence/link.txt",
            "../../outside",
            SimpleFileOptions::default(),
        )
        .unwrap();
    assert!(Package::import(&writer.finish().unwrap().into_inner()).is_err());
    assert!(Package::import(&archive_entry(
        "evidence/large.txt",
        &vec![0; 2 * 1024 * 1024 + 1]
    ))
    .is_err());
}
#[test]
fn hash_mismatch_rejected() {
    let p = package();
    let mut files = p.preview().unwrap();
    files.insert("capsule.json".into(), b"{}".to_vec());
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, content) in files {
        writer
            .start_file(name, SimpleFileOptions::default())
            .unwrap();
        writer.write_all(&content).unwrap();
    }
    assert!(Package::import(&writer.finish().unwrap().into_inner()).is_err());
}
#[test]
fn duplicate_central_directory_names_cannot_be_silently_collapsed() {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for name in ["evidence/a.txt", "evidence/b.txt"] {
        writer
            .start_file(name, SimpleFileOptions::default())
            .unwrap();
        writer.write_all(b"synthetic").unwrap();
    }
    let mut bytes = writer.finish().unwrap().into_inner();
    let needle = b"evidence/b.txt";
    for i in 0..=bytes.len() - needle.len() {
        if &bytes[i..i + needle.len()] == needle {
            bytes[i + 9] = b'a';
        }
    }
    let error = Package::import(&bytes).unwrap_err();
    assert!(error.contains("duplicate central-directory"), "{error}");
}
#[test]
fn unsupported_schema_and_executable_method_fields_rejected() {
    let mut p = package();
    p.manifest.schema_version = 2;
    assert!(p.export().is_err());
    assert!(serde_json::from_str::<tacitus_capsules::Method>(r#"{"id":"x","version":1,"kind":"procedure","description":"x","scope":"x","failure_conditions":["x"],"provenance":["x"],"review_state":"untrusted","javascript":"evil()"}"#).is_err());
}
#[test]
fn pre_and_post_retrieval_scope_and_dates() {
    let s = source();
    let mut sources = BTreeMap::from([(s.key(), s)]);
    for change in 0..5 {
        let mut modified = sources.clone();
        let s = modified.values_mut().next().unwrap();
        match change {
            0 => s.workspace = "other".into(),
            1 => s.withdrawn = true,
            2 => s.restricted = true,
            3 => s.available.start = 100,
            _ => s.valid.end = Some(15),
        }
        assert!(compile(&contract("first"), &modified, &Allowed).is_err());
    }
    struct Leaky;
    impl Retrieval for Leaky {
        fn retrieve(&self, _: &str, allowed: &BTreeSet<String>) -> Result<Vec<Candidate>> {
            assert!(!allowed.contains("secret@1"));
            Ok(vec![Candidate {
                source: "secret@1".into(),
                semantic_relevance: 1.0,
            }])
        }
    }
    let mut private = source();
    private.id = "secret".into();
    private.workspace = "other".into();
    sources.insert(private.key(), private);
    assert!(compile(&contract("first"), &sources, &Leaky).is_err());
    let mut c = contract("first");
    c.known_as_of = 11;
    assert!(compile(&c, &sources, &Allowed).is_err());
}
#[test]
fn small_budget_is_actionable_and_never_truncates() {
    let sources = BTreeMap::from([(source().key(), source())]);
    let mut c = contract("first");
    c.context_tokens = c.output_tokens + c.overhead_tokens + 10;
    let error = compile(&c, &sources, &Allowed).unwrap_err();
    assert!(error.contains("not truncated"));
    c.context_tokens = c.output_tokens;
    assert!(compile(&c, &sources, &Allowed).is_err());
}
#[test]
fn byte_offsets_and_stale_human_review() {
    let s = source();
    let sources = BTreeMap::from([(s.key(), s.clone())]);
    let mut a = assertion(&s);
    let receipt = ReviewReceipt {
        assertion_id: a.id.clone(),
        assertion_fingerprint: a.fingerprint(),
        evidence_hashes: BTreeMap::from([(s.key(), s.text_hash.clone())]),
        reviewer: "test-only-authenticated-actor".into(),
        rationale: "test action".into(),
        reviewed_at: 20,
        decision: ReviewDecision::Caveat,
        origin: ReviewOrigin::LocalHuman,
    };
    assert!(receipt.is_current_human_review(&a, &sources));
    a.proposition.push_str(" Changed.");
    assert!(!receipt.is_current_human_review(&a, &sources));
    let mut unicode = source();
    unicode.text = "élan".into();
    unicode.text_hash = digest(unicode.text.as_bytes());
    let span = Span {
        source: unicode.key(),
        text_hash: unicode.text_hash.clone(),
        start: 1,
        end: 3,
        locator: None,
        original_language: "en".into(),
        translation: None,
    };
    assert!(span.passage(&unicode).is_err());
}

fn method() -> tacitus_capsules::Method {
    tacitus_capsules::Method {
        id: "conditional-language".into(),
        version: 1,
        kind: "distinction".into(),
        description: "Preserve Council approval as a prerequisite.".into(),
        scope: "fictional pilot".into(),
        failure_conditions: vec!["Changed authority".into()],
        provenance: vec![source().key()],
        review_state: "accepted".into(),
    }
}

#[test]
fn imported_method_cannot_retain_acceptance() {
    let mut p = package();
    p.data.methods.push(method());
    let imported = Package::import(&p.export().unwrap()).unwrap();
    assert_eq!(imported.data.methods[0].review_state, "unreviewed_proposal");
    assert_eq!(imported.data.methods[0].description, method().description);
    assert_eq!(imported.data.methods[0].provenance, method().provenance);
}

#[test]
fn method_provenance_and_identity_are_checked() {
    for mutation in 0..4 {
        let mut p = package();
        let mut m = method();
        match mutation {
            0 => m.provenance = vec!["missing@1".into()],
            1 => m.description.clear(),
            2 => m.failure_conditions = vec![" ".into()],
            _ => p.data.methods.push(m.clone()),
        }
        p.data.methods.push(m);
        assert!(
            p.export().is_err(),
            "accepted invalid method case {mutation}"
        );
    }
}

#[test]
fn capsule_rejects_cross_workspace_assertion_even_without_bodies() {
    for references_only in [false, true] {
        let mut p = package();
        p.data.assertions[0].workspace = "unauthorized".into();
        if references_only {
            p.manifest.evidence_mode = "references_only".into();
            p.manifest.limitations.push("Bodies unavailable".into());
            p.evidence.clear();
        }
        assert!(p.export().is_err());
    }
}
