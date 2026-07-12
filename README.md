# AWS Nitro Enclaves NSM API

[![npm version](https://img.shields.io/npm/v/@nerd-coder/aws-nitro-enclaves-nsm-api-ts.svg)](https://www.npmjs.com/package/@nerd-coder/aws-nitro-enclaves-nsm-api-ts)
[![upstream crate](https://img.shields.io/crates/v/aws-nitro-enclaves-nsm-api.svg)](https://crates.io/crates/aws-nitro-enclaves-nsm-api/)

Bindings for the AWS Nitro Enclaves Nitro Secure Module (NSM) API.
The package wraps the upstream [`aws-nitro-enclaves-nsm-api`](https://crates.io/crates/aws-nitro-enclaves-nsm-api/)
Rust crate and ships two runtimes:

| Runtime | Mechanism | Install |
| --- | --- | --- |
| **Node.js / Bun** | Rust N-API addon (`napi-rs`) | prebuilt platform packages |
| **[Perry](https://docs.perryts.com/)** | `perry-ffi` staticlib + `perry.nativeLibrary` | [vendor prebuilts](https://docs.perryts.com/native-libraries/authoring-guide.html#two-distribution-models) in the npm tarball |

This package is intended for code that runs inside AWS Nitro Enclaves and needs
direct access to NSM operations such as attestation, PCR inspection, PCR locking,
and random byte generation.

## Package

- npm package: `@nerd-coder/aws-nitro-enclaves-nsm-api-ts`
- native layer: Rust + napi-rs (Node) / perry-ffi (Perry)
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

The same low-level NSM surface is exported for both runtimes:

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

### Node.js / Bun

Uses the prebuilt N-API addon under `dist/` (plus optional platform packages).
Local and CI smoke tests only verify that the generated native binding can be
imported.

### Perry

The package declares a `perry.nativeLibrary` block in `package.json` and ships
**vendor prebuilts** — ready-to-link staticlibs under `prebuilt/<os>-<arch>/` —
so `perry compile` does **not** need a Rust toolchain on the consumer machine.

Supported Perry targets:

| Manifest key | Prebuilt path |
| --- | --- |
| `macos-arm64` | `prebuilt/macos-arm64/libaws_nitro_enclaves_nsm_api.a` |
| `macos-x64` | `prebuilt/macos-x64/libaws_nitro_enclaves_nsm_api.a` |
| `linux-arm64` | `prebuilt/linux-arm64/libaws_nitro_enclaves_nsm_api.a` |
| `linux-x64` | `prebuilt/linux-x64/libaws_nitro_enclaves_nsm_api.a` |
| `windows` | unavailable (NSM uses Unix `ioctl` / file descriptors) |

When you `import` this package from a Perry program, the compiler:

1. Reads the manifest (`abiVersion: "0.5"`, `functions[]`, per-arch `prebuilt` paths)
2. Resolves the TypeScript surface via the `exports["."].perry` condition (`src/index.ts`)
3. Links the matching `prebuilt/<os>-<arch>/libaws_nitro_enclaves_nsm_api.a` and the
   `js_nsm_*` `extern "C"` symbols into your binary

```sh
bun add @nerd-coder/aws-nitro-enclaves-nsm-api-ts
```

Host programs must allow-list native libraries (Perry security gate):

```json
{
  "perry": {
    "allow": {
      "nativeLibrary": ["@nerd-coder/aws-nitro-enclaves-nsm-api-ts"]
    }
  }
}
```

```sh
perry compile main.ts -o main
```

#### Developing this package

Stage a host prebuilt (builds the staticlib with default `perry` features):

```sh
bun run build:perry:prebuilt
# or a specific triple:
bun run build:perry:prebuilt -- --target aarch64-apple-darwin
```

Validate the binding (requires the `perry` CLI; rebuilds from source and diffs
symbols against the manifest):

```sh
bun run validate:perry
# or
perry native validate
```

Published releases build all four Perry prebuilts in CI and include them in the
npm tarball (`files` includes `prebuilt/`).

Most calls require a real NSM device and are only expected to work inside an AWS
Nitro Enclave.

## License

[Apache 2.0](LICENSE)
