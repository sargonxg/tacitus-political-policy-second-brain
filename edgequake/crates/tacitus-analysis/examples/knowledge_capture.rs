use std::{
    fs,
    io::{Read, Write},
};
use tacitus_analysis::knowledge_capture as k;
fn read(path: &str) -> Result<Vec<u8>, String> {
    let mut b = Vec::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(8 * 1024 * 1024 + 1)
        .read_to_end(&mut b)
        .map_err(|e| e.to_string())?;
    if b.len() > 8 * 1024 * 1024 {
        return Err("input exceeds 8 MiB".into());
    }
    Ok(b)
}
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    let output=match args.get(1).map(String::as_str){
Some("propose") if args.len()==3=>serde_json::to_vec(&k::propose(&read(&args[2])?)?).map_err(|e|e.to_string())?,
Some("pack") if args.len()==4=>{let bytes=k::pack(&read(&args[2])?)?;let mut f=fs::OpenOptions::new().write(true).create_new(true).open(&args[3]).map_err(|e|e.to_string())?;f.write_all(&bytes).map_err(|e|e.to_string())?;return Ok(());},
Some("inspect") if args.len()==3=>serde_json::to_vec(&k::inspect(&read(&args[2])?)?).map_err(|e|e.to_string())?,
Some("reuse") if args.len()==3=>{let mut q=Vec::new();std::io::stdin().take(2*1024*1024+1).read_to_end(&mut q).map_err(|e|e.to_string())?;k::reuse(&read(&args[2])?,&q)?},
_=>return Err("usage: knowledge_capture propose DOCUMENTS.json | pack CAPTURE.json NEW.zip | inspect CAPSULE.zip | reuse CAPSULE.zip < QUERY.json".into())};
    std::io::stdout()
        .write_all(&output)
        .map_err(|e| e.to_string())?;
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
