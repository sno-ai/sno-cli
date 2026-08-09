use std::env;
use std::thread;
use std::time::{Duration, Instant};

use reqwest::blocking::{Client, RequestBuilder};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::redirect::Policy;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use url::Url;
use uuid::Uuid;

use crate::cli::print_json;
use crate::error::CliError;
use crate::state::{self, Identity};

const DEFAULT_BASE_URL: &str = "https://www.sno.ai";
const DEVICE_CODE_GRANT_TYPE: &str = "urn:ietf:params:oauth:grant-type:device_code";
const CLAIM_TIMEOUT: Duration = Duration::from_secs(30 * 60);
const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(5);
const MAX_POLL_INTERVAL: Duration = Duration::from_secs(30);
const MAX_TRANSIENT_POLL_ERRORS: u8 = 3;

struct HttpResponse {
    status: u16,
    value: Option<Value>,
    body: String,
}

struct RegisterResult {
    claimed: bool,
    user_cuid: String,
    machine_uuid: String,
}

struct ClaimCode {
    device_code: String,
    user_code: String,
    verification_uri: String,
    verification_uri_complete: Option<String>,
    interval: Duration,
}

pub fn run_register(json_enabled: bool) -> Result<i32, CliError> {
    let identity = state::bootstrap_identity()?;
    let result = register_machine(&identity)?;
    if json_enabled {
        print_json(&json!({
            "registered": true,
            "claimed": result.claimed,
            "user_cuid": result.user_cuid,
            "machine_uuid": result.machine_uuid,
        }))?;
    } else {
        println!("registered");
        println!("user_cuid={}", result.user_cuid);
        println!("machine_uuid={}", result.machine_uuid);
        println!("claimed={}", result.claimed);
    }
    Ok(0)
}

pub fn run_claim(json_enabled: bool) -> Result<i32, CliError> {
    let identity = state::bootstrap_identity()?;
    register_machine(&identity)?;
    let code = request_claim_code(&identity)?;
    if json_enabled {
        let mut authorization = json!({
            "type": "authorization",
            "user_code": code.user_code,
            "verification_uri": code.verification_uri,
        });
        if let Some(complete) = &code.verification_uri_complete {
            authorization["verification_uri_complete"] = Value::String(complete.clone());
        }
        print_json(&authorization)?;
    } else {
        println!("verification_uri={}", code.verification_uri);
        if let Some(complete) = &code.verification_uri_complete {
            println!("verification_uri_complete={complete}");
        }
        println!("user_code={}", code.user_code);
        println!("waiting_for_approval=true");
    }
    let completion = poll_claim(&code).and_then(|account_id| {
        state::update_identity_account(&identity, &account_id)?;
        Ok(account_id)
    });
    let account_id = match completion {
        Ok(account_id) => account_id,
        Err(error) if json_enabled => {
            print_json(&json!({
                "type": "error",
                "error": error.code,
                "message": error.message,
            }))?;
            return Ok(error.exit_code());
        }
        Err(error) => return Err(error),
    };
    if json_enabled {
        print_json(&json!({
            "type": "result",
            "claimed": true,
            "user_account_id": account_id,
            "user_cuid": identity.user_cuid,
            "machine_uuid": identity.machine_uuid,
        }))?;
    } else {
        println!("claimed");
        println!("user_account_id={account_id}");
        println!("user_cuid={}", identity.user_cuid);
        println!("machine_uuid={}", identity.machine_uuid);
    }
    Ok(0)
}

pub fn run_audit_verify(event_id: &str, json_enabled: bool) -> Result<i32, CliError> {
    let identity = state::bootstrap_identity()?;
    register_machine(&identity)?;
    let mut url = endpoint("/api/v1/audit/verify")?;
    url.query_pairs_mut().append_pair("event_id", event_id);
    let response = send(
        client()?
            .get(url)
            .header(AUTHORIZATION, format!("Bearer {}", identity.machine_secret)),
    )?;
    if response.status == 404 {
        return Err(CliError::runtime(
            "not_found_or_unowned",
            "event not found or not owned",
        ));
    }
    let value = response.value.ok_or_else(|| {
        CliError::runtime(
            "audit_verify_failed",
            audit_error_message(response.status, None, &response.body),
        )
    })?;
    if response.status != 200 {
        return Err(CliError::runtime(
            "audit_verify_failed",
            audit_error_message(response.status, Some(&value), &response.body),
        ));
    }
    let verified = value
        .get("verified")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if json_enabled {
        print_json(&value)?;
    } else {
        println!(
            "{}",
            if verified {
                "✓ verified"
            } else {
                "✗ tampered"
            }
        );
        println!("{}", serde_json::to_string_pretty(&value)?);
    }
    Ok(if verified { 0 } else { 1 })
}

fn register_machine(identity: &Identity) -> Result<RegisterResult, CliError> {
    let url = endpoint("/api/v1/identity/register-machine")?;
    let secret_hash = hex::encode(Sha256::digest(identity.machine_secret.as_bytes()));
    let response = send(
        client()?
            .post(url)
            .header(CONTENT_TYPE, "application/json")
            .json(&json!({
                "user_cuid": identity.user_cuid,
                "machine_uuid": identity.machine_uuid,
                "machine_secret_hash": secret_hash,
            })),
    )?;
    let value = response.value.as_ref();
    if response.status != 200 || value.is_none() {
        let code = server_error_code(value).unwrap_or("machine_registration_failed");
        return Err(CliError::runtime(
            code,
            registration_error_message(response.status, value, &response.body),
        ));
    }
    let value = value.expect("checked above");
    let user_cuid = value
        .get("user_cuid")
        .and_then(Value::as_str)
        .filter(|value| is_cuid2(value))
        .ok_or_else(|| malformed_registration(response.status))?;
    let machine_uuid = value
        .get("machine_uuid")
        .and_then(Value::as_str)
        .filter(|value| is_uuid_v7(value))
        .ok_or_else(|| malformed_registration(response.status))?;
    let claimed = value
        .get("claimed")
        .and_then(Value::as_bool)
        .ok_or_else(|| malformed_registration(response.status))?;
    if user_cuid != identity.user_cuid || machine_uuid != identity.machine_uuid {
        return Err(CliError::runtime(
            "machine_registration_identity_mismatch",
            "machine registration returned a different identity",
        ));
    }
    if let Some(account_value) = value.get("user_account_id") {
        if !account_value.is_null() {
            let account_id = account_value
                .as_str()
                .filter(|value| is_cuid2(value))
                .ok_or_else(|| malformed_registration(response.status))?;
            state::update_identity_account(identity, account_id)?;
        }
    }
    Ok(RegisterResult {
        claimed,
        user_cuid: user_cuid.to_owned(),
        machine_uuid: machine_uuid.to_owned(),
    })
}

fn request_claim_code(identity: &Identity) -> Result<ClaimCode, CliError> {
    let url = endpoint("/api/v1/device/code")?;
    let response = send(
        client()?
            .post(url)
            .header(CONTENT_TYPE, "application/json")
            .json(&json!({
                "user_cuid": identity.user_cuid,
                "machine_uuid": identity.machine_uuid,
            })),
    )?;
    let value = response.value.as_ref();
    if response.status != 200 || value.is_none() {
        return Err(claim_http_error(
            "device code request",
            response.status,
            value,
            &response.body,
        ));
    }
    let value = value.expect("checked above");
    let device_code = required_string(value, "device_code")?;
    let user_code = required_string(value, "user_code")?;
    let verification_uri = required_https_url(value, "verification_uri")?;
    let verification_uri_complete = value
        .get("verification_uri_complete")
        .map(|_| required_https_url(value, "verification_uri_complete"))
        .transpose()?;
    let expires_in = value
        .get("expires_in")
        .and_then(Value::as_u64)
        .filter(|value| *value > 0)
        .ok_or_else(|| CliError::runtime("claim_failed", "invalid device code response"))?;
    let _ = expires_in;
    let interval_seconds = value
        .get("interval")
        .and_then(Value::as_u64)
        .filter(|value| *value >= 1)
        .unwrap_or(DEFAULT_POLL_INTERVAL.as_secs());
    Ok(ClaimCode {
        device_code,
        user_code,
        verification_uri,
        verification_uri_complete,
        interval: normalize_poll_delay(Duration::from_secs(interval_seconds)),
    })
}

fn poll_claim(code: &ClaimCode) -> Result<String, CliError> {
    let started = Instant::now();
    let mut delay = code.interval;
    let mut network_delay = delay;
    let mut transient_errors = 0_u8;
    loop {
        if started.elapsed() >= CLAIM_TIMEOUT {
            return Err(CliError::runtime(
                "claim_timeout",
                "device authorization timed out",
            ));
        }
        let response = request_claim_token(&code.device_code);
        let response = match response {
            Ok(response) => response,
            Err(error) if error.code == "network_error" => {
                transient_errors += 1;
                if transient_errors > MAX_TRANSIENT_POLL_ERRORS {
                    return Err(CliError::runtime(
                        "claim_poll_network_error",
                        format!(
                            "device token request failed after {MAX_TRANSIENT_POLL_ERRORS} retries: {}",
                            error.message
                        ),
                    ));
                }
                sleep_claim(network_delay, started)?;
                network_delay = normalize_poll_delay(network_delay.saturating_mul(2));
                continue;
            }
            Err(error) => return Err(error),
        };
        if is_transient_claim_status(response.status) {
            transient_errors += 1;
            if transient_errors > MAX_TRANSIENT_POLL_ERRORS {
                let error = claim_http_error(
                    "device token request",
                    response.status,
                    response.value.as_ref(),
                    &response.body,
                );
                return Err(CliError::runtime(
                    "claim_poll_http_error",
                    format!(
                        "device token request failed after {MAX_TRANSIENT_POLL_ERRORS} retries: {}",
                        error.message
                    ),
                ));
            }
            sleep_claim(network_delay, started)?;
            network_delay = normalize_poll_delay(network_delay.saturating_mul(2));
            continue;
        }
        transient_errors = 0;
        network_delay = delay;
        if response.status == 200 {
            if let Some(account_id) = response
                .value
                .as_ref()
                .and_then(|value| value.get("user_account_id"))
                .and_then(Value::as_str)
                .filter(|value| is_cuid2(value))
            {
                return Ok(account_id.to_owned());
            }
        }
        let error_code = server_error_code(response.value.as_ref());
        if response.status == 400 && error_code == Some("authorization_pending") {
            sleep_claim(delay, started)?;
            continue;
        }
        if response.status == 400 && error_code == Some("slow_down") {
            delay = response
                .value
                .as_ref()
                .and_then(|value| value.get("interval"))
                .and_then(Value::as_u64)
                .map(Duration::from_secs)
                .map(normalize_poll_delay)
                .unwrap_or_else(|| normalize_poll_delay(delay + Duration::from_secs(5)));
            sleep_claim(delay, started)?;
            continue;
        }
        return Err(claim_http_error(
            "device token request",
            response.status,
            response.value.as_ref(),
            &response.body,
        ));
    }
}

fn request_claim_token(device_code: &str) -> Result<HttpResponse, CliError> {
    let url = endpoint("/api/v1/device/token")?;
    send(
        client()?
            .post(url)
            .header(CONTENT_TYPE, "application/json")
            .json(&json!({
                "device_code": device_code,
                "grant_type": DEVICE_CODE_GRANT_TYPE,
            })),
    )
}

fn client() -> Result<Client, CliError> {
    Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(Policy::none())
        .build()
        .map_err(network_error)
}

fn send(request: RequestBuilder) -> Result<HttpResponse, CliError> {
    let response = request.send().map_err(network_error)?;
    let status = response.status().as_u16();
    let body = response.text().map_err(network_error)?;
    let value = (!body.is_empty())
        .then(|| serde_json::from_str(&body).ok())
        .flatten();
    Ok(HttpResponse {
        status,
        value,
        body,
    })
}

fn base_url() -> Result<Url, CliError> {
    normalize_base_url(
        &env::var("SNO_OBSERVE_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_owned()),
    )
}

fn endpoint(path: &str) -> Result<Url, CliError> {
    let base = base_url()?;
    Url::parse(&format!("{}{path}", base.as_str().trim_end_matches('/'))).map_err(|error| {
        CliError::runtime("transport_error", format!("invalid service URL: {error}"))
    })
}

pub(crate) fn normalize_base_url(input: &str) -> Result<Url, CliError> {
    let trimmed = input.trim_end_matches('/');
    let parsed = Url::parse(trimmed).map_err(|_| {
        CliError::runtime(
            "transport_error",
            format!("invalid SNO_OBSERVE_BASE_URL: {input}"),
        )
    })?;
    let host = parsed.host_str().unwrap_or_default();
    if parsed.scheme() == "https"
        || (parsed.scheme() == "http"
            && matches!(host, "localhost" | "127.0.0.1" | "::1" | "[::1]"))
    {
        return Ok(parsed);
    }
    Err(CliError::runtime(
        "transport_error",
        format!(
            "SNO_OBSERVE_BASE_URL must use https:// (got {}://{})",
            parsed.scheme(),
            host
        ),
    ))
}

fn network_error(error: reqwest::Error) -> CliError {
    CliError::runtime("network_error", format!("network error: {error}"))
}

fn malformed_registration(status: u16) -> CliError {
    CliError::runtime(
        "machine_registration_failed",
        format!("machine registration failed with HTTP {status}: invalid response"),
    )
}

fn registration_error_message(status: u16, value: Option<&Value>, body: &str) -> String {
    let detail = server_error_detail(value).unwrap_or_else(|| body.trim());
    if detail.is_empty() {
        format!("machine registration failed with HTTP {status}")
    } else {
        format!("machine registration failed with HTTP {status}: {detail}")
    }
}

fn claim_http_error(action: &str, status: u16, value: Option<&Value>, body: &str) -> CliError {
    let detail = server_error_detail(value).unwrap_or_else(|| body.trim());
    let message = if detail.is_empty() {
        format!("{action} failed with HTTP {status}")
    } else {
        format!("{action} failed with HTTP {status}: {detail}")
    };
    CliError::runtime(server_error_code(value).unwrap_or("claim_failed"), message)
}

fn audit_error_message(status: u16, value: Option<&Value>, body: &str) -> String {
    let detail = server_error_detail(value).unwrap_or_else(|| body.trim());
    if detail.is_empty() {
        format!("audit verify failed with HTTP {status}")
    } else {
        format!("audit verify failed with HTTP {status}: {detail}")
    }
}

fn server_error_code(value: Option<&Value>) -> Option<&str> {
    value?.get("error")?.as_str()
}

fn server_error_detail(value: Option<&Value>) -> Option<&str> {
    value?
        .get("message")
        .and_then(Value::as_str)
        .or_else(|| value?.get("error").and_then(Value::as_str))
}

fn required_string(value: &Value, key: &str) -> Result<String, CliError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| CliError::runtime("claim_failed", "invalid device code response"))
}

fn required_https_url(value: &Value, key: &str) -> Result<String, CliError> {
    let raw = required_string(value, key)?;
    if Url::parse(&raw).is_ok_and(|url| url.scheme() == "https") {
        Ok(raw)
    } else {
        Err(CliError::runtime(
            "claim_failed",
            "invalid device code response",
        ))
    }
}

fn normalize_poll_delay(delay: Duration) -> Duration {
    delay.clamp(Duration::from_secs(1), MAX_POLL_INTERVAL)
}

fn is_transient_claim_status(status: u16) -> bool {
    matches!(status, 408 | 425 | 429 | 500 | 502 | 503 | 504)
}

fn sleep_claim(delay: Duration, started: Instant) -> Result<(), CliError> {
    let remaining = CLAIM_TIMEOUT.saturating_sub(started.elapsed());
    if remaining.is_zero() {
        return Err(CliError::runtime(
            "claim_timeout",
            "device authorization timed out",
        ));
    }
    thread::sleep(delay.min(remaining));
    Ok(())
}

fn is_cuid2(value: &str) -> bool {
    value.len() == 24
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
}

fn is_uuid_v7(value: &str) -> bool {
    value == value.to_ascii_lowercase()
        && Uuid::parse_str(value).is_ok_and(|uuid| uuid.get_version_num() == 7)
}

#[cfg(test)]
mod tests {
    use super::normalize_base_url;

    #[test]
    fn transport_accepts_https_and_loopback_only() {
        for accepted in [
            "https://www.sno.ai",
            "http://localhost",
            "http://127.0.0.1",
            "http://[::1]",
        ] {
            assert!(normalize_base_url(accepted).is_ok(), "{accepted}");
        }
        for rejected in ["http://www.sno.ai", "ftp://localhost", "not-a-url"] {
            assert!(normalize_base_url(rejected).is_err(), "{rejected}");
        }
    }
}
