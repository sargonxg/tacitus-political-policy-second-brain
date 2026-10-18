use std::{
    fs,
    io::{self, Read},
};
use tacitus_analysis::cited_brief::{compile, render_text, Request};
use tacitus_capsules::Package;
fn run() -> Result<(), String> {
    let path = std::env::args()
        .nth(1)
        .ok_or("usage: cited_brief selected.tacitus.zip < request.json")?;
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut data = Vec::new();
    file.take(8 * 1024 * 1024 + 1)
        .read_to_end(&mut data)
        .map_err(|e| e.to_string())?;
    let package = Package::import(&data)?;
    let mut input = String::new();
    io::stdin()
        .take(65537)
        .read_to_string(&mut input)
        .map_err(|e| e.to_string())?;
    if input.len() > 65536 {
        return Err("request too large".into());
    }
    let request: Request = serde_json::from_str(&input).map_err(|e| e.to_string())?;
    let output = compile(&package, &request)?;
    if std::env::args().nth(2).as_deref() == Some("--text") {
        print!("{}", render_text(&output, 500_000)?)
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&output).map_err(|e| e.to_string())?
        )
    };
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1)
    }
}
