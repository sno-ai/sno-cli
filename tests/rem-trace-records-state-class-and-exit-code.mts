import { spawn } from "node:child_process";
import { randomUUID } from "node:crypto";
import { once } from "node:events";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { createServer, type IncomingMessage, type ServerResponse } from "node:http";
import type { AddressInfo } from "node:net";

interface CommandResult {
	status: number;
	stdout: string;
	stderr: string;
}

interface TraceRecord {
	component?: unknown;
	correlation_id?: unknown;
	raw_state?: unknown;
	outcome_class?: unknown;
	exit_code?: unknown;
	[key: string]: unknown;
}

class ExpectedProductGap extends Error {}

const repoRoot = "/home/lh/code/sno-cli";
const externalRunner = "/home/lh/code/sno-station-core-edge-rem-wave/evals/memora/scripts/run_rem.sh";
const binary = `${repoRoot}/target/debug/sno`;
const runRoot = requiredEnvironment("QCG14_RUN_ROOT");
const profileRoot = `${runRoot}/profile`;
const stateRoot = `${runRoot}/state`;
const tracePath = `${stateRoot}/mem-claw/rem-trace.jsonl`;
const token = `qcg14-token-${randomUUID()}`;
const correlationId = `qcg14-correlation-${randomUUID()}`;
const jobId = `rem-qcg14-${randomUUID()}`;
const scope = `persona:qcg14-${randomUUID()}`;
const rawState = "future terminal/β: sidecar_response_invalid; exit 0";
const outcomeClass = "state vocabulary mismatch";
const requests: Array<{ method: string; path: string; token: string | undefined; correlation: string | undefined; body: unknown }> = [];
let handlerError: Error | undefined;

void main().catch((error: unknown) => {
	if (error instanceof ExpectedProductGap) {
		console.error(error.message);
	} else {
		console.error(`QCG-14 BLOCKED: ${errorMessage(error)}`);
	}
	process.exitCode = 1;
});

async function main(): Promise<void> {
	for (const path of [externalRunner, binary]) {
		if (!existsSync(path)) throw new Error(`required path missing: ${path}`);
	}

	const server = createServer((request, response) => {
		void handleRequest(request, response).catch((error: unknown) => {
			handlerError = error instanceof Error ? error : new Error(String(error));
			response.statusCode = 500;
			response.end("fixture failure");
		});
	});
	server.listen(0, "127.0.0.1");
	await once(server, "listening");
	const address = server.address() as AddressInfo;
	mkdirSync(`${profileRoot}/station`, { recursive: true });
	mkdirSync(stateRoot, { recursive: true });
	writeFileSync(
		`${profileRoot}/station/sidecar.json`,
		`${JSON.stringify({ pid: process.pid, port: address.port, token })}\n`,
		{ mode: 0o600 },
	);

	let result: CommandResult;
	try {
		result = await runOrdinaryRunner();
	} finally {
		server.close();
		await once(server, "close");
	}

	if (handlerError !== undefined) throw handlerError;
	assertOrdinaryRequests();
	if (result.status !== 5) {
		throw new Error(`runner exit ${result.status}, expected 5; stdout=${JSON.stringify(result.stdout)} stderr=${JSON.stringify(result.stderr)}`);
	}
	if (!result.stdout.includes(rawState)) {
		throw new Error(`runner stdout lost byte-identical raw state: ${JSON.stringify(result.stdout)}`);
	}
	if (!existsSync(tracePath)) throw new Error(`trace was not written: ${tracePath}`);

	const records = readTraceRecords();
	const cliRecords = records.filter((record) => record.component === "sno_cli" && record.correlation_id === correlationId);
	const harnessRecords = records.filter((record) => record.component === "memora_harness" && record.correlation_id === correlationId);
	if (cliRecords.length === 0 || harnessRecords.length === 0) {
		throw new Error(`missing logical trace stream: sno_cli=${cliRecords.length} memora_harness=${harnessRecords.length}`);
	}

	console.log(`QCG-14 boundary_reached requests=${requests.length} runner_exit=${result.status} trace_records=${records.length}`);
	const gaps = [
		describeTupleGap("sno_cli", cliRecords),
		describeTupleGap("memora_harness", harnessRecords),
	].filter((gap): gap is string => gap !== undefined);
	if (gaps.length > 0) {
		throw new ExpectedProductGap(`QCG-14 RED: ${gaps.join("; ")}`);
	}

	console.log(`QCG-14 PASS raw_state=${JSON.stringify(rawState)} outcome_class=${JSON.stringify(outcomeClass)} exit_code=5`);
}

async function handleRequest(request: IncomingMessage, response: ServerResponse): Promise<void> {
	const body = await readBody(request);
	let parsedBody: unknown = null;
	if (body.length > 0) parsedBody = JSON.parse(body);
	requests.push({
		method: request.method ?? "",
		path: request.url ?? "",
		token: header(request, "x-sidecar-token"),
		correlation: header(request, "x-rem-correlation-id"),
		body: parsedBody,
	});

	if (request.headers["x-sidecar-token"] !== token) {
		writeJson(response, 401, { error: "unauthorized" });
		return;
	}
	if (request.method === "POST" && request.url === "/rem/run") {
		writeJson(response, 202, { job_id: jobId });
		return;
	}
	if (request.method === "GET" && request.url === `/rem/jobs/${jobId}`) {
		writeJson(response, 200, {
			state: rawState,
			type: "rem-update",
			scope,
			started_at: "2026-08-10T12:00:00Z",
			finished_at: "2026-08-10T12:00:01Z",
			stats: { operations: 1 },
			error: null,
			correlation_id: correlationId,
		});
		return;
	}
	writeJson(response, 404, { error: "not_found" });
}

function assertOrdinaryRequests(): void {
	if (requests.length !== 2) throw new Error(`fixture saw ${requests.length} requests, expected 2`);
	const [start, status] = requests;
	if (start.method !== "POST" || start.path !== "/rem/run") {
		throw new Error(`unexpected start request: ${JSON.stringify(start)}`);
	}
	if (status.method !== "GET" || status.path !== `/rem/jobs/${jobId}`) {
		throw new Error(`unexpected status request: ${JSON.stringify(status)}`);
	}
	for (const request of requests) {
		if (request.token !== token) throw new Error(`request used wrong token: ${JSON.stringify(request)}`);
		if (request.correlation !== correlationId) throw new Error(`request used wrong correlation: ${JSON.stringify(request)}`);
	}
	const startBody = start.body as Record<string, unknown>;
	if (startBody["type"] !== "rem-update" || startBody["scope"] !== scope) {
		throw new Error(`unexpected start body: ${JSON.stringify(start.body)}`);
	}
}

async function runOrdinaryRunner(): Promise<CommandResult> {
	const child = spawn("bash", [externalRunner, scope], {
		cwd: repoRoot,
		env: {
			...process.env,
			MEM_CLAW_REM_TYPE: "rem-update",
			OPENCLAW_STATE_DIR: stateRoot,
			SNO_CLI_BIN: binary,
			SNO_PROFILE_DIR: profileRoot,
			SNO_REM_CORRELATION_ID: correlationId,
			SNO_REM_TRACE: "1",
		},
		stdio: ["ignore", "pipe", "pipe"],
	});
	const stdout: Buffer[] = [];
	const stderr: Buffer[] = [];
	child.stdout.on("data", (chunk: Buffer) => stdout.push(chunk));
	child.stderr.on("data", (chunk: Buffer) => stderr.push(chunk));
	const [code, signal] = await once(child, "exit") as [number | null, NodeJS.Signals | null];
	if (signal !== null || code === null) throw new Error(`runner terminated by ${signal ?? "unknown signal"}`);
	return {
		status: code,
		stdout: Buffer.concat(stdout).toString("utf8"),
		stderr: Buffer.concat(stderr).toString("utf8"),
	};
}

function readTraceRecords(): TraceRecord[] {
	return readFileSync(tracePath, "utf8")
		.split("\n")
		.filter(Boolean)
		.map((line, index) => {
			try {
				return JSON.parse(line) as TraceRecord;
			} catch (error) {
				throw new Error(`trace line ${index + 1} is not JSON: ${errorMessage(error)}`);
			}
		});
}

function describeTupleGap(component: string, records: TraceRecord[]): string | undefined {
	if (records.some((record) =>
		record.raw_state === rawState
		&& record.outcome_class === outcomeClass
		&& record.exit_code === 5
	)) return undefined;

	const missing = [
		records.some((record) => record.raw_state === rawState) ? undefined : "raw_state",
		records.some((record) => record.outcome_class === outcomeClass) ? undefined : "outcome_class",
		records.some((record) => record.exit_code === 5) ? undefined : "exit_code",
	].filter((field): field is string => field !== undefined);
	if (missing.length === 0) return `${component} complete tuple is split across records`;
	return `${component} missing tuple fields [${missing.join(",")}]`;
}

function header(request: IncomingMessage, name: string): string | undefined {
	const value = request.headers[name];
	return Array.isArray(value) ? value[0] : value;
}

function readBody(request: IncomingMessage): Promise<string> {
	return new Promise((resolve, reject) => {
		const chunks: Buffer[] = [];
		request.on("data", (chunk: Buffer) => chunks.push(chunk));
		request.on("end", () => resolve(Buffer.concat(chunks).toString("utf8")));
		request.on("error", reject);
	});
}

function writeJson(response: ServerResponse, status: number, body: unknown): void {
	const encoded = JSON.stringify(body);
	response.writeHead(status, {
		"content-length": Buffer.byteLength(encoded),
		"content-type": "application/json",
	});
	response.end(encoded);
}

function requiredEnvironment(key: string): string {
	const value = process.env[key];
	if (!value) throw new Error(`${key} is required`);
	return value;
}

function errorMessage(error: unknown): string {
	return error instanceof Error ? error.stack ?? error.message : String(error);
}
