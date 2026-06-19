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

## License

[Apache 2.0](LICENSE)
