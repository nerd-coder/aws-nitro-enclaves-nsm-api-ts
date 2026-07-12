#!/usr/bin/env node
/**
 * Build the Perry staticlib (default features) and stage it under
 * prebuilt/<os>-<arch>/ for vendor-prebuilt distribution.
 *
 * Usage:
 *   node scripts/stage-perry-prebuilt.mjs
 *   node scripts/stage-perry-prebuilt.mjs --target aarch64-apple-darwin
 *   node scripts/stage-perry-prebuilt.mjs --target x86_64-unknown-linux-gnu --skip-build
 *
 * Mapping (Perry #860 per-arch keys):
 *   aarch64-apple-darwin       → macos-arm64
 *   x86_64-apple-darwin        → macos-x64
 *   aarch64-unknown-linux-gnu  → linux-arm64
 *   x86_64-unknown-linux-gnu   → linux-x64
 */
import { copyFileSync, existsSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const LIB_BASENAME = "libaws_nitro_enclaves_nsm_api.a";

const TARGET_TO_PERRY_ARCH = {
	"aarch64-apple-darwin": "macos-arm64",
	"x86_64-apple-darwin": "macos-x64",
	"aarch64-unknown-linux-gnu": "linux-arm64",
	"x86_64-unknown-linux-gnu": "linux-x64",
};

function hostRustTarget() {
	const { status, stdout, stderr } = spawnSync(
		"rustc",
		["-vV"],
		{ encoding: "utf8" },
	);
	if (status !== 0) {
		throw new Error(`rustc -vV failed: ${stderr || stdout}`);
	}
	const match = /^host:\s*(\S+)/m.exec(stdout);
	if (!match) {
		throw new Error(`Could not parse host triple from rustc -vV:\n${stdout}`);
	}
	return match[1];
}

function parseArgs(argv) {
	let target = null;
	let skipBuild = false;
	for (let i = 0; i < argv.length; i++) {
		const arg = argv[i];
		if (arg === "--target") {
			target = argv[++i];
			if (!target) {
				throw new Error("--target requires a rustc triple");
			}
		} else if (arg === "--skip-build") {
			skipBuild = true;
		} else if (arg === "--help" || arg === "-h") {
			console.log(`Usage: stage-perry-prebuilt.mjs [--target <triple>] [--skip-build]`);
			process.exit(0);
		} else {
			throw new Error(`Unknown argument: ${arg}`);
		}
	}
	return { target: target ?? hostRustTarget(), skipBuild };
}

function main() {
	const { target, skipBuild } = parseArgs(process.argv.slice(2));
	const perryArch = TARGET_TO_PERRY_ARCH[target];
	if (!perryArch) {
		throw new Error(
			`Unsupported rust target "${target}". Supported: ${Object.keys(TARGET_TO_PERRY_ARCH).join(", ")}`,
		);
	}

	if (!skipBuild) {
		console.log(`Building Perry staticlib for ${target} (default features include perry)…`);
		const build = spawnSync(
			"cargo",
			["build", "--release", "--target", target],
			{ cwd: ROOT, stdio: "inherit" },
		);
		if (build.status !== 0) {
			process.exit(build.status ?? 1);
		}
	}

	const source = join(ROOT, "target", target, "release", LIB_BASENAME);
	// Host builds without --target land in target/release/; also accept that
	// when the requested target matches the host and the triple path is missing.
	const hostFallback = join(ROOT, "target", "release", LIB_BASENAME);
	const resolvedSource = existsSync(source)
		? source
		: existsSync(hostFallback) && target === hostRustTarget()
			? hostFallback
			: source;

	if (!existsSync(resolvedSource)) {
		throw new Error(
			`Staticlib not found at ${source}` +
				(resolvedSource !== source ? ` or ${hostFallback}` : "") +
				`. Run without --skip-build first.`,
		);
	}

	const destDir = join(ROOT, "prebuilt", perryArch);
	const dest = join(destDir, LIB_BASENAME);
	mkdirSync(destDir, { recursive: true });
	copyFileSync(resolvedSource, dest);
	console.log(`Staged ${resolvedSource} → ${dest}`);
}

try {
	main();
} catch (err) {
	console.error(err instanceof Error ? err.message : err);
	process.exit(1);
}
