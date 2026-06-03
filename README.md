# AWS Nitro Enclaves NSM API

Node.js bindings for the AWS Nitro Enclaves Nitro Secure Module (NSM) API.
The package is implemented as a Rust N-API addon and wraps the upstream
`aws-nitro-enclaves-nsm-api` Rust crate.

This package is intended for code that runs inside AWS Nitro Enclaves and needs
direct access to NSM operations such as attestation, PCR inspection, PCR locking,
and random byte generation.

## Package

- npm package: `@nerd-coder/aws-nitro-enclaves-nsm-api-ts`
- native layer: Rust + napi-rs
- package manager: Bun, pinned through `mise.toml`
- supported native targets:
  - `x86_64-pc-windows-msvc`
  - `x86_64-apple-darwin`
  - `aarch64-apple-darwin`
  - `x86_64-unknown-linux-gnu`
  - `aarch64-unknown-linux-gnu`

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

The CI workflow builds and smoke-tests all supported native targets. Release
publishing runs only on pushes to `main`.

Required repository secret:

- `NPM_TOKEN`: npm automation token with permission to publish
  `@nerd-coder/aws-nitro-enclaves-nsm-api-ts`.

`GITHUB_TOKEN` is provided by GitHub Actions. npm provenance is enabled through
the workflow's `id-token: write` permission, so no separate provenance secret is
needed.

Recommended branch protection:

- Require the `CI` workflow before merging to `main`.
- Do not allow release commits to bypass CI.

## Dependency Updates

Renovate is configured in `.github/renovate.json`. To enable it, install the
Renovate GitHub App for this repository and let it open dependency update pull
requests. The config keeps native CI noise low by grouping non-major updates and
limiting concurrent Renovate PRs.
