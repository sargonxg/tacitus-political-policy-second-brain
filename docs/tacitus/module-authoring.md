# Trusted analytical modules

Repository-shipped Rust handlers are the only execution boundary. Imported capsule
methods are declarative data. No JavaScript, shell, arbitrary agents or endpoint/tool
permissions may be activated by imported content.

Four initial library components exist: proposed decision briefs through a model trait,
editable hypothesis matrix validation, commitment records preserving wording, and
change-impact dependency closure. None is an end-to-end UI/persistence workflow yet.
The decision brief checks citation identity and structured completeness, not semantic
entailment; the output remains a proposal. The model adapter is not implemented.

An integrated manifest must declare id/version/domain/description, input/output schema,
prerequisites/exceptions/evidence requirements, model capabilities, network/tool permissions,
budgets/dependencies, review policy, license and evaluation cases. The execution layer must
check prerequisites rather than treating unknown as satisfied. Imported manifest text
cannot authorize a tool or write accepted knowledge.

After the four initial workflows, implement in order: institution mapping; power/interests/
constraints; options including status quo; scenarios/signposts; assumptions/red-team;
mandate/authority; narrative/source families; forecast/outcome; handover. Each needs its
schema, evidence rules, UI, synthetic example and acceptance test before appearing as a
working tool. They remain roadmap items, not navigation entries.
