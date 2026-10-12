# Inherited GCP deployment reference — not the Tacitus installer

This directory contains upstream release-image deployment machinery. It does not
build the Tacitus implementation or establish a working local-model policy runtime.
Use [Tacitus self-hosting](../../docs/tacitus/self-hosting.md) and the source-build
launcher instead. No VM, project, billing configuration or operator is selected yet.

Upstream project, Terraform-state bucket and operator defaults were removed. Supply
your own reviewed project/backend configuration if separately adapting this legacy
module. Its old release scripts are disabled unless explicitly unlocked with
`TACITUS_ALLOW_LEGACY_GCP=1`; unlocking does not make them a Tacitus installation.

Do not apply this module, copy upstream infrastructure identities, or use an old
EdgeQuake container image as evidence that Tacitus changes work. Suggested future
host sizing is 8 vCPU / 32 GiB RAM / 100 GiB free disk, optionally an L4 GPU VM.
Provisioning and public Internet hosting require a separately chosen deployment.
