//! Explicit local authoring. Does not fetch URLs, infer rights, or overwrite files.
use std::{
    fs,
    io::{Read, Write},
};
fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 || args[1] != "--confirm-original-evidence" {
        return Err("usage: author_native_v2 --confirm-original-evidence INPUT.json NEW_OUTPUT.zip; this flag declares embedded evidence is original authorized summary text, not publisher material".into());
    }
    let mut raw = Vec::new();
    fs::File::open(&args[2])
        .map_err(|e| e.to_string())?
        .take(2 * 1024 * 1024 + 1)
        .read_to_end(&mut raw)
        .map_err(|e| e.to_string())?;
    let bytes = tacitus_capsules::authoring::pack_original_summaries(&raw, true)?;
    let mut out = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[3])
        .map_err(|e| e.to_string())?;
    out.write_all(&bytes).map_err(|e| e.to_string())?;
    println!("Native-v2 local archive created and strict-roundtrip validated; source truth, ownership and expert review remain unverified.");
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
