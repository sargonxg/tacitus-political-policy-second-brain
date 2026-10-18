# TACITUS local capsule discovery UI

A local runnable interface for the existing capsule catalog. Search and filter → inspect exact revision, sources, declared rights, reported claims and limitations → export the exchange or explicitly prepare an unreviewed attachment proposal.

## Start

Node 22+; no runtime dependencies or API keys.

```sh
node tools/tacitus-capsule-catalog-ui/server.mjs 8787
```

Open http://127.0.0.1:8787 in your browser. Stop with Ctrl+C.

The process binds loopback only. It is an ephemeral, single-operator local session: imports disappear when it stops. Export anything you need to keep. The seed capsule is fictional with synthetic CC0 source records; original catalog metadata is preserved. No files are fetched from URLs. No payment, hosted marketplace, public upload, account or production PRAXIS integration is provided.

## Trust boundary

Imports go through the existing exact canonical exchange validator, source hashes and span checks. All imported values are rendered with textContent. Preparing an attachment requires explicit local evidence inclusion; the server supplies current time and rechecks expiry and retirement. This is not a PRAXIS review approval. The existing contract stays unchanged.

Loopback Host and Origin checks, cross-site request rejection, a per-process custom-header token, restrictive CSP and no CORS protect the local HTTP boundary. These do not replace authenticated owner/workspace authorization required for hosting. Any program running as the local user may access this service. Do not expose or reverse-proxy it.

## Tests and known limits

```sh
node --test tools/tacitus-capsule-catalog-ui/test.mjs tools/tacitus-capsule-catalog-ui/security.test.mjs
```

Optional browser test requires an already installed Playwright and Chromium. Run browser-test.cjs from this directory; it checks discovery, consent, download, malformed/duplicate imports and phone overflow, then saves screenshots. Browser launch was blocked in the development environment; no browser acceptance or screenshots are claimed. HTTP workflow/security tests and JavaScript syntax checks are separate from visual verification.

Responsive CSS, visible labels, keyboard focus styles and live status announcements are implemented. Full keyboard/screen-reader and visual QA remain pending.
