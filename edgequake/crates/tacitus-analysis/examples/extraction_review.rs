use std::{
    fs,
    io::{Read, Write},
};
use tacitus_analysis::extraction_review::{Decision, Proposal};
use tacitus_capsules::strict_json;
use tacitus_core::policy::SourceRecord;
fn read(p: &str) -> Result<Vec<u8>, String> {
    let mut b = Vec::new();
    fs::File::open(p)
        .map_err(|e| e.to_string())?
        .take(2 * 1024 * 1024 + 1)
        .read_to_end(&mut b)
        .map_err(|e| e.to_string())?;
    if b.len() > 2 * 1024 * 1024 {
        return Err("input exceeds 2 MiB".into());
    }
    Ok(b)
}
fn run() -> Result<(), String> {
    let a: Vec<_> = std::env::args().collect();
    match a.get(1).map(String::as_str){
 Some("inspect") if a.len()==3=>{let p=Proposal::parse(&read(&a[2])?)?;let value:serde_json::Value=strict_json::from_slice(p.review_material())?;println!("{}",serde_json::json!({"proposal_digest":p.digest(),"status":"candidate only; local review required","review_material":value}));Ok(())},
 Some("promote-local") if a.len()==6=>{let p=Proposal::parse(&read(&a[2])?)?;let current:Vec<SourceRecord>=strict_json::from_slice(&read(&a[3])?)?;let d:Decision=strict_json::from_slice(&read(&a[4])?)?;let now=d.reviewed_at;let actor=d.actor.clone();let grant=p.acknowledge(&current,d)?;let promoted=p.promote(grant,&current,now,&actor)?;let mut f=fs::OpenOptions::new().write(true).create_new(true).open(&a[5]).map_err(|e|e.to_string())?;f.write_all(&promoted.archive).map_err(|e|e.to_string())?;println!("{}",serde_json::to_string(&promoted.receipt).map_err(|e|e.to_string())?);Ok(())},
 _=>Err("usage: extraction_review inspect PROPOSAL.json | promote-local PROPOSAL.json CURRENT_SOURCES.json LOCAL_DECISION.json NEW.zip".into())
 }
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1)
    }
}
