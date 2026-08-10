#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RemError {
    Usage,
    JobFailed,
    Timeout,
    StateUnrecognised,
    ResponseInvalid,
    ResponseTruncated,
    SidecarNotRunning,
    SidecarUnauthorized,
    SidecarClient,
    SidecarDiscovery,
    SidecarDiscoveryInvalid,
    SidecarResponse,
    Profile,
    Trace,
    JobNotFound,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct RemErrorCode {
    kind: RemError,
    code: &'static str,
}

#[derive(Debug)]
pub(crate) struct RemOutcome {
    name: &'static str,
    pub(crate) exit_code: i32,
    error_codes: &'static [RemErrorCode],
}

impl RemOutcome {
    pub(crate) const fn name(&self) -> &'static str {
        self.name
    }
}

pub(crate) struct ResolvedRemError {
    pub(crate) code: &'static str,
    pub(crate) outcome: &'static RemOutcome,
}

pub(crate) const REM_OUTCOMES: &[RemOutcome] = &[
    RemOutcome {
        name: "success",
        exit_code: 0,
        error_codes: &[],
    },
    RemOutcome {
        name: "unclassified failure",
        exit_code: 1,
        error_codes: &[],
    },
    RemOutcome {
        name: "invalid usage",
        exit_code: 2,
        error_codes: &[RemErrorCode {
            kind: RemError::Usage,
            code: "usage_error",
        }],
    },
    RemOutcome {
        name: "job failed",
        exit_code: 3,
        error_codes: &[RemErrorCode {
            kind: RemError::JobFailed,
            code: "rem_job_failed",
        }],
    },
    RemOutcome {
        name: "wait deadline passed",
        exit_code: 4,
        error_codes: &[RemErrorCode {
            kind: RemError::Timeout,
            code: "rem_timeout",
        }],
    },
    RemOutcome {
        name: "state vocabulary mismatch",
        exit_code: 5,
        error_codes: &[RemErrorCode {
            kind: RemError::StateUnrecognised,
            code: "rem_state_unrecognised",
        }],
    },
    RemOutcome {
        name: "malformed or truncated response",
        exit_code: 6,
        error_codes: &[
            RemErrorCode {
                kind: RemError::ResponseInvalid,
                code: "sidecar_response_invalid",
            },
            RemErrorCode {
                kind: RemError::ResponseTruncated,
                code: "sidecar_response_truncated",
            },
        ],
    },
    RemOutcome {
        name: "sidecar failure",
        exit_code: 7,
        error_codes: &[
            RemErrorCode {
                kind: RemError::SidecarNotRunning,
                code: "sidecar_not_running",
            },
            RemErrorCode {
                kind: RemError::SidecarUnauthorized,
                code: "sidecar_unauthorized",
            },
            RemErrorCode {
                kind: RemError::SidecarClient,
                code: "sidecar_client_error",
            },
            RemErrorCode {
                kind: RemError::SidecarDiscovery,
                code: "sidecar_discovery_error",
            },
            RemErrorCode {
                kind: RemError::SidecarDiscoveryInvalid,
                code: "sidecar_discovery_invalid",
            },
            RemErrorCode {
                kind: RemError::SidecarResponse,
                code: "sidecar_response_error",
            },
        ],
    },
    RemOutcome {
        name: "local environment failure",
        exit_code: 8,
        error_codes: &[
            RemErrorCode {
                kind: RemError::Profile,
                code: "profile_error",
            },
            RemErrorCode {
                kind: RemError::Trace,
                code: "rem_trace_error",
            },
        ],
    },
    RemOutcome {
        name: "unknown job identifier",
        exit_code: 9,
        error_codes: &[RemErrorCode {
            kind: RemError::JobNotFound,
            code: "rem_job_not_found",
        }],
    },
];

pub(crate) fn resolve(error: RemError) -> ResolvedRemError {
    for outcome in REM_OUTCOMES {
        for candidate in outcome.error_codes {
            if candidate.kind == error {
                return ResolvedRemError {
                    code: candidate.code,
                    outcome,
                };
            }
        }
    }
    unreachable!("every typed REM error belongs to the declaration")
}

pub(crate) fn resolve_code(code: &str) -> Option<ResolvedRemError> {
    for outcome in REM_OUTCOMES {
        for candidate in outcome.error_codes {
            if candidate.code == code {
                return Some(ResolvedRemError {
                    code: candidate.code,
                    outcome,
                });
            }
        }
    }
    None
}

const fn strings_equal(left: &str, right: &str) -> bool {
    let left = left.as_bytes();
    let right = right.as_bytes();
    if left.len() != right.len() {
        return false;
    }
    let mut index = 0;
    while index < left.len() {
        if left[index] != right[index] {
            return false;
        }
        index += 1;
    }
    true
}

const fn validate_outcomes(outcomes: &[RemOutcome]) {
    let mut outcome_index = 0;
    while outcome_index < outcomes.len() {
        let outcome = &outcomes[outcome_index];
        assert!(!outcome.name.is_empty());
        if outcome.exit_code == 1 {
            assert!(outcome.error_codes.is_empty());
        } else if outcome.exit_code != 0 {
            assert!(!outcome.error_codes.is_empty());
        }

        let mut other_outcome_index = outcome_index + 1;
        while other_outcome_index < outcomes.len() {
            let other = &outcomes[other_outcome_index];
            assert!(outcome.exit_code != other.exit_code);

            let mut error_index = 0;
            while error_index < outcome.error_codes.len() {
                let mut other_error_index = 0;
                while other_error_index < other.error_codes.len() {
                    assert!(!strings_equal(
                        outcome.error_codes[error_index].code,
                        other.error_codes[other_error_index].code,
                    ));
                    other_error_index += 1;
                }
                error_index += 1;
            }
            other_outcome_index += 1;
        }
        outcome_index += 1;
    }
}

const _: () = validate_outcomes(REM_OUTCOMES);
