/**
 * TypeScript surface for the Perry native binding.
 *
 * Perry's FFI dispatch keys on the call-site identifier, so each export
 * must call the matching `js_nsm_*` symbol listed in
 * `package.json` → `perry.nativeLibrary.functions[]`.
 *
 * The Node.js / N-API surface is generated separately into `dist/`.
 */

declare function js_nsm_init(): number;
declare function js_nsm_exit(fd: number): void;
declare function js_nsm_get_random(fd: number): Uint8Array;
declare function js_nsm_extend_pcr(
  fd: number,
  index: number,
  data: Uint8Array,
): Uint8Array;
declare function js_nsm_describe_pcr(
  fd: number,
  index: number,
): DescribePcrResponse;
declare function js_nsm_lock_pcr(fd: number, index: number): void;
declare function js_nsm_lock_pcrs(fd: number, range: number): void;
declare function js_nsm_describe_nsm(fd: number): DescribeNsmResponse;
declare function js_nsm_get_attestation_doc(
  fd: number,
  userData?: Uint8Array | null,
  nonce?: Uint8Array | null,
  publicKey?: Uint8Array | null,
): Uint8Array;

export interface DescribePcrResponse {
  lock: boolean;
  data: Uint8Array;
}

export interface DescribeNsmResponse {
  versionMajor: number;
  versionMinor: number;
  versionPatch: number;
  moduleId: string;
  maxPcrs: number;
  lockedPcrs: number[];
  digest: string;
}

/** Open the NSM device. Returns a file descriptor (negative on failure). */
export function nsmInit(): number {
  return js_nsm_init();
}

/** Close a previously opened NSM file descriptor. */
export function nsmExit(fd: number): void {
  js_nsm_exit(fd);
}

/** Request entropy from the NSM. */
export function nsmGetRandom(fd: number): Uint8Array {
  return js_nsm_get_random(fd);
}

/** Extend a PCR and return the new PCR value. */
export function nsmExtendPcr(
  fd: number,
  index: number,
  data: Uint8Array,
): Uint8Array {
  return js_nsm_extend_pcr(fd, index, data);
}

/** Describe a single PCR (`lock` flag + current value). */
export function nsmDescribePcr(
  fd: number,
  index: number,
): DescribePcrResponse {
  return js_nsm_describe_pcr(fd, index);
}

/** Lock a single PCR against further extension. */
export function nsmLockPcr(fd: number, index: number): void {
  js_nsm_lock_pcr(fd, index);
}

/** Lock PCRs in the half-open range `[0, range)`. */
export function nsmLockPcrs(fd: number, range: number): void {
  js_nsm_lock_pcrs(fd, range);
}

/** Describe the NSM module (version, capacity, locked PCRs, digest). */
export function nsmDescribeNsm(fd: number): DescribeNsmResponse {
  return js_nsm_describe_nsm(fd);
}

/** Request an attestation document, optionally binding user data / nonce / public key. */
export function nsmGetAttestationDoc(
  fd: number,
  userData?: Uint8Array | null,
  nonce?: Uint8Array | null,
  publicKey?: Uint8Array | null,
): Uint8Array {
  return js_nsm_get_attestation_doc(fd, userData, nonce, publicKey);
}
