# Security boundary status

Current library tests cover scope/temporal rejection, stale reviews, exact approved local
model identities and malicious capsule archives. These are not proof of server-side
authorization: upstream endpoints and provider constructors are not yet constrained by
the Tacitus library. Do not import private research into this unfinished product.

Required runtime boundary: local authentication with generated persistent bootstrap
credentials; enforce tenant/workspace/selected-source scope on every document, query,
graph expansion, cache, summary and export path; reject saved connections/request
overrides that evade local-only identities; no developer cloud-key inheritance; no mock
fallbacks. Imported documents are evidence data, never instruction authority.

Outstanding acceptance: endpoint cross-workspace isolation, HTML/script injection,
prompt-injection behavior with real local models, URL/RSS SSRF protections, deletion and
revocation invalidation, cancellation/restart recovery, direct upstream route enforcement.
Public URL/RSS importing and executable plugins are not enabled.

Runtime documents, .env, uploads, exports, logs, dumps, models and local test artifacts
are excluded from Git and Docker build contexts. Only intentional synthetic fixtures
are public. Existing upstream history remains public and will be scanned separately
from the new changes. Findings must be reported without disclosing secrets.

Planned runtime services bind published ports to 127.0.0.1, with no host database/Ollama
ports by default. Localhost does not protect against every user/process on the computer.
Container offline isolation must be tested separately from host-Ollama restrictions.
No enterprise/classified readiness is claimed.
