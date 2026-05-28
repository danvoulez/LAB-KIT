use crate::{
    commands::{self, ReportKind as CommandReportKind},
    labd,
};
use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::{net::SocketAddr, path::PathBuf};
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(
    name = "logline-lab",
    version,
    about = "Rust-native LogLine Lab Kit CLI/labd"
)]
pub struct Cli {
    #[arg(long, env = "SUPABASE_DB_URL", global = true)]
    database_url: Option<String>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Init(InitArgs),
    Doctor(DoctorArgs),
    Status,
    Canon {
        #[command(subcommand)]
        command: CanonCommand,
    },
    Act {
        #[command(subcommand)]
        command: ActCommand,
    },
    Ghost {
        #[command(subcommand)]
        command: GhostCommand,
    },
    Evidence {
        #[command(subcommand)]
        command: EvidenceCommand,
    },
    Receipt {
        #[command(subcommand)]
        command: ReceiptCommand,
    },
    Report {
        #[command(subcommand)]
        command: ReportCommand,
    },
    Projector {
        #[command(subcommand)]
        command: ProjectorCommand,
    },
    Hook {
        #[command(subcommand)]
        command: HookCommand,
    },
    Clock {
        #[command(subcommand)]
        command: ClockCommand,
    },
    Dispatch {
        #[command(subcommand)]
        command: DispatchCommand,
    },
    Workorder {
        #[command(subcommand)]
        command: WorkorderCommand,
    },
    Labd(LabdArgs),
}

#[derive(Args, Debug)]
struct InitArgs {
    #[arg(long, default_value = "manifests/lab.manifest.example.yaml")]
    manifest: PathBuf,
    #[arg(long)]
    no_seed: bool,
}

#[derive(Args, Debug)]
struct DoctorArgs {
    #[arg(long)]
    json: bool,
}

#[derive(Subcommand, Debug)]
enum CanonCommand {
    Status,
}

#[derive(Subcommand, Debug)]
enum ActCommand {
    Emit(FileArg),
    Get { id: Uuid },
}

#[derive(Subcommand, Debug)]
enum GhostCommand {
    List,
    Open(GhostOpenArgs),
}

#[derive(Args, Debug)]
struct GhostOpenArgs {
    #[arg(long)]
    key: String,
    #[arg(long)]
    what_missing: String,
    #[arg(long, default_value = "operator_observation")]
    source: String,
    #[arg(long, default_value = "operator")]
    who: String,
}

#[derive(Subcommand, Debug)]
enum EvidenceCommand {
    Add(FileArg),
    List,
}

#[derive(Subcommand, Debug)]
enum ReceiptCommand {
    Prepare(FileArg),
}

#[derive(Subcommand, Debug)]
enum ReportCommand {
    Generate { kind: ReportKind },
}

#[derive(Clone, Debug, ValueEnum)]
enum ReportKind {
    DailyExpedition,
    WeeklyReview,
}

impl From<ReportKind> for CommandReportKind {
    fn from(value: ReportKind) -> Self {
        match value {
            ReportKind::DailyExpedition => CommandReportKind::DailyExpedition,
            ReportKind::WeeklyReview => CommandReportKind::WeeklyReview,
        }
    }
}

#[derive(Subcommand, Debug)]
enum ProjectorCommand {
    Run {
        #[arg(long, default_value = "all")]
        name: String,
    },
}

#[derive(Subcommand, Debug)]
enum HookCommand {
    Run {
        #[arg(long)]
        hook: String,
        #[arg(long)]
        payload: Option<PathBuf>,
    },
}

#[derive(Subcommand, Debug)]
enum ClockCommand {
    Tick {
        #[arg(long, default_value = "daily")]
        kind: String,
    },
}

#[derive(Subcommand, Debug)]
enum DispatchCommand {
    Prepare {
        #[arg(long)]
        process: String,
    },
}

#[derive(Subcommand, Debug)]
enum WorkorderCommand {
    Prepare {
        #[arg(long)]
        dispatch_file: PathBuf,
    },
}

#[derive(Args, Debug)]
struct FileArg {
    #[arg(long)]
    file: PathBuf,
}

#[derive(Args, Debug)]
struct LabdArgs {
    #[arg(long, env = "LABD_BIND", default_value = "127.0.0.1:8787")]
    bind: SocketAddr,
}

pub async fn run() -> Result<()> {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();
    match cli.command {
        Command::Init(args) => {
            commands::init(cli.database_url.as_deref(), &args.manifest, args.no_seed).await
        }
        Command::Doctor(args) => commands::doctor(cli.database_url.as_deref(), args.json).await,
        Command::Status => commands::status(cli.database_url.as_deref()).await,
        Command::Canon {
            command: CanonCommand::Status,
        } => commands::canon_status(),
        Command::Act { command } => match command {
            ActCommand::Emit(args) => {
                commands::emit_from_file(cli.database_url.as_deref(), &args.file).await
            }
            ActCommand::Get { id } => commands::act_get(cli.database_url.as_deref(), id).await,
        },
        Command::Ghost { command } => match command {
            GhostCommand::List => {
                commands::list_projection(cli.database_url.as_deref(), "audit.v_open_ghosts").await
            }
            GhostCommand::Open(args) => {
                commands::ghost_open(
                    cli.database_url.as_deref(),
                    args.key,
                    args.what_missing,
                    args.source,
                    args.who,
                )
                .await
            }
        },
        Command::Evidence { command } => match command {
            EvidenceCommand::Add(args) => {
                commands::emit_from_file(cli.database_url.as_deref(), &args.file).await
            }
            EvidenceCommand::List => {
                commands::list_projection(cli.database_url.as_deref(), "evidence.records").await
            }
        },
        Command::Receipt {
            command: ReceiptCommand::Prepare(args),
        } => commands::receipt_prepare(cli.database_url.as_deref(), &args.file).await,
        Command::Report {
            command: ReportCommand::Generate { kind },
        } => commands::report_generate(cli.database_url.as_deref(), kind.into()).await,
        Command::Projector {
            command: ProjectorCommand::Run { name },
        } => commands::projector_run(cli.database_url.as_deref(), &name).await,
        Command::Hook {
            command: HookCommand::Run { hook, payload },
        } => commands::hook_run(&hook, payload.as_deref()),
        Command::Clock {
            command: ClockCommand::Tick { kind },
        } => commands::clock_tick(cli.database_url.as_deref(), &kind).await,
        Command::Dispatch {
            command: DispatchCommand::Prepare { process },
        } => commands::dispatch_prepare(&process),
        Command::Workorder {
            command: WorkorderCommand::Prepare { dispatch_file },
        } => commands::workorder_prepare(&dispatch_file),
        Command::Labd(args) => labd::serve(cli.database_url, args.bind).await,
    }
}
