use std::ffi::OsString;
use std::io::{self, Write};
use std::process::Command;

use clap::error::ErrorKind;
use clap::{CommandFactory, Parser, Subcommand};
use serde_json::{Value, json};

use crate::doctor;
use crate::error::CliError;
use crate::export::{self, ExportFormat};
use crate::service;
use crate::state::{self, ConsentValue};

const RETIRED_ROOT_COMMANDS: &[&str] =
    &["consent", "observe", "register", "claim", "audit", "doctor"];

#[derive(Debug, Parser)]
#[command(
    name = "sno",
    version,
    about = "The unified command-line interface for SNO",
    long_about = "The unified command-line interface for SNO\n\nAdditional commands can be installed as external subcommands named sno-<command>."
)]
struct SnoCli {
    #[arg(long, global = true, help = "Emit stable JSON output")]
    json: bool,

    #[command(subcommand)]
    command: Option<RootCommand>,
}

#[derive(Debug, Subcommand)]
enum RootCommand {
    #[command(about = "Manage account and machine identity")]
    Account {
        #[command(subcommand)]
        command: AccountCommand,
    },
    #[command(name = "station", about = "Manage this SNO Station")]
    SnoStation {
        #[command(subcommand)]
        command: SnoStationCommand,
    },
    #[command(name = "starport", about = "Manage SNO extensions")]
    SnoStarport,
    #[command(external_subcommand)]
    External(Vec<OsString>),
}

#[derive(Debug, Subcommand)]
enum AccountCommand {
    #[command(about = "Manage this machine's account identity")]
    Machine {
        #[command(subcommand)]
        command: MachineCommand,
    },
}

#[derive(Debug, Subcommand)]
enum MachineCommand {
    #[command(about = "Register this machine anonymously")]
    Register,
    #[command(about = "Claim this machine to a SNO account")]
    Claim,
}

#[derive(Debug, Subcommand)]
enum SnoStationCommand {
    #[command(about = "Manage local telemetry")]
    Telemetry {
        #[command(subcommand)]
        command: TelemetryCommand,
    },
    #[command(about = "Verify audit events")]
    Audit {
        #[command(subcommand)]
        command: AuditCommand,
    },
    #[command(about = "Run local SNO diagnostics")]
    Doctor,
}

#[derive(Debug, Subcommand)]
enum TelemetryCommand {
    #[command(about = "Manage local telemetry consent")]
    Consent {
        #[command(subcommand)]
        command: ConsentCommand,
    },
    #[command(about = "Pause cloud telemetry")]
    Pause,
    #[command(about = "Resume cloud telemetry")]
    Resume,
    #[command(about = "Export local audit events")]
    Export {
        #[arg(value_name = "PATH")]
        path: Option<String>,
        #[arg(long, value_name = "PATH")]
        out: Option<String>,
        #[arg(long, value_name = "FORMAT")]
        format: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
enum ConsentCommand {
    #[command(about = "Print current consent")]
    Get,
    #[command(about = "Set consent to off, metadata-only, or full")]
    Set { value: String },
}

#[derive(Debug, Subcommand)]
enum AuditCommand {
    #[command(about = "Verify a server-stored event")]
    Verify { event_id: String },
}

pub fn run<I, T>(arguments: I) -> i32
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let arguments: Vec<OsString> = arguments.into_iter().map(Into::into).collect();
    let json_enabled = arguments.iter().any(|argument| argument == "--json");
    let parsed = match SnoCli::try_parse_from(arguments.clone()) {
        Ok(parsed) => parsed,
        Err(error) => return print_parse_error(error, json_enabled, &arguments),
    };
    let result = match parsed.command {
        None => return print_missing_command(parsed.json),
        Some(command) => dispatch(command, parsed.json),
    };
    match result {
        Ok(exit_code) => exit_code,
        Err(error) => print_cli_error(&error, parsed.json),
    }
}

fn dispatch(command: RootCommand, json_enabled: bool) -> Result<i32, CliError> {
    match command {
        RootCommand::Account { command } => dispatch_account(command, json_enabled),
        RootCommand::SnoStation { command } => dispatch_sno_station(command, json_enabled),
        RootCommand::SnoStarport => Err(CliError::usage("no starport verbs are released yet")),
        RootCommand::External(arguments) => dispatch_external(arguments),
    }
}

fn dispatch_account(command: AccountCommand, json_enabled: bool) -> Result<i32, CliError> {
    match command {
        AccountCommand::Machine { command } => match command {
            MachineCommand::Register => service::run_register(json_enabled),
            MachineCommand::Claim => service::run_claim(json_enabled),
        },
    }
}

fn dispatch_sno_station(command: SnoStationCommand, json_enabled: bool) -> Result<i32, CliError> {
    match command {
        SnoStationCommand::Telemetry { command } => dispatch_telemetry(command, json_enabled),
        SnoStationCommand::Audit { command } => match command {
            AuditCommand::Verify { event_id } => service::run_audit_verify(&event_id, json_enabled),
        },
        SnoStationCommand::Doctor => doctor::run(json_enabled),
    }
}

fn dispatch_telemetry(command: TelemetryCommand, json_enabled: bool) -> Result<i32, CliError> {
    match command {
        TelemetryCommand::Consent { command } => match command {
            ConsentCommand::Get => {
                let consent = state::read_consent()?;
                if json_enabled {
                    print_json(&json!({ "consent": consent }))?;
                } else {
                    println!("{consent}");
                }
                Ok(0)
            }
            ConsentCommand::Set { value } => {
                let consent = ConsentValue::parse_cli(&value)?;
                let previous = state::read_consent()?;
                state::set_consent(consent, "sno cli consent set")?;
                if json_enabled {
                    print_json(&json!({
                        "consent": consent,
                        "chain_epoch_advanced": previous != consent,
                    }))?;
                } else {
                    println!("{consent}");
                }
                Ok(0)
            }
        },
        TelemetryCommand::Pause => {
            let (consent, already_paused) = state::pause_telemetry()?;
            if json_enabled {
                print_json(&json!({
                    "consent": consent,
                    "paused": consent == ConsentValue::Off,
                    "already_paused": already_paused,
                }))?;
            } else {
                println!(
                    "{}",
                    if already_paused {
                        "already paused"
                    } else {
                        "paused"
                    }
                );
                println!("{consent}");
            }
            Ok(0)
        }
        TelemetryCommand::Resume => {
            let consent = state::resume_telemetry()?;
            if json_enabled {
                print_json(&json!({ "consent": consent }))?;
            } else {
                println!("resumed: {consent}");
            }
            Ok(0)
        }
        TelemetryCommand::Export { path, out, format } => {
            if path.is_some() && out.is_some() {
                return Err(CliError::usage(
                    "provide either a positional path or --out, not both",
                ));
            }
            export::run(
                path.or(out),
                format.as_deref().map(parse_export_format).transpose()?,
                json_enabled,
            )
        }
    }
}

fn dispatch_external(arguments: Vec<OsString>) -> Result<i32, CliError> {
    let (name, child_arguments) = arguments
        .split_first()
        .ok_or_else(|| CliError::usage("missing external subcommand"))?;
    let name = name
        .to_str()
        .ok_or_else(|| CliError::usage("external subcommand name is not valid UTF-8"))?;
    if RETIRED_ROOT_COMMANDS.contains(&name) {
        return Err(CliError::usage(format!(
            "'{name}' is not a top-level command; run 'sno --help'"
        )));
    }
    let executable = format!("sno-{name}");
    match Command::new(&executable).args(child_arguments).status() {
        Ok(status) => Ok(status.code().unwrap_or(1)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Err(CliError::usage(format!(
            "unknown command '{name}'; external executable '{executable}' was not found on PATH"
        ))),
        Err(error) => Err(CliError::runtime(
            "external_command_failed",
            error.to_string(),
        )),
    }
}

fn print_missing_command(json_enabled: bool) -> i32 {
    if json_enabled {
        let _ = print_json(&json!({ "error": "usage_error", "message": "missing command" }));
    } else {
        let mut command = SnoCli::command();
        let _ = command.print_help();
        println!();
    }
    2
}

fn print_parse_error(error: clap::Error, json_enabled: bool, arguments: &[OsString]) -> i32 {
    match error.kind() {
        ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
            print!("{error}");
            0
        }
        _ if json_enabled => {
            let message = normalize_clap_message(&error, arguments);
            let _ = print_json(&json!({ "error": "usage_error", "message": message }));
            2
        }
        _ => {
            eprintln!("error: {}", normalize_clap_message(&error, arguments));
            2
        }
    }
}

fn normalize_clap_message(error: &clap::Error, arguments: &[OsString]) -> String {
    if error.kind() == ErrorKind::MissingRequiredArgument
        && contains_sequence(arguments, &["station", "audit", "verify"])
    {
        return "missing required argument 'event_id'".to_owned();
    }
    let first_line = error
        .to_string()
        .lines()
        .next()
        .unwrap_or("invalid command")
        .trim_start_matches("error: ")
        .trim_start_matches("error: ")
        .trim_matches('`')
        .to_owned();
    if let Some(argument) = first_line
        .strip_prefix("unexpected argument '")
        .and_then(|value| value.strip_suffix("' found"))
    {
        return format!("unknown option '{argument}'");
    }
    first_line
}

fn parse_export_format(value: &str) -> Result<ExportFormat, CliError> {
    match value {
        "tarball" => Ok(ExportFormat::Tarball),
        "jsonl" => Ok(ExportFormat::Jsonl),
        "csv" => Ok(ExportFormat::Csv),
        _ => Err(CliError::usage(
            "invalid export format: expected one of tarball, jsonl, csv",
        )),
    }
}

fn contains_sequence(arguments: &[OsString], sequence: &[&str]) -> bool {
    arguments.windows(sequence.len()).any(|window| {
        window
            .iter()
            .zip(sequence)
            .all(|(value, expected)| value == expected)
    })
}

fn print_cli_error(error: &CliError, json_enabled: bool) -> i32 {
    if json_enabled {
        let _ = print_json(&json!({ "error": error.code, "message": error.message }));
    } else {
        eprintln!("error: {}", error.message);
    }
    error.exit_code
}

pub(crate) fn print_json(value: &Value) -> Result<(), CliError> {
    let mut stdout = io::stdout().lock();
    serde_json::to_writer(&mut stdout, value)?;
    stdout.write_all(b"\n")?;
    Ok(())
}
