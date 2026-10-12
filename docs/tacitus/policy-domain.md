# Policy-domain rules

Assertions distinguish reported statement, observation, allegation, inference, hypothesis,
forecast, value judgment and recommendation. Attribution, wording/modality, conditions,
exceptions, audience, speech act, evidence, two time intervals, dependencies, challenges
and confidence rationale are separate fields. Confidence is qualitative, not a fabricated
calibrated probability. Original language spans and translations remain distinct.

Publication and retrieval timestamps are separate from valid time and knowledge
availability. All integer timestamps use UTC Unix seconds; intervals are half-open.
Citation offsets are half-open UTF-8 **byte** ranges in immutable extracted text, not
JavaScript UTF-16 offsets. A client must convert explicitly for highlighting.

A statement proves that an attributed claim was made, not its truth. Discussion does not
establish participation. Approval, mandate, authorization, appropriation, expenditure
and implementation are distinct. Evidence references do not themselves prove delivery.
Declared interests and analyst-inferred motives must be distinct profiles. Weak name
similarity must not merge institutions. Common-origin reporting is grouped separately.

Current records support the shared assertion primitive; richer typed profiles for actors,
events, interests, constraints, leverage, mandates, instruments, assumptions, options,
indicators, forecasts, decisions, narratives and gaps still need dedicated validation/UI.
Independent source-quality fields (directness, expertise, incentives, independence,
completeness, verification) are planned, not a universal truth score.

Reviews bind assertion fingerprints and evidence hashes. Imported attestations are
downgraded on import. Only an authenticated server-recorded local human action can set
the trusted origin; the library alone cannot authenticate it. Any text/version/evidence
change requires review again. Tests simulate actions and do not create expert credentials.
