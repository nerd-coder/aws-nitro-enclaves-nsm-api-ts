import { readFileSync, writeFileSync } from "node:fs";

const upstreamVersion = process.argv[2] || process.env.UPSTREAM_VERSION;

if (!upstreamVersion) {
	console.log("No upstream version provided; leaving Cargo.toml and .upstream-version unchanged.");
	process.exit(0);
}

if (!/^\d+\.\d+\.\d+$/.test(upstreamVersion)) {
	throw new Error(`Expected stable semver upstream version, got "${upstreamVersion}"`);
}

const cargoTomlPath = "Cargo.toml";
const cargoToml = readFileSync(cargoTomlPath, "utf8");
const updatedCargoToml = cargoToml.replace(
	/^aws-nitro-enclaves-nsm-api = "([^"]+)"$/m,
	`aws-nitro-enclaves-nsm-api = "${upstreamVersion}"`,
);

if (updatedCargoToml === cargoToml && !cargoToml.includes(`aws-nitro-enclaves-nsm-api = "${upstreamVersion}"`)) {
	throw new Error("Could not find aws-nitro-enclaves-nsm-api dependency in Cargo.toml");
}

writeFileSync(cargoTomlPath, updatedCargoToml);
writeFileSync(".upstream-version", `${upstreamVersion}\n`);
console.log(`Set upstream aws-nitro-enclaves-nsm-api version to ${upstreamVersion}`);
