//! Read one explicit JSON dossier from stdin; emit model-independent context JSON.
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    io::{self, Read},
};
use tacitus_analysis::layers::{compile, Graph, Request};
use tacitus_core::policy::{PolicyAssertion, SourceRecord};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    request: Request,
    sources: BTreeMap<String, SourceRecord>,
    assertions: BTreeMap<String, PolicyAssertion>,
    graph: Graph,
}
fn run() -> Result<(), String> {
    let mut bytes = Vec::new();
    io::stdin()
        .take(2 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 2 * 1024 * 1024 {
        return Err("dossier exceeds 2 MiB".into());
    }
    let input: Input = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let output = compile(
        &input.request,
        &input.sources,
        &input.assertions,
        &input.graph,
    )?;
    println!(
        "{}",
        serde_json::to_string(&output).map_err(|e| e.to_string())?
    );
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
