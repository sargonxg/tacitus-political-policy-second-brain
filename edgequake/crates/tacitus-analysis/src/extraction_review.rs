//! Process-local extraction review. No imported attestation grants authority.
use crate::{
    knowledge_capture::{self, Capture},
    layers::{Knowledge, LinkKind},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tacitus_capsules::strict_json;
use tacitus_core::{digest, policy::SourceRecord, Result, Span};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Extractor {
    pub kind: String,
    pub tool: String,
    pub version: String,
    pub run_id: String,
    pub produced_at: i64,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mention {
    pub span: Span,
    pub quote: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Envelope {
    pub format: String,
    pub extractor: Extractor,
    pub capture: Capture,
    pub entity_mentions: BTreeMap<String, Vec<Mention>>,
}
/// Immutable validated candidate, not reusable knowledge or a review grant.
pub struct Proposal {
    raw: Vec<u8>,
    envelope: Envelope,
    snapshot: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Decision {
    pub proposal_digest: String,
    pub actor: String,
    pub workspace: String,
    pub reviewed_at: i64,
    pub rationale: String,
    pub reviewed_all_claims_entities_relations_and_times: bool,
    pub reviewed_manifest_and_source_metadata: bool,
    pub acknowledges_corrections: bool,
    pub confirms_rights: bool,
}
#[derive(Serialize)]
pub struct Receipt {
    pub proposal_digest: String,
    pub source_snapshot_digest: String,
    pub actor: String,
    pub workspace: String,
    pub reviewed_at: i64,
    pub rationale: String,
    pub scope: String,
}
/// Deliberately non-Clone and non-Deserialize; consumed by promotion.
pub struct LocalReview {
    receipt: Receipt,
}
pub struct Promotion {
    pub archive: Vec<u8>,
    pub receipt: Receipt,
}
fn json<T: Serialize>(v: &T) -> Result<Vec<u8>> {
    serde_json::to_vec(v).map_err(|e| e.to_string())
}
fn text(v: &str) -> bool {
    !v.trim().is_empty() && v.len() <= 4000
}
fn snapshot(docs: &[SourceRecord]) -> Result<String> {
    let mut map = BTreeMap::new();
    for d in docs {
        d.validate()?;
        if map.insert(d.key(), d).is_some() {
            return Err("duplicate source identity".into());
        }
    }
    Ok(digest(&json(&map)?))
}
impl Proposal {
    pub fn parse(raw: &[u8]) -> Result<Self> {
        let e: Envelope = strict_json::from_slice(raw)?;
        let c = &e.capture;
        if e.format != "tacitus-extraction-proposal/1"
            || c.confirms_rights
            || !["human", "local_model", "external_model"].contains(&e.extractor.kind.as_str())
            || !text(&e.extractor.tool)
            || !text(&e.extractor.version)
            || !text(&e.extractor.run_id)
            || e.extractor.produced_at != c.manifest.created_at
            || c.graph.knowledge.values().any(|k| *k != Knowledge::Unknown)
            || e.entity_mentions.len() != c.graph.nodes.len()
        {
            return Err("bounded unreviewed extraction proposal required".into());
        }
        let snapshot = snapshot(&c.documents)?;
        let sources: BTreeMap<_, _> = c.documents.iter().map(|d| (d.key(), d)).collect();
        for (id, node) in &c.graph.nodes {
            let mentions = e
                .entity_mentions
                .get(id)
                .ok_or("unresolved entity mention")?;
            if mentions.is_empty() || mentions.len() > 32 {
                return Err("entity mentions required".into());
            }
            for claim_id in &node.claims {
                let claim = c
                    .assertions
                    .iter()
                    .find(|a| a.id == *claim_id)
                    .ok_or("unresolved entity claim")?;
                if !mentions.iter().any(|m| {
                    claim.evidence.iter().any(|p| {
                        p.source == m.span.source && p.start <= m.span.start && p.end >= m.span.end
                    })
                }) {
                    return Err("each entity claim requires anchored mention evidence".into());
                }
            }
            for m in mentions {
                let s = sources
                    .get(&m.span.source)
                    .ok_or("missing mention source")?;
                if s.passage(&m.span)? != m.quote
                    || !node.claims.iter().any(|id| {
                        c.assertions.iter().any(|a| {
                            a.id == *id
                                && a.evidence.iter().any(|p| {
                                    p.source == m.span.source
                                        && p.start <= m.span.start
                                        && p.end >= m.span.end
                                })
                        })
                    })
                {
                    return Err("entity quote or claim provenance mismatch".into());
                }
            }
        }
        for link in &c.graph.links {
            if link.kind == LinkKind::Supersedes {
                let from = c
                    .assertions
                    .iter()
                    .find(|a| a.id == link.from)
                    .ok_or("unresolved correction")?;
                let to = c
                    .assertions
                    .iter()
                    .find(|a| a.id == link.to)
                    .ok_or("unresolved correction")?;
                if from.created_at < to.created_at {
                    return Err("correction authorship precedes target".into());
                }
            }
        }
        // Exercise existing exact-span, ontology, graph fingerprint and chronology validators.
        // This private validation archive is discarded and never grants candidate authority.
        let mut validation = c.clone();
        validation.confirms_rights = true;
        knowledge_capture::pack(&json(&validation)?)?;
        Ok(Self {
            raw: raw.to_vec(),
            envelope: e,
            snapshot,
        })
    }
    pub fn digest(&self) -> String {
        digest(&self.raw)
    }
    pub fn review_material(&self) -> &[u8] {
        &self.raw
    }
    fn current(&self, current: &[SourceRecord], now: i64) -> Result<()> {
        if snapshot(current)? != self.snapshot {
            return Err("source snapshot changed; new review required".into());
        }
        if now < self.envelope.extractor.produced_at
            || self
                .envelope
                .capture
                .manifest
                .valid_until
                .is_some_and(|t| now >= t)
            || current.iter().any(|s| {
                !s.eligible(&self.envelope.capture.manifest.scope, now, Some(now))
                    || !s.redistributable
            })
        {
            return Err("review checkpoint or source eligibility invalid".into());
        }
        Ok(())
    }
    /// The caller is the trusted local operator boundary. Actor is a label, not authentication.
    pub fn acknowledge(&self, current: &[SourceRecord], decision: Decision) -> Result<LocalReview> {
        self.current(current, decision.reviewed_at)?;
        if decision.proposal_digest != self.digest()
            || decision.workspace != self.envelope.capture.manifest.scope
            || !text(&decision.actor)
            || !text(&decision.rationale)
            || !decision.reviewed_all_claims_entities_relations_and_times
            || !decision.reviewed_manifest_and_source_metadata
            || !decision.confirms_rights
            || self
                .envelope
                .capture
                .graph
                .links
                .iter()
                .any(|l| l.kind == LinkKind::Supersedes)
                && !decision.acknowledges_corrections
        {
            return Err("explicit complete digest-bound local review required".into());
        }
        Ok(LocalReview {receipt:Receipt {proposal_digest:self.digest(),source_snapshot_digest:self.snapshot.clone(),actor:decision.actor,workspace:decision.workspace,reviewed_at:decision.reviewed_at,rationale:decision.rationale,scope:"process-local extraction and rights acknowledgement; not authentication, expert approval or truth verification".into()}})
    }
    pub fn promote(
        &self,
        grant: LocalReview,
        current: &[SourceRecord],
        now: i64,
        actor: &str,
    ) -> Result<Promotion> {
        self.current(current, now)?;
        if grant.receipt.actor != actor
            || grant.receipt.proposal_digest != self.digest()
            || grant.receipt.source_snapshot_digest != self.snapshot
            || grant.receipt.workspace != self.envelope.capture.manifest.scope
            || grant.receipt.reviewed_at > now
        {
            return Err("review grant does not bind this proposal/checkpoint".into());
        }
        let mut capture = self.envelope.capture.clone();
        capture.confirms_rights = true;
        capture.manifest.created_at = grant.receipt.reviewed_at;
        capture.graph.available_at = grant.receipt.reviewed_at;
        capture.extractor=String::from_utf8(json(&serde_json::json!({"format":"tacitus-extraction-review-provenance/1","extractor":self.envelope.extractor,"local_review":grant.receipt}))?).map_err(|e|e.to_string())?;
        let archive = knowledge_capture::pack(&json(&capture)?)?;
        Ok(Promotion {
            archive,
            receipt: grant.receipt,
        })
    }
}
