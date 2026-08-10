/**
 * QCG-5 live-sidecar exit-code matrix.
 *
 * The unfamiliar and truncated rows are live-sidecar-backed fault injection:
 * the proxy forwards each target GET to the real sidecar before changing the
 * response. It does not claim that the production sidecar emits those faults.
 */

import { spawn } from "node:child_process";
import { createHash, randomUUID } from "node:crypto";
import { once } from "node:events";
import {
	appendFileSync,
	existsSync,
	readFileSync,
	writeFileSync,
} from "node:fs";
import { createServer, request as httpRequest, type IncomingHttpHeaders } from "node:http";
import { resolve } from "node:path";
import type { AddressInfo } from "node:net";

import {
	startRemProductionEntryFixture,
	type ProductionEntryFixture,
} from "/home/lh/code/sno-station-core-edge-rem-wave/tests/apps/mem-claw/helpers/rem-production-entry-fixture.ts";
import { createRemOwnerDecidedOperationalConfiguration } from "/home/lh/code/sno-station-core-edge-rem-wave/tests/apps/mem-claw/helpers/rem-entry-config-fixture.ts";
import { installRemSchema } from "/home/lh/code/sno-station-core-edge-rem-wave/packages/rem-core/src/index.ts";

type RowName =
	| "done"
	| "failed"
	| "timeout"
	| "unrecognised"
	| "truncated"
	| "stopped"
	| "profile"
	| "unknown";

interface CommandResult {
	status: number;
	stdout: string;
	stderr: string;
}

interface Observation {
	row: RowName;
	repetition: number;
	exitCode: number;
	machineCode: string | null;
	state: string | null;
	stdoutSha256: string;
	stderrSha256: string;
}

interface Discovery {
	pid: number;
	port: number;
	token: string;
}

interface FaultEvent {
	kind: "unrecognised" | "truncated";
	path: string;
	upstreamStatus: number;
	upstreamSha256: string;
	downstreamSha256: string;
	upstreamBytes: number;
	downstreamBytes: number;
}

const repoRoot = "/home/lh/code/sno-cli";
const sidecarRepo = "/home/lh/code/sno-station-core-edge-rem-wave";
const sidecarEntry = `${sidecarRepo}/apps/mem-claw/src/sidecar/main.ts`;
const binary = `${repoRoot}/target/debug/sno`;
const plan = `${repoRoot}/tests/rem-status-exit-code-matrix.plan.md`;
const planReceipt = `${repoRoot}/tests/rem-status-exit-code-matrix.plan.sha256`;
const shellRunner = `${repoRoot}/tests/rem-status-exit-code-matrix.sh`;
const helper = `${repoRoot}/tests/rem-status-exit-code-matrix.mts`;
const evidencePath = `${repoRoot}/openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/qcg-5.txt`;
const runRoot = requiredEnvironment("QCG5_RUN_ROOT");
const mode = process.argv[2];
const repetitions = mode === "full" ? 10 : mode === "smoke" ? 1 : 0;
const expectedObservations = mode === "full" ? 80 : mode === "smoke" ? 10 : 0;
const resultsPath = `${runRoot}/observations.jsonl`;
const faultPath = `${runRoot}/fault-injections.jsonl`;
const manifestPath = `${runRoot}/manifest.json`;
const summaryPath = `${runRoot}/summary.txt`;
const observations: Observation[] = [];
const activeFixtures: ProductionEntryFixture[] = [];
const activeServers = new Set<ReturnType<typeof createServer>>();
const startedAt = Date.now();
let sidecarStarts = 0;
let shuttingDown = false;

if (repetitions === 0) throw new Error("mode must be smoke or full");

const expected = new Map<RowName, { exitCode: number; machineCode: string | null; state: string | null }>([
	["done", { exitCode: 0, machineCode: null, state: "done" }],
	["failed", { exitCode: 3, machineCode: "rem_job_failed", state: null }],
	["timeout", { exitCode: 4, machineCode: "rem_timeout", state: null }],
	["unrecognised", { exitCode: 5, machineCode: "rem_state_unrecognised", state: null }],
	["truncated", { exitCode: 6, machineCode: "sidecar_response_truncated", state: null }],
	["stopped", { exitCode: 7, machineCode: "sidecar_not_running", state: null }],
	["profile", { exitCode: 8, machineCode: "profile_error", state: null }],
	["unknown", { exitCode: 9, machineCode: "rem_job_not_found", state: null }],
]);

process.on("SIGTERM", () => void emergencyShutdown(124));
process.on("SIGINT", () => void emergencyShutdown(130));

void main().catch(async (error: unknown) => {
	console.error(`QCG-5 BLOCKED: ${errorMessage(error)}`);
	await shutdown();
	process.exitCode = 1;
});

async function main(): Promise<void> {
	validatePreflight();
	const frozenBinarySha256 = sha256File(binary);
	const switchedOff = createSwitchedOffConfiguration();

	const normal = await startFixture({ configuration: switchedOff });
	await runDoneAndUnknown(normal);

	const failed = await startFixture({ configuration: switchedOff, grammar: "missing-accepted" });
	await runFailed(failed);

	const timeout = await withEnvironment("SNO_REM_TEST_HOLD_MS", "2000", () =>
		startFixture({ configuration: switchedOff }),
	);
	await runTimeout(timeout);

	const proxied = await startFixture({ configuration: switchedOff });
	await runFaultRows(proxied);

	const stopped = await startFixture({ configuration: switchedOff });
	await runStopped(stopped);

	const profile = await startFixture({ configuration: switchedOff });
	await runProfile(profile);

	if (observations.length !== expectedObservations) {
		throw new Error(`observation count mismatch: ${observations.length} != ${expectedObservations}`);
	}
	if (sidecarStarts !== 6) throw new Error(`sidecar start count mismatch: ${sidecarStarts} != 6`);
	if (sha256File(binary) !== frozenBinarySha256) throw new Error("target/debug/sno changed during the run");

	const faultEvents = readJsonLines<FaultEvent>(faultPath);
	const expectedFaults = mode === "full" ? 20 : 4;
	if (faultEvents.length !== expectedFaults) {
		throw new Error(`fault event count mismatch: ${faultEvents.length} != ${expectedFaults}`);
	}

	const durationMs = Date.now() - startedAt;
	const counts = Object.fromEntries(
		[...expected.keys()].map((row) => [row, observations.filter((item) => item.row === row).length]),
	);
	const manifest = {
		schemaVersion: 1,
		gate: "QCG-5",
		mode,
		boundary: "live-sidecar-backed fault injection",
		productionSidecarNativeFaultClaim: false,
		command: `bash tests/rem-status-exit-code-matrix.sh ${mode}`,
		runtimeRoot: runRoot,
		runtimeRootCleanupRequired: true,
		binary: { path: binary, sha256: frozenBinarySha256 },
		sidecar: { entry: sidecarEntry, sha256: sha256File(sidecarEntry), starts: sidecarStarts },
		artifacts: {
			plan: { path: plan, sha256: sha256File(plan) },
			planReceipt: { path: planReceipt, sha256: sha256File(planReceipt) },
			shellRunner: { path: shellRunner, sha256: sha256File(shellRunner) },
			helper: { path: helper, sha256: sha256File(helper) },
		},
		durationMs,
		observations: { expected: expectedObservations, passed: observations.length, failed: 0, counts },
		faultInjection: {
			observations: faultEvents.length,
			upstreamGetsForwardedBeforeInjection: faultEvents.length,
			responseHashesRetained: faultEvents.every(
				(item) => item.upstreamSha256.length === 64 && item.downstreamSha256.length === 64,
			),
		},
		noInterchange: true,
	};
	writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);
	const summary = [
		"QCG-5 PASS",
		`boundary=live-sidecar-backed fault injection; production sidecar native fault claim=false`,
		`command=bash tests/rem-status-exit-code-matrix.sh ${mode}`,
		`duration_ms=${durationMs}`,
		`observations=${observations.length}/${expectedObservations} failures=0 sidecar_starts=${sidecarStarts}`,
		`counts=${JSON.stringify(counts)}`,
		`fault_injections=${faultEvents.length} upstream_gets_before_injection=${faultEvents.length}`,
		`binary_sha256=${frozenBinarySha256}`,
		`plan_sha256=${sha256File(plan)}`,
		"no_interchange=true",
	].join("\n");
	writeFileSync(summaryPath, `${summary}\n`);
	if (mode === "full") writeEvidence(manifest, summary, faultEvents);
	await shutdown();
	console.log(summary.replaceAll("\n", " "));
}

async function runDoneAndUnknown(fixture: ProductionEntryFixture): Promise<void> {
	const doneCount = repetitions;
	const unknownCount = mode === "full" ? repetitions : 1;
	for (let index = 1; index <= doneCount; index += 1) {
		const job = await fixture.submit("rem-update", scope("done", index), correlation("done", index));
		const jobId = requiredString(job["job_id"], "done job id");
		await observe("done", index, await runCli(fixture.profileRoot, [
			"station", "rem-status", jobId, "--wait", "--timeout", "5", "--json",
		]));
	}
	for (let index = 1; index <= unknownCount; index += 1) {
		await observe("unknown", index, await runCli(fixture.profileRoot, [
			"station", "rem-status", `rem-wave-absent-${randomUUID()}`, "--json",
		]));
	}
}

async function runFailed(fixture: ProductionEntryFixture): Promise<void> {
	for (let index = 1; index <= repetitions; index += 1) {
		const job = await fixture.submit("rem-update", scope("failed", index), correlation("failed", index));
		const jobId = requiredString(job["job_id"], "failed job id");
		await observe("failed", index, await runCli(fixture.profileRoot, [
			"station", "rem-status", jobId, "--wait", "--timeout", "5", "--json",
		]));
	}
}

async function runTimeout(fixture: ProductionEntryFixture): Promise<void> {
	for (let index = 1; index <= repetitions; index += 1) {
		const job = await fixture.submit("rem-update", scope("timeout", index), correlation("timeout", index));
		const jobId = requiredString(job["job_id"], "timeout job id");
		await observe("timeout", index, await runCli(fixture.profileRoot, [
			"station", "rem-status", jobId, "--wait", "--timeout", "1", "--json",
		]));
	}
}

async function runFaultRows(fixture: ProductionEntryFixture): Promise<void> {
	const proxy = await startFaultProxy(fixture.profileRoot);
	try {
		const repeats = mode === "full" ? repetitions : 2;
		for (let index = 1; index <= repeats; index += 1) {
			const job = await fixture.submit("rem-update", scope("fault", index), correlation("fault", index));
			const jobId = requiredString(job["job_id"], "fault job id");
			await fixture.waitForTerminal(jobId, 5_000);
			proxy.setFault(`/rem/jobs/${jobId}`, "unrecognised");
			await observe("unrecognised", index, await runCli(fixture.profileRoot, [
				"station", "rem-status", jobId, "--wait", "--timeout", "5", "--json",
			]));
			proxy.setFault(`/rem/jobs/${jobId}`, "truncated");
			await observe("truncated", index, await runCli(fixture.profileRoot, [
				"station", "rem-status", jobId, "--json",
			]));
		}
	} finally {
		await proxy.stop();
	}
}

async function runStopped(fixture: ProductionEntryFixture): Promise<void> {
	const discovery = readDiscovery(fixture.profileRoot);
	await fixture.killSidecar();
	if (isProcessAlive(discovery.pid)) throw new Error(`stopped sidecar PID still alive: ${discovery.pid}`);
	for (let index = 1; index <= repetitions; index += 1) {
		await observe("stopped", index, await runCli(fixture.profileRoot, [
			"station", "rem-status", `rem-wave-stopped-${randomUUID()}`, "--json",
		]));
	}
}

async function runProfile(fixture: ProductionEntryFixture): Promise<void> {
	const discovery = readDiscovery(fixture.profileRoot);
	for (let index = 1; index <= repetitions; index += 1) {
		if (!isProcessAlive(discovery.pid)) throw new Error(`profile-row live sidecar died: ${discovery.pid}`);
		await observe("profile", index, await runCli(fixture.profileRoot, [
			"station", "rem-status", `rem-wave-profile-${randomUUID()}`, "--json",
		], ["SNO_PROFILE_DIR", "SNO_HOME", "HOME", "USERPROFILE"]));
	}
}

async function observe(row: RowName, repetition: number, result: CommandResult): Promise<void> {
	const parsed = parseOutput(result.stdout);
	const wanted = expected.get(row);
	if (wanted === undefined) throw new Error(`missing expected row: ${row}`);
	if (result.status !== wanted.exitCode) {
		throw new Error(`${row} repetition ${repetition}: exit ${result.status}, expected ${wanted.exitCode}; stdout=${JSON.stringify(result.stdout)} stderr=${JSON.stringify(result.stderr)}`);
	}
	if (parsed.machineCode !== wanted.machineCode) {
		throw new Error(`${row} repetition ${repetition}: error ${parsed.machineCode}, expected ${wanted.machineCode}; stdout=${JSON.stringify(result.stdout)}`);
	}
	if (parsed.state !== wanted.state) {
		throw new Error(`${row} repetition ${repetition}: state ${parsed.state}, expected ${wanted.state}; stdout=${JSON.stringify(result.stdout)}`);
	}
	const observation: Observation = {
		row,
		repetition,
		exitCode: result.status,
		machineCode: parsed.machineCode,
		state: parsed.state,
		stdoutSha256: sha256(result.stdout),
		stderrSha256: sha256(result.stderr),
	};
	observations.push(observation);
	appendFileSync(resultsPath, `${JSON.stringify(observation)}\n`);
	const elapsedSeconds = Math.max((Date.now() - startedAt) / 1_000, 0.001);
	const rate = observations.length / elapsedSeconds;
	const etaSeconds = (expectedObservations - observations.length) / rate;
	console.log(`progress ${observations.length}/${expectedObservations} row=${row} repetition=${repetition} pass rate=${rate.toFixed(2)}_obs/s eta=${etaSeconds.toFixed(1)}s`);
	if (mode === "full" && observations.length >= 20) {
		if (observations.length === 20 && rate < 0.97) {
			throw new Error(`throughput kill line reached: ${rate.toFixed(2)} obs/s < 0.97 obs/s at observation 20`);
		}
		const projectedTotalSeconds = expectedObservations / rate;
		if (projectedTotalSeconds > 82) {
			throw new Error(`ETA kill line reached: projected total ${projectedTotalSeconds.toFixed(1)}s > 82s`);
		}
	}
}

async function runCli(profileRoot: string, args: string[], unset: string[] = []): Promise<CommandResult> {
	const stdout: Buffer[] = [];
	const stderr: Buffer[] = [];
	const environment: NodeJS.ProcessEnv = {
		...process.env,
		OPENCLAW_STATE_DIR: resolve(profileRoot, "../state"),
		SNO_PROFILE_DIR: profileRoot,
	};
	for (const key of unset) delete environment[key];
	const child = spawn(binary, args, {
		cwd: repoRoot,
		env: environment,
		stdio: ["ignore", "pipe", "pipe"],
	});
	child.stdout.on("data", (chunk: Buffer) => stdout.push(chunk));
	child.stderr.on("data", (chunk: Buffer) => stderr.push(chunk));
	const [code, signal] = await once(child, "exit") as [number | null, NodeJS.Signals | null];
	if (signal !== null || code === null) throw new Error(`sno terminated by ${signal ?? "unknown signal"}`);
	return {
		status: code,
		stdout: Buffer.concat(stdout).toString("utf8"),
		stderr: Buffer.concat(stderr).toString("utf8"),
	};
}

async function startFixture(input: Parameters<typeof startRemProductionEntryFixture>[0]): Promise<ProductionEntryFixture> {
	const fixture = await startRemProductionEntryFixture({ ...input, entry: "source" });
	installRemSchema(fixture.database.runtime.db);
	activeFixtures.push(fixture);
	sidecarStarts += 1;
	return fixture;
}

async function startFaultProxy(profileRoot: string): Promise<{
	setFault(path: string, kind: FaultEvent["kind"]): void;
	stop(): Promise<void>;
}> {
	const discoveryPath = `${profileRoot}/station/sidecar.json`;
	const upstream = readDiscovery(profileRoot);
	let fault: { path: string; kind: FaultEvent["kind"] } | undefined;
	const server = createServer(async (request, response) => {
		try {
			const upstreamResponse = await forward(upstream, request.method ?? "GET", request.url ?? "/", request.headers);
			if (fault === undefined || request.url !== fault.path || request.method !== "GET") {
				response.writeHead(upstreamResponse.status, upstreamResponse.headers);
				response.end(upstreamResponse.body);
				return;
			}
			const currentFault = fault;
			fault = undefined;
			let downstream: Buffer;
			if (currentFault.kind === "unrecognised") {
				const job = JSON.parse(upstreamResponse.body.toString("utf8")) as Record<string, unknown>;
				job["state"] = "future-terminal-α";
				downstream = Buffer.from(JSON.stringify(job));
				response.writeHead(upstreamResponse.status, {
					...safeResponseHeaders(upstreamResponse.headers),
					"content-length": String(downstream.length),
				});
				response.end(downstream);
			} else {
				downstream = upstreamResponse.body.subarray(0, Math.max(1, upstreamResponse.body.length - 3));
				response.writeHead(upstreamResponse.status, {
					...safeResponseHeaders(upstreamResponse.headers),
					connection: "close",
					"content-length": String(upstreamResponse.body.length),
				});
				response.end(downstream);
			}
			const event: FaultEvent = {
				kind: currentFault.kind,
				path: currentFault.path,
				upstreamStatus: upstreamResponse.status,
				upstreamSha256: sha256(upstreamResponse.body),
				downstreamSha256: sha256(downstream),
				upstreamBytes: upstreamResponse.body.length,
				downstreamBytes: downstream.length,
			};
			appendFileSync(faultPath, `${JSON.stringify(event)}\n`);
		} catch (error) {
			response.statusCode = 502;
			response.end(errorMessage(error));
		}
	});
	activeServers.add(server);
	server.listen(0, "127.0.0.1");
	await once(server, "listening");
	const address = server.address() as AddressInfo;
	writeFileSync(discoveryPath, `${JSON.stringify({ ...upstream, port: address.port })}\n`, { mode: 0o600 });
	return {
		setFault(path, kind): void {
			if (fault !== undefined) throw new Error(`previous fault was not consumed: ${fault.kind}`);
			fault = { path, kind };
		},
		async stop(): Promise<void> {
			if (fault !== undefined) throw new Error(`fault was not consumed: ${fault.kind}`);
			server.close();
			await once(server, "close");
			activeServers.delete(server);
		},
	};
}

async function forward(
	discovery: Discovery,
	method: string,
	path: string,
	headers: IncomingHttpHeaders,
): Promise<{ status: number; headers: IncomingHttpHeaders; body: Buffer }> {
	return new Promise((resolvePromise, reject) => {
		const request = httpRequest({
			host: "127.0.0.1",
			port: discovery.port,
			method,
			path,
			headers: { ...headers, host: `127.0.0.1:${discovery.port}` },
		}, (response) => {
			const chunks: Buffer[] = [];
			response.on("data", (chunk: Buffer) => chunks.push(chunk));
			response.on("end", () => resolvePromise({
				status: response.statusCode ?? 500,
				headers: response.headers,
				body: Buffer.concat(chunks),
			}));
			response.on("error", reject);
		});
		request.on("error", reject);
		request.end();
	});
}

function safeResponseHeaders(headers: IncomingHttpHeaders): IncomingHttpHeaders {
	const safe = { ...headers };
	delete safe["transfer-encoding"];
	delete safe["content-length"];
	delete safe["connection"];
	return safe;
}

function parseOutput(stdout: string): { machineCode: string | null; state: string | null } {
	let machineCode: string | null = null;
	let state: string | null = null;
	for (const line of stdout.split("\n").filter(Boolean)) {
		try {
			const value = JSON.parse(line) as Record<string, unknown>;
			if (typeof value["error"] === "string") machineCode = value["error"];
			if (typeof value["state"] === "string") state = value["state"];
		} catch {
			// The raw unfamiliar state is deliberately printed before the JSON error.
		}
	}
	return { machineCode, state };
}

function validatePreflight(): void {
	for (const path of [binary, sidecarEntry, plan, planReceipt, shellRunner, helper]) {
		if (!existsSync(path)) throw new Error(`required path missing: ${path}`);
	}
	const receipt = readFileSync(planReceipt, "utf8").trim().split(/\s+/)[0];
	if (receipt !== sha256File(plan)) throw new Error(`reviewed plan hash mismatch: ${receipt} != ${sha256File(plan)}`);
	if (receipt !== "b3986e6000c35b9d625cec6db22a7804233070e9246a8a0f5a0c76690fe1ba48") {
		throw new Error(`unexpected admitted plan hash: ${receipt}`);
	}
}

function createSwitchedOffConfiguration(): Record<string, unknown> {
	const configuration = createRemOwnerDecidedOperationalConfiguration();
	configuration["operations"] = {
		"rem-update": false,
		"rem-replace": false,
		"rem-distill": false,
		"rem-retire": false,
	};
	return configuration;
}

function readDiscovery(profileRoot: string): Discovery {
	const path = `${profileRoot}/station/sidecar.json`;
	const parsed = JSON.parse(readFileSync(path, "utf8")) as Discovery;
	if (!Number.isInteger(parsed.pid) || !Number.isInteger(parsed.port) || !parsed.token) {
		throw new Error(`invalid sidecar discovery at ${path}`);
	}
	return parsed;
}

function writeEvidence(manifest: unknown, summary: string, faults: FaultEvent[]): void {
	const evidence = [
		"QCG-5 live-sidecar exit-code matrix evidence",
		"",
		"Boundary: live-sidecar-backed fault injection. The unfamiliar and truncated rows each forwarded the target GET to the live production sidecar before changing the response. This evidence does not claim that the production sidecar natively emits either fault.",
		"",
		"Manifest:",
		JSON.stringify(manifest, null, 2),
		"",
		"Summary:",
		summary,
		"",
		"Fault injection events:",
		...faults.map((event) => JSON.stringify(event)),
		"",
	].join("\n");
	writeFileSync(evidencePath, evidence);
}

async function shutdown(): Promise<void> {
	for (const server of activeServers) {
		server.close();
		await once(server, "close").catch(() => undefined);
	}
	activeServers.clear();
	for (const fixture of activeFixtures.splice(0).reverse()) {
		await fixture.stop().catch((error: unknown) => console.error(`fixture cleanup failed: ${errorMessage(error)}`));
	}
}

async function emergencyShutdown(code: number): Promise<void> {
	if (shuttingDown) return;
	shuttingDown = true;
	console.error(`QCG-5 interrupted; cleaning live sidecars with exit ${code}`);
	await shutdown();
	process.exit(code);
}

async function withEnvironment<T>(key: string, value: string, callback: () => Promise<T>): Promise<T> {
	const previous = process.env[key];
	process.env[key] = value;
	try {
		return await callback();
	} finally {
		if (previous === undefined) delete process.env[key];
		else process.env[key] = previous;
	}
}

function readJsonLines<T>(path: string): T[] {
	if (!existsSync(path)) return [];
	return readFileSync(path, "utf8").split("\n").filter(Boolean).map((line) => JSON.parse(line) as T);
}

function isProcessAlive(pid: number): boolean {
	try {
		process.kill(pid, 0);
		return true;
	} catch {
		return false;
	}
}

function requiredEnvironment(key: string): string {
	const value = process.env[key];
	if (!value) throw new Error(`${key} is required`);
	return value;
}

function requiredString(value: unknown, label: string): string {
	if (typeof value !== "string" || value.length === 0) throw new Error(`${label} is missing`);
	return value;
}

function scope(row: string, repetition: number): string {
	return `persona:qcg5-${row}-${repetition}-${randomUUID()}`;
}

function correlation(row: string, repetition: number): string {
	return `qcg5-${row}-${repetition}-${randomUUID()}`;
}

function sha256(value: string | Buffer): string {
	return createHash("sha256").update(value).digest("hex");
}

function sha256File(path: string): string {
	return sha256(readFileSync(path));
}

function errorMessage(error: unknown): string {
	return error instanceof Error ? error.stack ?? error.message : String(error);
}
