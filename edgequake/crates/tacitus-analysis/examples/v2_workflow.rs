use std::{
    fs,
    io::{self, Read},
};
use tacitus_analysis::v2_workflow::{render_text, LocalCatalog, Request};
fn read(path: &str) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(8 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 8 * 1024 * 1024 {
        return Err("archive too large".into());
    }
    Ok(bytes)
}
fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 || args.len() > 102 {
        return Err(
            "usage: v2_workflow inspect FILE... | review|review-text FILE < request.json".into(),
        );
    }
    let mut catalog = LocalCatalog::default();
    if args[1] == "inspect" {
        for path in &args[2..] {
            catalog.import_native_v2(&read(path)?)?;
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&catalog.search("")?).map_err(|e| e.to_string())?
        );
        return Ok(());
    }
    if args.len() != 3 || !["review", "review-text"].contains(&args[1].as_str()) {
        return Err("unknown operation".into());
    }
    let descriptor = catalog.import_native_v2(&read(&args[2])?)?;
    let mut input = String::new();
    io::stdin()
        .take(65537)
        .read_to_string(&mut input)
        .map_err(|e| e.to_string())?;
    if input.len() > 65536 {
        return Err("request too large".into());
    }
    let request: Request = serde_json::from_str(&input).map_err(|e| e.to_string())?;
    let result = catalog.export_review(&descriptor.archive_digest, &request)?;
    if args[1] == "review-text" {
        print!("{}", render_text(&result, 1_000_000)?)
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&result).map_err(|e| e.to_string())?
        )
    }
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1)
    }
}
