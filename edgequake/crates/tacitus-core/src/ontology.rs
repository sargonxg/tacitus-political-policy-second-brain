//! Versioned domain vocabularies and bounded graph views. No imported executable tools.
use crate::{
    policy::{PolicyAssertion, RELATIONS},
    EpistemicType,
};
use serde::Serialize;
pub const REGISTRY_VERSION: &str = "tacitus-domain-graphs/1";
#[derive(Debug, Clone, Serialize)]
pub struct EntityDefinition {
    pub id: String,
    pub extraction_type: String,
    pub definition: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct RelationDefinition {
    pub id: String,
    pub predicate: String,
    pub definition: String,
    pub subject_types: Vec<String>,
    pub object_types: Vec<String>,
    pub safeguard: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct DomainPack {
    pub id: String,
    pub version: u32,
    pub name: String,
    pub description: String,
    pub entities: Vec<EntityDefinition>,
    pub predicates: Vec<String>,
    pub tools: Vec<String>,
    pub safeguards: Vec<String>,
}
#[derive(Debug, Clone, Serialize)]
pub struct GraphTool {
    pub id: String,
    pub version: u32,
    pub name: String,
    pub description: String,
    pub predicates: Vec<String>,
    pub epistemic_types: Vec<EpistemicType>,
    pub safeguard: String,
}
impl GraphTool {
    pub fn matches(&self, assertion: &PolicyAssertion) -> bool {
        self.predicates.iter().any(|p| p == &assertion.predicate)
            && (self.epistemic_types.is_empty()
                || self.epistemic_types.contains(&assertion.epistemic_type))
    }
}
fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|v| v.to_string()).collect()
}
pub fn relations() -> Vec<RelationDefinition> {
    RELATIONS
        .iter()
        .map(|predicate| {
            let (definition, objects, safeguard) = match *predicate {
                "states" => (
                    "An attributed source statement",
                    vec!["SOURCE_STATEMENT"],
                    "A statement is not independently established truth.",
                ),
                "expresses_position" => (
                    "An actor's reported position on an issue",
                    vec!["POSITION", "POLICY"],
                    "Discussion is not acceptance; retain audience, conditions and wording.",
                ),
                "conditionally_undertakes" => (
                    "A reported undertaking with explicit prerequisites",
                    vec!["COMMITMENT", "ACTION"],
                    "Conditional willingness is not an unconditional commitment.",
                ),
                "authorizes" => (
                    "A reported exercise of decision authority",
                    vec!["POLICY", "ACTION", "PROGRAMME"],
                    "Authorization is not appropriation, implementation or expenditure.",
                ),
                "constrains" => (
                    "A stated limitation affecting an actor or action",
                    vec!["ACTOR", "ACTION", "POLICY"],
                    "Distinguish legal, resource, operational and inferred constraints.",
                ),
                "funds" => (
                    "An attributed funding-stage statement",
                    vec!["FUNDING_STAGE", "PROGRAMME"],
                    "Pledged, committed, appropriated, disbursed and expended are distinct.",
                ),
                "implements" => (
                    "A reported implementation activity",
                    vec!["POLICY", "PROGRAMME", "ACTION"],
                    "Announcing an action is not evidence that it occurred.",
                ),
                "withdraws" => (
                    "An attributed withdrawal from a position or undertaking",
                    vec!["POSITION", "COMMITMENT"],
                    "Retain the withdrawn wording, actor, timing and qualifications.",
                ),
                "contradicts" => (
                    "A proposed evidential or positional contradiction",
                    vec!["SOURCE_STATEMENT", "POSITION"],
                    "Different dates, scopes or conditions may explain apparent disagreement.",
                ),
                "depends_on" => (
                    "A stated or proposed implementation prerequisite",
                    vec!["ACTION", "COMMITMENT", "RESOURCE"],
                    "Inferred dependencies stay inferences and require support.",
                ),
                "affiliated_with" => (
                    "A reported affiliation",
                    vec!["ACTOR", "INSTITUTION"],
                    "Affiliation is not control; shared labels do not resolve identity.",
                ),
                _ => unreachable!("Canonical relation requires a domain definition"),
            };
            RelationDefinition {
                id: format!("tacitus.relation.{predicate}"),
                predicate: predicate.to_string(),
                definition: definition.into(),
                subject_types: strings(&["ACTOR", "INSTITUTION", "SOURCE_STATEMENT"]),
                object_types: strings(&objects),
                safeguard: safeguard.into(),
            }
        })
        .collect()
}
fn tool(
    id: &str,
    name: &str,
    description: &str,
    predicates: &[&str],
    types: Vec<EpistemicType>,
    safeguard: &str,
) -> GraphTool {
    GraphTool {
        id: id.into(),
        version: 1,
        name: name.into(),
        description: description.into(),
        predicates: strings(predicates),
        epistemic_types: types,
        safeguard: safeguard.into(),
    }
}
pub fn tools() -> Vec<GraphTool> {
    vec![
        tool(
            "authority",
            "Authority and constraints",
            "Inspect authorization statements and limitations affecting action.",
            &["authorizes",
                "constrains",
                "depends_on"],
            vec![],
            "An edge does not establish lawful authority or completed expenditure."),

        tool(
            "positions",
            "Positions and conditional commitments",
            "Inspect attributed positions, prerequisites and withdrawals.",
            &["expresses_position",
                "conditionally_undertakes",
                "withdraws"],
            vec![],
            "Retain original wording; discussion and conditional willingness are not acceptance."),

        tool(
            "funding",
            "Funding and implementation",
            "Inspect funding stages, implementation reports and prerequisites.",
            &["funds",
                "authorizes",
                "implements",
                "depends_on",
                "constrains"],
            vec![],
            "Do not collapse authorization, appropriation, disbursement and expenditure."),

        tool(
            "conflict",
            "Conflict and competing positions",
            "Inspect proposed contradictions, constraints, positions and dependencies.",
            &["contradicts",
                "constrains",
                "expresses_position",
                "conditionally_undertakes",
                "depends_on"],
            vec![],
            "This filter does not prove conflict, intent, control or causality."),

        tool(
            "hypotheses",
            "Hypotheses and inferences",
            "Separate proposed explanations from reported statements.",
            RELATIONS,
            vec![EpistemicType::Inference,
                EpistemicType::Hypothesis,
                EpistemicType::Forecast],
            "No invented probabilities or independent confirmation from repeated origins."),

        tool(
            "rights",
            "Rights claims and observations",
            "Inspect reported statements, allegations and observations for rights analysis.",
            &["states",
                "contradicts",
                "constrains"],
            vec![EpistemicType::ReportedStatement,
                EpistemicType::Allegation,
                EpistemicType::Observation],
            "An allegation is not an established violation or a legal finding. This view does not classify a claim as rights-related automatically."),

        tool(
            "workforce",
            "Workforce policy and implementation",
            "Inspect decisions, positions, implementation and workforce-policy dependencies.",
            &["authorizes",
                "expresses_position",
                "constrains",
                "implements",
                "depends_on"],
            vec![],
            "Do not infer individual eligibility, performance or discrimination from an affiliation or a model-generated edge."),

    ]
}
fn pack(
    id: &str,
    name: &str,
    description: &str,
    extra: &[(&str, &str)],
    predicates: &[&str],
    tools: &[&str],
    safeguard: &str,
) -> DomainPack {
    let common = [
        ("ACTOR",
            "A source-bound person or collective; ambiguous identities remain separate."),
        ("INSTITUTION",
            "An organization with reported roles; institutional affiliation does not establish control."),
        ("SOURCE_STATEMENT",
            "An attributed statement with exact evidence and epistemic status."),
        ("POSITION",
            "An actor's attributed stance, preserving qualifications."),
        ("COMMITMENT",
            "A reported undertaking; retain prerequisites and implementation unknowns."),
        ("EVIDENCE_GAP",
            "Evidence needed but not supplied; absence is not negative evidence automatically.")];
    let entities = common
        .iter()
        .chain(extra)
        .map(|(kind, definition)| EntityDefinition {
            id: format!("{id}.entity.{}", kind.to_lowercase()),
            extraction_type: kind.to_string(),
            definition: definition.to_string(),
        })
        .collect();
    DomainPack {
        id:id.into(),
        version:1,
        name:name.into(),
        description:description.into(),
        entities,
        predicates:strings(predicates),
        tools:strings(tools),
        safeguards:strings(&[safeguard,
                "Source passages, attribution, language, time and human review remain authoritative. Entity typing is a proposed extraction, not adjudication."])}
}
pub fn packs() -> Vec<DomainPack> {
    vec![
        pack(
            "public-policy",
            "Public policy",
            "Authority, programme resources, implementation dependencies and affected groups.",
            &[("POLICY",
                    "A proposed or adopted policy; adoption and implementation are distinct."),
                ("PROGRAMME",
                    "A programme with documented mandate and implementation stage."),
                ("ACTION",
                    "A proposed or reported action."),
                ("FUNDING_STAGE",
                    "A documented funding stage, retaining amount/currency when evidenced."),
                ("RESOURCE",
                    "A stated resource requirement or constraint."),
                ("AFFECTED_GROUP",
                    "A group affected by an option; do not invent its preferences.")],
            RELATIONS,
            &["authority",
                "funding",
                "positions",
                "hypotheses"],
            "Authorization is not expenditure; proposed benefits are not observed outcomes."),

        pack(
            "diplomacy-conflict",
            "Diplomacy and conflict",
            "Actors, negotiations, positions, conflict events and conditional undertakings.",
            &[("NEGOTIATION",
                    "A reported negotiation, with participation distinct from acceptance."),
                ("CONFLICT_EVENT",
                    "An attributed event with uncertainty and temporal qualifications."),
                ("AGREEMENT",
                    "A reported agreement; signature, entry into force and implementation differ."),
                ("ACTION",
                    "A proposed or reported action."),
                ("RESOURCE",
                    "A stated resource or constraint."),
                ("AFFECTED_GROUP",
                    "A source-described group affected by conflict or policy.")],
            RELATIONS,
            &["conflict",
                "positions",
                "authority",
                "hypotheses"],
            "Do not infer motive, command/control, identity or agreement from repeated names or affiliation."),

        pack(
            "human-rights",
            "Human rights",
            "Rights claims, alleged events, legal instruments, affected groups and safeguards.",
            &[("RIGHT",
                    "A claimed right with its stated legal basis, not an automatic legal finding."),
                ("LEGAL_INSTRUMENT",
                    "A cited legal instrument; jurisdiction and applicability require assessment."),
                ("ALLEGED_EVENT",
                    "An attributed allegation, distinct from independently verified observation."),
                ("AFFECTED_GROUP",
                    "A source-described affected group."),
                ("SAFEGUARD",
                    "A proposed or reported safeguard with documented scope."),
                ("ACTION",
                    "A proposed or reported action.")],
            &["states",
                "contradicts",
                "constrains",
                "authorizes",
                "implements",
                "depends_on",
                "expresses_position"],
            &["rights",
                "authority",
                "hypotheses",
                "positions"],
            "Preserve allegation versus observation and do not adjudicate violations automatically."),

        pack(
            "workforce-policy",
            "HR / workforce policy",
            "Policies, roles, eligibility rules, procedures, resources and affected groups.",
            &[("POLICY",
                    "An employment or workforce policy with stated scope."),
                ("JOB_ROLE",
                    "A documented role; role descriptions are not individual performance evidence."),
                ("ELIGIBILITY_RULE",
                    "A stated eligibility rule with exceptions and jurisdiction."),
                ("PROCEDURE",
                    "A documented process, not evidence of its application in an individual case."),
                ("RESOURCE",
                    "A stated resource requirement."),
                ("AFFECTED_GROUP",
                    "A source-described workforce group."),
                ("ACTION",
                    "A proposed or reported action.")],
            &["states",
                "authorizes",
                "expresses_position",
                "constrains",
                "implements",
                "depends_on",
                "affiliated_with"],
            &["workforce",
                "authority",
                "positions",
                "hypotheses"],
            "Use authorized material; do not infer sensitive personal attributes or individual judgments."),

    ]
}
pub fn find_pack(id: &str) -> Option<DomainPack> {
    packs().into_iter().find(|p| p.id == id)
}
pub fn find_tool(id: &str) -> Option<GraphTool> {
    tools().into_iter().find(|t| t.id == id)
}

/// A label search only; it must never resolve or merge actor identities.
pub fn actor_matches(assertion: &PolicyAssertion, label: Option<&str>) -> bool {
    let needle = label.unwrap_or_default().trim().to_lowercase();
    needle.is_empty()
        || [&assertion.subject, &assertion.speaker]
            .iter()
            .any(|s| s.to_lowercase().contains(&needle))
}
