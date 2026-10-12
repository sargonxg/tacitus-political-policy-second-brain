import { api } from "./client";
export type PolicyTask =
  "auto" | "answer" | "brief" | "hypotheses" | "commitments";
export interface Span {
  source: string;
  text_hash: string;
  start: number;
  end: number;
  locator: string | null;
  original_language: string;
  translation: string | null;
}
export interface Passage {
  id: string;
  span: Span;
  text: string;
  title: string;
  author: string | null;
  source_family: string;
  selection_reason: string;
}
export interface Section {
  heading: string;
  text: string;
  citations: string[];
  qualification: string;
}
export interface PolicyAnswer {
  question: string;
  sections: Section[];
  uncertainties: string[];
  hypotheses: string[];
  matrix: {
    hypothesis: string;
    citation: string;
    consistency: string;
    diagnosticity: string;
  }[];
  positions: {
    actor: string;
    action: string;
    original_wording: string;
    citation: string;
    authority: string | null;
    deadline: string | null;
    conditions: string[];
    implementation: string;
  }[];
}
export interface Assertion {
  id: string;
  version: number;
  speaker: string;
  subject: string;
  predicate: string;
  object: string | null;
  proposition: string;
  epistemic_type: string;
  modality: string;
  conditions: string[];
  exceptions: string[];
  evidence: Span[];
}
export interface Method {
  id: string;
  version: number;
  kind: string;
  description: string;
  scope: string;
  prerequisites: string[];
  failure_conditions: string[];
  evidence: Span[];
}
export interface MethodEntry {
  method: Method;
  fingerprint: string;
  review: { decision: string; rationale: string } | null;
  applicability: { state: string; reasons: string[] };
}
export interface Analysis {
  id: string;
  status: string;
  request: { question: string };
  context: {
    passages: Passage[];
    omissions: string[];
    budget_unit: string;
  } | null;
  result: {
    answer: PolicyAnswer;
    assertions: Assertion[];
    model: {
      model: string;
      model_digest: string;
      prompt_tokens: number;
      output_tokens: number;
      duration_ms: number;
    };
    task: PolicyTask;
    review_state: string;
    methods: { id: string; state: string; reasons: string[] }[];
    citation_validation: string;
  } | null;
  error: string | null;
  freshness?: string;
}
export interface Source {
  id: string;
  version: number;
  key: string;
  title: string;
  language: string;
  source_family: string;
  redistributable: boolean;
}
export interface SourceDetail {
  id: string;
  version: number;
  title: string;
  author: string | null;
  language: string;
  published_at: number | null;
  valid_from: number | null;
  valid_until: number | null;
  source_family: string;
  redistributable: boolean;
  license: string | null;
}
export const policy = {
  suggest: (body: unknown) =>
    api.post("/tacitus/methods/suggest", body, { timeoutMs: 650000 }),
  status: () =>
    api.get<{
      provider: string;
      model: string;
      local_only_enforced: boolean;
      runtime_acceptance: string;
    }>("/tacitus/status"),
  ask: (body: unknown) =>
    api.post<{ id: string; status: string }>("/tacitus/analyses", body),
  analysis: (id: string) =>
    api.get<Analysis>(`/tacitus/analyses/${encodeURIComponent(id)}`),
  analyses: () =>
    api.get<{ analyses: { id: string; status: string; question: string }[] }>(
      "/tacitus/analyses",
    ),
  cancel: (id: string) =>
    api.post(`/tacitus/analyses/${encodeURIComponent(id)}/cancel`),
  methods: () => api.get<{ methods: MethodEntry[] }>("/tacitus/methods"),
  propose: (body: unknown) => api.post("/tacitus/methods", body),
  review: (body: unknown) => api.post("/tacitus/reviews", body),
  sources: () => api.get<{ sources: Source[] }>("/tacitus/sources"),
  inspect: (span: Span) => {
    const split = span.source.lastIndexOf("@");
    return api.get<{ source: SourceDetail; passage: string; span: Span }>(
      `/tacitus/sources/${encodeURIComponent(span.source.slice(0, split))}?version=${span.source.slice(split + 1)}&start=${span.start}&end=${span.end}`,
    );
  },
  metadata: (id: string, body: unknown) =>
    api.patch(`/tacitus/sources/${encodeURIComponent(id)}`, body),
  replace: (id: string, body: unknown) =>
    api.post(`/tacitus/sources/${encodeURIComponent(id)}/replacement`, body),
  graph: () =>
    api.get<{
      nodes: { id: string; label: string }[];
      edges: {
        id: string;
        source: string;
        target: string;
        label: string;
        assertion: Assertion;
        fingerprint: string;
        review_state: string;
      }[];
    }>("/tacitus/graph"),
  impact: () =>
    api.get<{ affected: { kind: string; id: string; reason: string }[] }>(
      "/tacitus/impact",
    ),
  preview: (body: unknown) =>
    api.post<{
      preview_hash: string;
      files: Record<string, string>;
      warning: string;
    }>("/tacitus/capsules/preview", body),
  export: (body: unknown) =>
    api.post<{ filename: string; capsule_base64: string }>(
      "/tacitus/capsules/export",
      body,
    ),
  import: (capsule_base64: string) =>
    api.post<{
      documents: { document_id: string; status: string }[];
      method_ids: string[];
      review_state: string;
    }>("/tacitus/capsules/import", { capsule_base64 }),
};
