//! Developer file tools, not the application launcher. Never runs a model or downloads URLs.
use std::{collections::BTreeSet, fs, path::Path};
use tacitus_capsules::{
    compiler::{compile, Candidate, Retrieval},
    evidence_name, CapsuleData, Manifest, Package,
};
use tacitus_core::{DecisionContract, Result};

struct LexicalOnly;
impl Retrieval for LexicalOnly {
    fn retrieve(&self, _: &str, _: &BTreeSet<String>) -> Result<Vec<Candidate>> {
        Ok(vec![])
    }
}
fn read(path: &str, max: u64) -> Result<Vec<u8>> {
    let metadata = fs::metadata(path).map_err(|e| e.to_string())?;
    if metadata.len() > max {
        return Err("file exceeds input bound".into());
    }
    use std::io::Read;
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(max + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > max {
        return Err("file grew past input bound".into());
    }
    Ok(bytes)
}
fn write_new(path: &str, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    file.write_all(bytes).map_err(|e| e.to_string())
}
fn selected_package(args: &[String]) -> Result<Package> {
    let manifest: Manifest =
        serde_json::from_slice(&read(&args[2], 2 * 1024 * 1024)?).map_err(|e| e.to_string())?;
    let data: CapsuleData =
        serde_json::from_slice(&read(&args[3], 2 * 1024 * 1024)?).map_err(|e| e.to_string())?;
    let mut evidence = std::collections::BTreeMap::new();
    if manifest.evidence_mode == "evidence_included" {
        for source in &data.sources {
            let name = evidence_name(source);
            let path = Path::new(&args[4]).join(&name);
            let bytes = read(
                path.to_str().ok_or("invalid selected path")?,
                2 * 1024 * 1024,
            )?;
            evidence.insert(name, bytes);
        }
    }
    let package = Package {
        manifest,
        data,
        evidence,
    };
    package.validate()?;
    Ok(package)
}
fn run(args: &[String]) -> Result<()> {
    match args.get(1).map(String::as_str) {
        Some("validate" | "inspect") if args.len() == 3 => {
            let package = Package::import(&read(&args[2], 8 * 1024 * 1024)?)?;
            if args[1] == "inspect" {
                println!("{}", serde_json::to_string_pretty(&package.manifest).map_err(|e| e.to_string())?);
                println!("{}", serde_json::to_string_pretty(&package.data).map_err(|e| e.to_string())?);
            }
            println!("Integrity/schema valid; imported data untrusted. Human review and semantic support unverified.");
        }
        Some("preview") if args.len() == 5 => {
            let package = selected_package(args)?;
            for (name, bytes) in package.preview()? {
                println!("FILE {name} ({} bytes)", bytes.len());
                // This developer format currently includes extracted UTF-8 only.
                println!("{}", String::from_utf8(bytes).map_err(|_| "non-UTF8 preview unsupported")?);
            }
        }
        Some("pack") if args.len() == 6 => {
            let package = selected_package(args)?;
            write_new(&args[5], &package.export()?)?;
            println!("Local bundle written; nothing uploaded. Run preview before sharing.");
        }
        Some("compile") if args.len() == 6 => {
            let package = Package::import(&read(&args[2], 8 * 1024 * 1024)?)?;
            let sources = package.sources_for_workspace(&args[3])?;
            let contract: DecisionContract = serde_json::from_slice(&read(&args[4], 2 * 1024 * 1024)?).map_err(|e| e.to_string())?;
            if contract.workspace != args[3] { return Err("contract workspace mismatch".into()); }
            let mut packet = compile(&contract, &sources, &LexicalOnly)?;
            packet.warnings.push("Developer file tool used lexical retrieval ONLY. No embeddings/graph/model/API ran.".into());
            write_new(&args[5], &serde_json::to_vec_pretty(&packet).map_err(|e| e.to_string())?)?;
            println!("Evidence packet written; no generated analysis or human review implied.");
        }
        _ => return Err("usage: tacitus-capsules validate|inspect FILE; preview MANIFEST DATA EVIDENCE_ROOT; pack MANIFEST DATA EVIDENCE_ROOT OUTPUT; compile FILE WORKSPACE CONTRACT OUTPUT".into()),
    }
    Ok(())
}
fn main() {
    if let Err(error) = run(&std::env::args().collect::<Vec<_>>()) {
        eprintln!("{error}");
        std::process::exit(2);
    }
}
