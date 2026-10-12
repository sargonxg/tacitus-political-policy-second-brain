# Hardware and local models — unmeasured

Actual test host: Windows 11 x64, 16 GB RAM, Intel Iris Xe, approximately 2.6 GB free RAM
and 13 GB free disk at preflight. Rust libraries ran; local models did not. Do not assume
this currently available memory fits qwen3:4b plus embeddings, PostgreSQL and source builds.

Requested candidates: `qwen3:4b` and `qwen3-embedding:0.6b`; optional `qwen3:8b` later.
No weights downloaded. Model availability/license, quantization/digest, dimensions,
structured output, effective context and measured memory remain unverified.

Required acceptance profiles: CPU container Ollama first, NVIDIA only after device/toolkit
checks, host Ollama only after explicitly selecting a native server and testing API-container
connectivity. No NVIDIA profile can be accepted on the current observed hardware.

Set and verify `OLLAMA_NO_CLOUD=1` in the actual server environment. Resolve every active
role locally (extract/query/keyword/summary/embedding); explicitly disable untested vision,
decision/reranking/translation roles. Use extraction concurrency one and reserve output
budget. Changing embedding identity requires a separate compatible re-embedding/index.

Pin source-built API/frontend and compatible pgvector+Apache AGE database artifacts;
verify registry architecture/digests and extension loading. Do not use plain PostgreSQL.
Keep dependencies/models in persistent volumes; never remove volumes automatically.
Neither Compose nor offline container startup is implemented/verified yet.
