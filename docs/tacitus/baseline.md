# Sanitized preflight / blocked baseline

2026-10-11; Windows 11 Pro, x64, Intel Iris Xe integrated graphics.
Visible RAM 16,489,528 KiB; free RAM 2,731,800 KiB at preflight.
Free C: disk 14,349,701,120 bytes before clone. Upstream Git pack is approximately 449 MiB.
Rust 1.95.0; Node 24.12.0. These are development tools, not proof of container support.

Commands executed: `Get-CimInstance Win32_OperatingSystem`, `Get-PSDrive C`,
`Get-CimInstance Win32_VideoController`, `gh api user --jq .login`,
`gh repo view ... --json visibility,viewerPermission,url,isFork`, `docker version`,
`docker compose version`, `Get-Command ollama`, `wsl --list --quiet`,
`cargo --version`, `node --version`, `git remote -v`, `git status --short`.

Observed: authenticated account sargonxg; new fork PUBLIC; ADMIN permission;
Docker commands not found; standard Docker Desktop binary absent; Ollama command absent;
no WSL distribution output; no NVIDIA device observed. Parent home is not a Git repository.

Baseline models: NOT DOWNLOADED. qwen3:4b / qwen3-embedding:0.6b are requested candidates,
not accepted defaults. License, digest, quantization, effective context, dimensions,
memory use, structured output, ingestion timings and local answers are NOT MEASURED.
No paid or mock inference substituted. No inference timings can be reported.

## Exact next prerequisite

Install/start a compatible Docker Desktop Linux-container runtime with WSL2 support
through the user's ordinary installation process. Do not disable security controls.
Docker installation and host virtualization changes were not performed automatically.
Run `docker version` and `docker compose version`; free memory and disk before pulling
models or building this large upstream workspace. Then complete the unmodified live
baseline before accepting any Tacitus runtime changes. Native Ollama is optional only
after explicitly selecting and testing the host profile from inside the API container.
