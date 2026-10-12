# Evaluation status and design

No live model comparison has run. Public synthetic documents are developer regressions,
not held-out evidence of superiority. No percentages or frontier comparison claims exist.

Implemented records distinguish full-document, EdgeQuake retrieval, Tacitus context,
Tacitus reviewed-memory, lexical/vector and external-NOT-RUN arms. Comparisons require
equal model/dataset/evidence/time/prior-notes/output access and measured latency.
Absent human coverage/errors/qualifications/omissions/corrections are null and pending.

The future local runner must record input/output tokens, latency, retrieval/ingestion
cost, prompt/compiler/model digests and cold/warm state, then separate automatic citation
checks from analyst judgments. Repeated-task baselines receive the same prior corrections.
External inference remains disabled unless explicitly authorized later.

Required live regression: generate a fresh unpredictable fact, upload it through the
real document API, await extraction/vector/graph completion, retrieve and ask the local
model, verify its citations point to the new immutable source. This has NOT RUN.
Expected evaluation answers must remain outside fixtures/ingestion/model prompts.
