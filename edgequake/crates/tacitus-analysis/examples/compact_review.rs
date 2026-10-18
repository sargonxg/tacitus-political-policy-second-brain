use std::{
    fs,
    io::{self, Read},
};
use tacitus_analysis::compact_brief::{
    compile, evidence_status, render_text, EvidenceStatus, Mapping, Request,
};
fn read(path: &str, max: u64) -> Result<Vec<u8>, String> {
    let mut b = vec![];
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(max + 1)
        .read_to_end(&mut b)
        .map_err(|e| e.to_string())?;
    if b.len() as u64 > max {
        return Err("file bound exceeded".into());
    }
    Ok(b)
}
fn run() -> Result<(), String> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 4 || !["json", "text"].contains(&args[1].as_str()) {
        return Err("usage: compact_review json|text ARCHIVE MAPPING < request.json".into());
    }
    let archive = read(&args[2], 8 * 1024 * 1024)?;
    let mapping: Mapping = tacitus_capsules::strict_json::from_slice(&read(&args[3], 1000000)?)
        .map_err(|e| e.to_string())?;
    let mut raw = vec![];
    io::stdin()
        .take(65537)
        .read_to_end(&mut raw)
        .map_err(|e| e.to_string())?;
    if raw.len() > 65536 {
        return Err("request bound exceeded".into());
    }
    let r: Request = tacitus_capsules::strict_json::from_slice(&raw).map_err(|e| e.to_string())?;
    let x = compile(&archive, &mapping, &r)?;
    let status = match evidence_status(&x.document)? {
        EvidenceStatus::OmissionOnly => "omission_only",
        EvidenceStatus::SelectedEvidence => "selected_evidence",
    };
    if args[1] == "json" {
        let payload = serde_json::to_vec(&x.document).map_err(|e| e.to_string())?;
        if payload.len() > r.review.max_bytes {
            return Err("emitted payload exceeds budget".into());
        }
        eprintln!(
            "TACITUS_EVIDENCE_STATUS={status} FINDINGS={}",
            x.document.claims.len()
        );
        use std::io::Write;
        io::stdout()
            .write_all(&payload)
            .map_err(|e| e.to_string())?;
    } else {
        let text = render_text(&x, r.review.max_bytes)?;
        eprintln!(
            "TACITUS_EVIDENCE_STATUS={status} FINDINGS={}",
            x.document.claims.len()
        );
        print!("{text}")
    }
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1)
    }
}
