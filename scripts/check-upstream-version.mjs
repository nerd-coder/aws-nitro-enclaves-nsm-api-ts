import { appendFileSync, readFileSync } from "node:fs";

const CRATE_NAME = "aws-nitro-enclaves-nsm-api";
const CRATE_API_URL = `https://crates.io/api/v1/crates/${CRATE_NAME}`;

function parseVersion(version) {
	const match = version.match(/^(\d+)\.(\d+)\.(\d+)$/);
	if (!match) {
		throw new Error(`Expected stable semver version, got "${version}"`);
	}
	return match.slice(1).map(Number);
}

function compareVersions(left, right) {
	const leftParts = parseVersion(left);
	const rightParts = parseVersion(right);
	for (let i = 0; i < leftParts.length; i += 1) {
		if (leftParts[i] > rightParts[i]) return 1;
		if (leftParts[i] < rightParts[i]) return -1;
	}
	return 0;
}

function releaseTypeFor(currentVersion, latestVersion) {
	const [currentMajor, currentMinor] = parseVersion(currentVersion);
	const [latestMajor, latestMinor] = parseVersion(latestVersion);
	if (latestMajor > currentMajor) return "major";
	if (latestMinor > currentMinor) return "minor";
	return "patch";
}

function setOutput(name, value) {
	console.log(`${name}=${value}`);
	if (process.env.GITHUB_OUTPUT) {
		appendFileSync(process.env.GITHUB_OUTPUT, `${name}=${value}\n`);
	}
}

const response = await fetch(CRATE_API_URL, {
	headers: {
		Accept: "application/json",
		"User-Agent": "aws-nitro-enclaves-nsm-api-ts-upstream-watch",
	},
});

if (!response.ok) {
	throw new Error(`Failed to fetch ${CRATE_API_URL}: ${response.status} ${response.statusText}`);
}

const payload = await response.json();
const latestVersion = payload.crate?.max_stable_version;

if (!latestVersion) {
	throw new Error(`No max_stable_version found for ${CRATE_NAME}`);
}

const currentVersion = readFileSync(".upstream-version", "utf8").trim();
const hasNewRelease = compareVersions(latestVersion, currentVersion) > 0;
const releaseType = hasNewRelease ? releaseTypeFor(currentVersion, latestVersion) : "patch";

setOutput("current_version", currentVersion);
setOutput("latest_version", latestVersion);
setOutput("release_type", releaseType);
setOutput("should_release", hasNewRelease ? "true" : "false");
