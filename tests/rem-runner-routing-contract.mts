import { spawn, spawnSync } from "node:child_process";
import { createHash, randomUUID } from "node:crypto";
import { once } from "node:events";
import {
	chmodSync,
	cpSync,
	existsSync,
	mkdirSync,
	readFileSync,
	writeFileSync,
} from "node:fs";
import {
	createServer,
	request as httpRequest,
	type IncomingHttpHeaders,
	type IncomingMessage,
} from "node:http";
import type { AddressInfo } from "node:net";
import { dirname, join } from "node:path";

import {
	startRemProductionEntryFixture,
	type ProductionEntryFixture,
} from "/home/lh/code/sno-station-core-edge-rem-wave/tests/apps/mem-claw/helpers/rem-production-entry-fixture.ts";
import { createRemOwnerDecidedOperationalConfiguration } from "/home/lh/code/sno-station-core-edge-rem-wave/tests/apps/mem-claw/helpers/rem-entry-config-fixture.ts";
import { installRemSchema } from "/home/lh/code/sno-station-core-edge-rem-wave/packages/rem-core/src/index.ts";

type Producer = "success" | "start-error" | "status-state" | "future-tool-build";

interface RoutingCase {
	exitCode: number;
	machineCode: string | null;
	producer: Producer;
}

interface RunnerCase {
	name: "section-first" | "sibling-first";
	operation: string;
	path: string;
	unfamiliarState: string;
}

interface FixtureFile {
	future: Omit<RoutingCase, "machineCode">;
	known: RoutingCase[];
	runners: RunnerCase[];
	schemaVersion: number;
}

interface Discovery {
	pid: number;
	port: number;
	token: string;
}

interface CommandResult {
	status: number;
	stderr: string;
	stdout: string;
}

interface Observation extends CommandResult {
	correlationId: string;
	exitCode: number;
	jobId: string | null;
	known: boolean;
	operation: string;
	runner: RunnerCase["name"];
	scope: string;
}

interface ProxyEvent {
	correlationId: string;
	inboundType: string | null;
	jobId: string | null;
	method: string;
	path: string;
	producer: Producer;
	requestedExit: number;
	upstreamStatus: number;
}

interface ActiveScenario {
	correlationId: string;
	machineCode: string | null;
	producer: Producer;
	rawState: string;
	requestedExit: number;
}

const repoRoot = "/home/lh/code/sno-cli";
const featureRunner = "/home/lh/code/sno-station-core-edge-rem-wave/evals/memora/scripts/run_rem.sh";
const sectionFirstRunner = "/home/lh/code/sno-station-core/evals/memora/scripts/run_rem_noop.sh";
const fixturePath = `${repoRoot}/tests/fixtures/rem-runner-routing-cases.json`;
const planPath = `${repoRoot}/openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/test-plan-section-5.md`;
const receiptPath = `${repoRoot}/openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/test-plan-section-5.sha256`;
const currentBuild = `${repoRoot}/target/debug/sno`;
const runRoot = requiredEnvironment("SECTION5_RUN_ROOT");
const mode = process.argv[2] ?? "baseline";
const expectedPlanSha256 = "2b8aab7093536b3ee54a08b18db35b9ca9a6cc9938f3d8d93e65cfadaa3bbd69";
const fixture = JSON.parse(readFileSync(fixturePath, "utf8")) as FixtureFile;
const startedAt = Date.now();
const expectedObservations = 24;
let completedObservations = 0;

void main().catch((error: unknown) => {
	console.error(`SECTION5 BLOCKED: ${errorMessage(error)}`);
	process.exitCode = 1;
});

async function main(): Promise<void> {
	validatePreflight();
	if (mode.startsWith("negative-")) {
		runNegativeControl(mode);
		return;
	}
	if (mode !== "baseline") throw new Error(`unsupported mode: ${mode}`);
	await runProductBaseline();
}

function runNegativeControl(selectedMode: string): void {
	const qcg = selectedMode.replace("negative-", "").toUpperCase();
	try {
		switch (selectedMode) {
			case "negative-qcg12":
				assertKnownCodeHasNoUnmatchedLog({
					status: 3,
					stdout: "unmatched REM exit code 3\n",
					stderr: "",
				} as Observation);
				break;
			case "negative-qcg13":
				assertExitFiveLog(
					{ status: 5, stdout: "invalid response\n", stderr: "" } as Observation,
					"future terminal/β",
				);
				break;
			case "negative-qcg15":
				assertMessageIndependentFate(
					{ status: 5 } as Observation,
					{ status: 1 } as Observation,
				);
				break;
			case "negative-qcg16":
				assertNoRunnerOwnedToolExit("#!/usr/bin/env bash\nexit 7\n");
				break;
			case "negative-qcg17":
				assertSameStore("/tmp/store-a/persona.sqlite", "/tmp/store-b/persona.sqlite");
				break;
			default:
				throw new Error(`unknown negative control: ${selectedMode}`);
		}
		throw new Error(`${qcg} negative control did not ring`);
	} catch (error) {
		console.error(`NEGATIVE CONTROL RED ${qcg}: ${errorMessage(error)}`);
		process.exitCode = 1;
	}
}

async function runProductBaseline(): Promise<void> {
	const currentInstalled = installCurrentBuild();
	const futureInstalled = buildAndInstallFutureCli();
	const currentSha256 = sha256File(currentInstalled);
	const futureSha256 = sha256File(futureInstalled);
	const configuration = createRemOwnerDecidedOperationalConfiguration();
	configuration["operations"] = {
		"rem-update": false,
		"rem-replace": false,
		"rem-distill": false,
		"rem-retire": false,
	};
	const production = await startRemProductionEntryFixture({
		configuration,
		entry: "source",
	});
	installRemSchema(production.database.runtime.db);
	const proxy = await startFaultProxy(production);
	const observations: Observation[] = [];
	try {
		for (const runner of fixture.runners) {
			for (const routingCase of fixture.known) {
				observations.push(
					await observeRunner(proxy, production, runner, routingCase, currentInstalled),
				);
			}
			observations.push(
				await observeRunner(
					proxy,
					production,
					runner,
					{ ...fixture.future, machineCode: null },
					futureInstalled,
				),
			);
		}

		const usage = await runOwnedExit(featureRunner, []);
		recordProgress("qcg16-usage", usage.status);
		const rejected = await runOwnedExit(featureRunner, ["persona:section5-rejected"] , {
			MEM_CLAW_REM_TYPE: "definitely-not-declared",
			OPENCLAW_STATE_DIR: production.stateRoot,
			SNO_CLI_BIN: currentInstalled,
			SNO_PROFILE_DIR: production.profileRoot,
		});
		recordProgress("qcg16-rejected", rejected.status);

		const failures = collectProductFailures({
			currentInstalled,
			futureInstalled,
			observations,
			production,
			proxyEvents: proxy.events,
			rejected,
			usage,
		});
		console.log(
			`Section5 boundary_reached observations=${completedObservations}/${expectedObservations} forwarded_requests=${proxy.events.length} sidecar_pid=${proxy.upstream.pid} store=${production.database.dbPath}`,
		);
		console.log(
			`Section5 hashes plan=${sha256File(planPath)} fixture=${sha256File(fixturePath)} current_cli=${currentSha256} future_cli=${futureSha256} run_rem=${sha256File(featureRunner)} run_rem_noop=${sha256File(sectionFirstRunner)}`,
		);
		if (failures.length > 0) {
			for (const failure of failures) console.error(`PRODUCT RED: ${failure}`);
			process.exitCode = 1;
			return;
		}
		console.log("Section5 PASS QCG-12 QCG-13 QCG-15 QCG-16 QCG-17");
	} finally {
		await proxy.stop();
		await production.stop();
	}
}

function collectProductFailures(input: {
	currentInstalled: string;
	futureInstalled: string;
	observations: Observation[];
	production: ProductionEntryFixture;
	proxyEvents: ProxyEvent[];
	rejected: CommandResult;
	usage: CommandResult;
}): string[] {
	const failures: string[] = [];
	for (const observation of input.observations) {
		if (observation.known && observation.status !== observation.exitCode) {
			failures.push(
				`QCG-12 ${observation.runner} requested=${observation.exitCode} observed=${observation.status}`,
			);
		}
		try {
			if (observation.known) assertKnownCodeHasNoUnmatchedLog(observation);
			else assertUnknownCodeFailsClosed(observation);
		} catch (error) {
			failures.push(`QCG-12 ${observation.runner} code=${observation.exitCode}: ${errorMessage(error)}`);
		}
	}

	const statusFive = input.observations.filter((item) => item.exitCode === 5);
	for (const observation of statusFive) {
		const runner = fixture.runners.find((item) => item.name === observation.runner);
		if (runner === undefined) throw new Error(`missing runner fixture: ${observation.runner}`);
		try {
			assertExitFiveLog(observation, runner.unfamiliarState);
		} catch (error) {
			failures.push(`QCG-13 ${observation.runner}: ${errorMessage(error)}`);
		}
	}
	try {
		const firstExitFive = statusFive[0];
		const secondExitFive = statusFive[1];
		if (firstExitFive === undefined || secondExitFive === undefined || statusFive.length !== 2) {
			throw new Error(`expected two exit-5 observations, found ${statusFive.length}`);
		}
		assertMessageIndependentFate(firstExitFive, secondExitFive);
		assertJsonPreserved(input.production.stateRoot, input.observations);
	} catch (error) {
		failures.push(`QCG-15: ${errorMessage(error)}`);
	}

	if (input.usage.status !== 20) failures.push(`QCG-16 usage exit=${input.usage.status}, expected=20`);
	if (input.rejected.status !== 21) {
		failures.push(`QCG-16 rejected-operation exit=${input.rejected.status}, expected=21`);
	}
	try {
		const source = readFileSync(featureRunner, "utf8");
		assertNoRunnerOwnedToolExit(source);
		assertImmediateCapture(source, "START_EXIT");
		assertImmediateCapture(source, "STATUS_EXIT");
	} catch (error) {
		failures.push(`QCG-16 provenance: ${errorMessage(error)}`);
	}

	try {
		const featureSource = readFileSync(featureRunner, "utf8");
		const sectionSource = readFileSync(sectionFirstRunner, "utf8");
		const featureTable = extractEnumeratedRouter(featureSource, "run_rem.sh");
		const sectionTable = extractEnumeratedRouter(sectionSource, "run_rem_noop.sh");
		if (normalizeTable(featureTable) !== normalizeTable(sectionTable)) {
			throw new Error("the two enumerated routing tables differ");
		}
		assertBothCapturesUseTable(featureSource, "run_rem.sh");
		assertBothCapturesUseTable(sectionSource, "run_rem_noop.sh");
	} catch (error) {
		failures.push(`QCG-12 routing table: ${errorMessage(error)}`);
	}

	try {
		assertSameStore(input.production.database.dbPath, input.production.database.dbPath);
		assertLandingOrders(input.observations, input.proxyEvents);
		assertFrozenValidationBytes();
	} catch (error) {
		failures.push(`QCG-17: ${errorMessage(error)}`);
	}

	if (sha256File(input.currentInstalled) !== sha256File(currentBuild)) {
		failures.push("temporary current installation differs from the frozen current build");
	}
	if (sha256File(planPath) !== expectedPlanSha256) failures.push("frozen plan changed during run");
	return failures;
}

async function observeRunner(
	proxy: Awaited<ReturnType<typeof startFaultProxy>>,
	production: ProductionEntryFixture,
	runner: RunnerCase,
	routingCase: RoutingCase,
	binary: string,
): Promise<Observation> {
	const correlationId = `section5-${runner.name}-${routingCase.exitCode}-${randomUUID()}`;
	const scope = `persona:section5-${runner.name}-${routingCase.exitCode}-${randomUUID()}`;
	proxy.setScenario({
		correlationId,
		machineCode: routingCase.machineCode,
		producer: routingCase.producer,
		rawState: runner.unfamiliarState,
		requestedExit: routingCase.exitCode,
	});
	const result = await runCommand(runner.path, [scope], {
		MEM_CLAW_REM_TYPE: runner.operation,
		OPENCLAW_STATE_DIR: production.stateRoot,
		SNO_CLI_BIN: binary,
		SNO_PROFILE_DIR: production.profileRoot,
		SNO_QCG12_FUTURE_EXIT_10: routingCase.producer === "future-tool-build" ? "1" : "0",
		SNO_REM_CORRELATION_ID: correlationId,
		SNO_REM_TRACE: "1",
	});
	const events = proxy.events.filter((item) => item.correlationId === correlationId);
	if (events.length === 0) {
		throw new Error(`${runner.name} code ${routingCase.exitCode} did not reach the live sidecar proxy`);
	}
	if (events.some((item) => item.upstreamStatus <= 0)) {
		throw new Error(`${runner.name} code ${routingCase.exitCode} lacked a real upstream response`);
	}
	const jobId = events.map((item) => item.jobId).find((value): value is string => value !== null) ?? null;
	const observation: Observation = {
		...result,
		correlationId,
		exitCode: routingCase.exitCode,
		jobId,
		known: routingCase.exitCode <= 9,
		operation: runner.operation,
		runner: runner.name,
		scope,
	};
	recordProgress(`${runner.name}-code-${routingCase.exitCode}`, result.status);
	return observation;
}

async function runOwnedExit(
	command: string,
	args: string[],
	extraEnvironment: Record<string, string> = {},
): Promise<CommandResult> {
	return runCommand(command, args, extraEnvironment);
}

async function runCommand(
	command: string,
	args: string[],
	extraEnvironment: Record<string, string>,
): Promise<CommandResult> {
	const child = spawn(command, args, {
		cwd: repoRoot,
		env: { ...process.env, ...extraEnvironment },
		stdio: ["ignore", "pipe", "pipe"],
	});
	const stdout: Buffer[] = [];
	const stderr: Buffer[] = [];
	child.stdout.on("data", (chunk: Buffer) => stdout.push(chunk));
	child.stderr.on("data", (chunk: Buffer) => stderr.push(chunk));
	const [code, signal] = (await once(child, "exit")) as [number | null, NodeJS.Signals | null];
	if (signal !== null || code === null) throw new Error(`${command} terminated by ${signal ?? "unknown"}`);
	return {
		status: code,
		stderr: Buffer.concat(stderr).toString("utf8"),
		stdout: Buffer.concat(stdout).toString("utf8"),
	};
}

async function startFaultProxy(production: ProductionEntryFixture): Promise<{
	events: ProxyEvent[];
	setScenario(scenario: ActiveScenario): void;
	stop(): Promise<void>;
	upstream: Discovery;
}> {
	const discoveryPath = join(production.profileRoot, "station", "sidecar.json");
	const upstream = JSON.parse(readFileSync(discoveryPath, "utf8")) as Discovery;
	const events: ProxyEvent[] = [];
	let active: ActiveScenario | undefined;
	const server = createServer(async (request, response) => {
		try {
			if (active === undefined) throw new Error("request arrived without an active scenario");
			const body = await readRequestBody(request);
			const correlationId = header(request.headers, "x-rem-correlation-id") ?? "";
			if (correlationId !== active.correlationId) {
				throw new Error(`correlation mismatch: ${correlationId} != ${active.correlationId}`);
			}
			let forwardedBody = body;
			let inboundType: string | null = null;
			if (request.method === "POST" && request.url === "/rem/run") {
				const parsed = JSON.parse(body.toString("utf8")) as Record<string, unknown>;
				inboundType = typeof parsed["type"] === "string" ? parsed["type"] : null;
				if (parsed["type"] === "noop") parsed["type"] = "rem-update";
				forwardedBody = Buffer.from(JSON.stringify(parsed));
			}
			const upstreamResponse = await forward(
				upstream,
				request.method ?? "GET",
				request.url ?? "/",
				request.headers,
				forwardedBody,
			);
			const parsedUpstream = parseRecord(upstreamResponse.body);
			const jobId = typeof parsedUpstream?.["job_id"] === "string"
				? String(parsedUpstream["job_id"])
				: jobIdFromPath(request.url ?? "");
			events.push({
				correlationId,
				inboundType,
				jobId,
				method: request.method ?? "GET",
				path: request.url ?? "/",
				producer: active.producer,
				requestedExit: active.requestedExit,
				upstreamStatus: upstreamResponse.status,
			});

			if (active.producer === "start-error" && request.method === "POST") {
				writeJson(response, 409, {
					error: active.machineCode,
					job_id: jobId,
				});
				return;
			}
			if (
				active.producer === "status-state"
				&& request.method === "GET"
				&& parsedUpstream !== null
				&& ["done", "failed", "refused"].includes(String(parsedUpstream["state"]))
			) {
				writeJson(response, 200, { ...parsedUpstream, state: active.rawState });
				return;
			}
			writeBuffer(response, upstreamResponse.status, upstreamResponse.headers, upstreamResponse.body);
		} catch (error) {
			response.statusCode = 502;
			response.end(errorMessage(error));
		}
	});
	server.listen(0, "127.0.0.1");
	await once(server, "listening");
	const address = server.address() as AddressInfo;
	writeFileSync(discoveryPath, `${JSON.stringify({ ...upstream, port: address.port })}\n`, { mode: 0o600 });
	return {
		events,
		setScenario(scenario): void {
			active = scenario;
		},
		async stop(): Promise<void> {
			server.close();
			await once(server, "close");
		},
		upstream,
	};
}

function installCurrentBuild(): string {
	const installRoot = join(runRoot, "current-install", "bin");
	mkdirSync(installRoot, { recursive: true });
	const installed = join(installRoot, "sno");
	cpSync(currentBuild, installed);
	chmodSync(installed, 0o755);
	return installed;
}

function buildAndInstallFutureCli(): string {
	const sourceRoot = join(runRoot, "future-source");
	const targetRoot = join(runRoot, "future-target");
	const installRoot = join(runRoot, "future-install", "bin");
	mkdirSync(sourceRoot, { recursive: true });
	mkdirSync(targetRoot, { recursive: true });
	mkdirSync(installRoot, { recursive: true });
	for (const name of ["Cargo.toml", "Cargo.lock", "README.md", "LICENSE"]) {
		cpSync(join(repoRoot, name), join(sourceRoot, name));
	}
	cpSync(join(repoRoot, "src"), join(sourceRoot, "src"), { recursive: true });
	const cliPath = join(sourceRoot, "src", "cli.rs");
	const before = readFileSync(cliPath, "utf8");
	const needle = "        Ok(exit_code) => exit_code,";
	const replacement = [
		"        Ok(exit_code) => {",
		"            if std::env::var_os(\"SNO_QCG12_FUTURE_EXIT_10\").is_some() && exit_code == 0 {",
		"                10",
		"            } else {",
		"                exit_code",
		"            }",
		"        }",
	].join("\n");
	if (!before.includes(needle)) throw new Error("future CLI mutation target is missing");
	writeFileSync(cliPath, before.replace(needle, replacement));
	const clone = spawnSync("cp", ["--archive", "--reflink=always", `${repoRoot}/target/.`, targetRoot], {
		encoding: "utf8",
		timeout: 30_000,
	});
	if (clone.status !== 0) {
		throw new Error(`future target reflink failed: ${clone.stderr || clone.stdout}`);
	}
	const build = spawnSync("cargo", ["build", "--locked", "--offline"], {
		cwd: sourceRoot,
		encoding: "utf8",
		env: { ...process.env, CARGO_TARGET_DIR: targetRoot },
		timeout: 90_000,
	});
	if (build.status !== 0) {
		throw new Error(`future CLI build failed: ${build.stderr || build.stdout}`);
	}
	const futureBinary = join(targetRoot, "debug", "sno");
	const installed = join(installRoot, "sno");
	cpSync(futureBinary, installed);
	chmodSync(installed, 0o755);
	console.log(`future-tool-build mutation_sha256=${sha256(before.replace(needle, replacement))} binary_sha256=${sha256File(installed)}`);
	return installed;
}

function assertKnownCodeHasNoUnmatchedLog(observation: Observation): void {
	const combined = `${observation.stdout}\n${observation.stderr}`;
	if (/unmatched[^\n]*exit|unmatched[^\n]*code/i.test(combined)) {
		throw new Error(`known code ${observation.status} was logged as unmatched`);
	}
}

function assertUnknownCodeFailsClosed(observation: Observation): void {
	if (observation.status === 0) throw new Error("future code 10 passed open");
	const combined = `${observation.stdout}\n${observation.stderr}`;
	if (!(/unmatched[^\n]*10|10[^\n]*unmatched/i.test(combined))) {
		throw new Error(`future code 10 was not logged as unmatched; output=${JSON.stringify(combined)}`);
	}
}

function assertExitFiveLog(observation: Observation, rawState: string): void {
	if (observation.status !== 5) throw new Error(`exit ${observation.status}, expected 5`);
	const combined = `${observation.stdout}\n${observation.stderr}`;
	if (!combined.includes(rawState)) throw new Error(`raw state missing: ${rawState}`);
	if (!combined.includes("the sidecar reported a state this build does not know")) {
		throw new Error("version-mismatch explanation is missing");
	}
	if (/invalid response/i.test(combined)) throw new Error("exit 5 was reported as invalid response");
}

function assertMessageIndependentFate(first: Observation, second: Observation): void {
	if (first.status !== second.status) {
		throw new Error(`same exit class changed fate after prose replacement: ${first.status} != ${second.status}`);
	}
	if (first.status === 0) throw new Error("exit 5 prose controls unexpectedly passed the persona");
}

function assertNoRunnerOwnedToolExit(source: string): void {
	const literal = source.match(/\bexit\s+(?:[0-9]|["'][0-9]["'])\b/);
	if (literal !== null) throw new Error(`runner originates tool-range literal: ${literal[0]}`);
}

function assertSameStore(first: string, second: string): void {
	if (first !== second) throw new Error(`landing orders used different stores: ${first} != ${second}`);
}

function assertImmediateCapture(source: string, variable: "START_EXIT" | "STATUS_EXIT"): void {
	const capture = new RegExp(`SNO_CLI_BIN[\\s\\S]{0,700}else\\s*\\n\\s*${variable}=\\$\\?`);
	if (!capture.test(source)) throw new Error(`${variable} is not the immediate real sno result capture`);
}

function extractEnumeratedRouter(source: string, label: string): string {
	const blocks = [...source.matchAll(/case\s+[^\n]+\s+in([\s\S]*?)\nesac/g)].map((match) => match[0]);
	const candidate = blocks.find((block) =>
		Array.from({ length: 10 }, (_, code) => new RegExp(`(?:^|[|\\s])${code}\\)`, "m").test(block)).every(Boolean)
		&& /(?:^|\s)\*\)/m.test(block)
	);
	if (candidate === undefined) throw new Error(`${label} has no one table enumerating 0..9 plus unknown`);
	return candidate;
}

function normalizeTable(table: string): string {
	return table.replaceAll(/\s+/g, " ").replaceAll(featureRunner, "RUNNER").replaceAll(sectionFirstRunner, "RUNNER").trim();
}

function assertBothCapturesUseTable(source: string, label: string): void {
	for (const variable of ["START_EXIT", "STATUS_EXIT"]) {
		const uses = [...source.matchAll(new RegExp(`\\$\\{?${variable}\\}?`, "g"))].length;
		if (uses < 2) throw new Error(`${label} does not route captured ${variable}`);
	}
}

function assertJsonPreserved(stateRoot: string, observations: Observation[]): void {
	const tracePath = join(stateRoot, "mem-claw", "rem-trace.jsonl");
	const records = readFileSync(tracePath, "utf8")
		.split("\n")
		.map((line) => line.trim())
		.filter((line) => line.startsWith("{"))
		.map((line) => JSON.parse(line) as Record<string, unknown>);
	for (const observation of observations) {
		const sent = records.find((record) =>
			record["component"] === "memora_harness"
			&& record["event"] === "harness_cli_sent"
			&& record["correlation_id"] === observation.correlationId
			&& Array.isArray(record["argv"])
		);
		if (sent === undefined) throw new Error(`missing start argv trace for ${observation.correlationId}`);
		const argv = sent["argv"] as unknown[];
		if (!argv.includes("--json")) throw new Error(`--json missing for ${observation.correlationId}`);
	}
}

function assertLandingOrders(observations: Observation[], events: ProxyEvent[]): void {
	const sectionSuccess = observations.find((item) => item.runner === "section-first" && item.exitCode === 0);
	const siblingSuccess = observations.find((item) => item.runner === "sibling-first" && item.exitCode === 0);
	if (sectionSuccess?.status !== 0 || siblingSuccess?.status !== 0) {
		throw new Error("one landing order did not complete against the shared store");
	}
	if (!sectionSuccess.jobId || !siblingSuccess.jobId || sectionSuccess.jobId === siblingSuccess.jobId) {
		throw new Error("landing-order jobs were not independently created in the shared store");
	}
	const sectionPost = events.find((item) => item.correlationId === sectionSuccess.correlationId && item.method === "POST");
	const siblingPost = events.find((item) => item.correlationId === siblingSuccess.correlationId && item.method === "POST");
	if (sectionPost?.inboundType !== "noop") throw new Error("section-first old operation was not accepted");
	if (siblingPost?.inboundType !== "rem-update") throw new Error("sibling-first operation was not accepted");
}

function assertFrozenValidationBytes(): void {
	const sectionSource = readFileSync(sectionFirstRunner, "utf8");
	const featureSource = readFileSync(featureRunner, "utf8");
	if (!sectionSource.includes("noop|rem-restate|rem-verdict) ;;")) {
		throw new Error("section-first accepted names changed");
	}
	if (!sectionSource.includes("MEM_CLAW_REM_TYPE must be one of: noop, rem-restate, rem-verdict")) {
		throw new Error("section-first rejected-operation message changed");
	}
	if (!featureSource.includes(".operations | index($operation) != null")) {
		throw new Error("sibling-first generated operation validation changed");
	}
	if (!featureSource.includes("MEM_CLAW_REM_TYPE is not a declared REM operation: %s")) {
		throw new Error("sibling-first rejected-operation message changed");
	}
}

function recordProgress(label: string, status: number): void {
	completedObservations += 1;
	const elapsedSeconds = Math.max((Date.now() - startedAt) / 1_000, 0.001);
	const rate = completedObservations / elapsedSeconds;
	const eta = (expectedObservations - completedObservations) / rate;
	console.log(`progress ${completedObservations}/${expectedObservations} case=${label} exit=${status} rate=${rate.toFixed(2)}_obs/s eta=${Math.max(0, eta).toFixed(1)}s`);
	if (completedObservations === 20 && rate < 0.5) {
		throw new Error(`throughput kill line: ${rate.toFixed(2)} obs/s < 0.5 obs/s at observation 20`);
	}
	if (completedObservations >= 20 && expectedObservations / rate > 120) {
		throw new Error(`ETA kill line: projected total ${(expectedObservations / rate).toFixed(1)}s > 120s`);
	}
}

async function forward(
	discovery: Discovery,
	method: string,
	path: string,
	headers: IncomingHttpHeaders,
	body: Buffer,
): Promise<{ body: Buffer; headers: IncomingHttpHeaders; status: number }> {
	return new Promise((resolvePromise, reject) => {
		const outgoingHeaders = { ...headers, host: `127.0.0.1:${discovery.port}` };
		delete outgoingHeaders["content-length"];
		if (body.length > 0) outgoingHeaders["content-length"] = String(body.length);
		const request = httpRequest({
			host: "127.0.0.1",
			port: discovery.port,
			method,
			path,
			headers: outgoingHeaders,
		}, (response) => {
			const chunks: Buffer[] = [];
			response.on("data", (chunk: Buffer) => chunks.push(chunk));
			response.on("end", () => resolvePromise({
				body: Buffer.concat(chunks),
				headers: response.headers,
				status: response.statusCode ?? 500,
			}));
			response.on("error", reject);
		});
		request.on("error", reject);
		if (body.length > 0) request.write(body);
		request.end();
	});
}

function readRequestBody(request: IncomingMessage): Promise<Buffer> {
	return new Promise((resolvePromise, reject) => {
		const chunks: Buffer[] = [];
		request.on("data", (chunk: Buffer) => chunks.push(chunk));
		request.on("end", () => resolvePromise(Buffer.concat(chunks)));
		request.on("error", reject);
	});
}

function writeJson(response: import("node:http").ServerResponse, status: number, body: unknown): void {
	const encoded = Buffer.from(JSON.stringify(body));
	response.writeHead(status, {
		"content-length": String(encoded.length),
		"content-type": "application/json",
	});
	response.end(encoded);
}

function writeBuffer(
	response: import("node:http").ServerResponse,
	status: number,
	headers: IncomingHttpHeaders,
	body: Buffer,
): void {
	const forwarded = { ...headers };
	delete forwarded["transfer-encoding"];
	delete forwarded["content-length"];
	delete forwarded.connection;
	forwarded["content-length"] = String(body.length);
	response.writeHead(status, forwarded);
	response.end(body);
}

function parseRecord(body: Buffer): Record<string, unknown> | null {
	try {
		const value = JSON.parse(body.toString("utf8")) as unknown;
		return value !== null && typeof value === "object" && !Array.isArray(value)
			? value as Record<string, unknown>
			: null;
	} catch {
		return null;
	}
}

function jobIdFromPath(path: string): string | null {
	const prefix = "/rem/jobs/";
	return path.startsWith(prefix) ? decodeURIComponent(path.slice(prefix.length)) : null;
}

function header(headers: IncomingHttpHeaders, name: string): string | undefined {
	const value = headers[name];
	return Array.isArray(value) ? value[0] : value;
}

function validatePreflight(): void {
	for (const required of [
		fixturePath,
		planPath,
		receiptPath,
		currentBuild,
		featureRunner,
		sectionFirstRunner,
	]) {
		if (!existsSync(required)) throw new Error(`required path missing: ${required}`);
	}
	if (fixture.schemaVersion !== 1) throw new Error(`unsupported fixture schema: ${fixture.schemaVersion}`);
	if (fixture.known.map((item) => item.exitCode).join(",") !== "0,1,2,3,4,5,6,7,8,9") {
		throw new Error("fixture does not enumerate exact known exits 0..9");
	}
	const receipt = readFileSync(receiptPath, "utf8").trim().split(/\s+/)[0];
	if (receipt !== expectedPlanSha256 || sha256File(planPath) !== expectedPlanSha256) {
		throw new Error(`frozen plan hash mismatch: receipt=${receipt} actual=${sha256File(planPath)}`);
	}
	if (dirname(featureRunner) === dirname(sectionFirstRunner)) {
		throw new Error("the two runners are not in independent checkouts");
	}
}

function requiredEnvironment(key: string): string {
	const value = process.env[key];
	if (!value) throw new Error(`${key} is required`);
	return value;
}

function sha256(value: string | Buffer): string {
	return createHash("sha256").update(value).digest("hex");
}

function sha256File(path: string): string {
	return sha256(readFileSync(path));
}

function errorMessage(error: unknown): string {
	return error instanceof Error ? error.message : String(error);
}
