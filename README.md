# AWS Nitro Enclaves NSM API

[![npm version](https://img.shields.io/npm/v/@nerd-coder/aws-nitro-enclaves-nsm-api-ts.svg)](https://www.npmjs.com/package/@nerd-coder/aws-nitro-enclaves-nsm-api-ts)
[![upstream crate](https://img.shields.io/crates/v/aws-nitro-enclaves-nsm-api.svg)](https://crates.io/crates/aws-nitro-enclaves-nsm-api/)

Node.js bindings for the AWS Nitro Enclaves Nitro Secure Module (NSM) API.
The package is implemented as a Rust N-API addon and wraps the upstream
`aws-nitro-enclaves-nsm-api` Rust crate.

This package is intended for code that runs inside AWS Nitro Enclaves and needs
direct access to NSM operations such as attestation, PCR inspection, PCR locking,
and random byte generation.

## Package

- npm package: `@nerd-coder/aws-nitro-enclaves-nsm-api-ts`
- native layer: Rust + napi-rs
- upstream Rust crate:
  [`aws-nitro-enclaves-nsm-api`](https://crates.io/crates/aws-nitro-enclaves-nsm-api/)
- package manager: Bun, pinned through `mise.toml`
- supported native targets:
  - `x86_64-apple-darwin`
  - `aarch64-apple-darwin`
  - `x86_64-unknown-linux-gnu`
  - `aarch64-unknown-linux-gnu`

Windows is intentionally unsupported because the upstream Rust driver uses Unix
file descriptor and `ioctl` APIs.

## API

The generated ESM entrypoint exports low-level NSM bindings:

```ts
import {
  nsmDescribeNsm,
  nsmExit,
  nsmGetAttestationDoc,
  nsmInit,
} from "@nerd-coder/aws-nitro-enclaves-nsm-api-ts";

const fd = nsmInit();

try {
  const description = nsmDescribeNsm(fd);
  const document = nsmGetAttestationDoc(fd);

  console.log(description, document);
} finally {
  nsmExit(fd);
}
```

Most calls require a real NSM device and are only expected to work inside an AWS
Nitro Enclave. Local and CI smoke tests only verify that the generated native
binding can be imported.

## Development

Install the pinned tools with mise:

```sh
mise install
```

Install dependencies:

```sh
bun install --frozen-lockfile
```

Build the local native binding:

```sh
bun run build
```

Run local checks:

```sh
bun run lint
bun run smoke
```

`bun run check` runs linting, builds the local binding, and imports the generated
entrypoint. The project intentionally does not keep a benchmark harness because
NSM calls are device-bound and generic benchmark results are not useful for this
package.

## CI and Release Setup

Keep CI setup notes in this README while the release process stays this small.
If the workflow grows beyond the required secret and branch protection notes,
move the detailed operator runbook to `docs/ci.md` and leave a short pointer here.

GitHub Actions uses `mise.toml` to install Bun and Node, then runs:

- `bun install --frozen-lockfile`
- `bun run lint`
- target-specific `bun run build ...`
- `bun run smoke` against downloaded native artifacts

The `CI` workflow builds and smoke-tests all supported native targets on pushes,
pull requests, and manual runs. It does not publish.

The `Publish` workflow can be started manually with a `release_type` of `patch`,
`minor`, or `major`. It also runs daily at `00:00` UTC and checks the upstream
Rust crate recorded in `.upstream-version`; when crates.io has a newer stable
release, it chooses the npm release type from the upstream semver delta.

When a release is needed, `Publish` builds all native artifacts, uses
`release-it` to bump the npm package version, creates the release commit and
tag, and publishes to npm through the npm Trusted Publisher configured for
workflow filename `publish.yml`.

`GITHUB_TOKEN` is provided by GitHub Actions and is used by `release-it` to push
the release commit/tag and create the GitHub release. npm authentication and
provenance use OIDC through the workflow's `id-token: write` permission, so no
`NPM_TOKEN` secret is needed.

Recommended branch protection:

- Require the `CI` workflow before merging pull requests to `main`.
- If `main` is protected, allow the `Publish` workflow or a dedicated release
  token to push release commits and tags after the workflow's own checks pass.

## Dependency Updates

Renovate is configured in `.github/renovate.json`. To enable it, install the
Renovate GitHub App for this repository and let it open dependency update pull
requests. The config keeps native CI noise low by grouping non-major updates and
limiting concurrent Renovate PRs.
