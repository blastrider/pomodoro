// bin/pomodoro.rs
use anyhow::Context;
use clap::Parser;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

// import depuis la lib (crate name from Cargo.toml: "pomodoro-cli" -> pomodoro_cli)
use pomodoro_cli::{CliArgs, Config, Journal, SessionRunner};

#[derive(Parser, Debug)]
#[command(name = "pomodoro-cli", about = "CLI Pomodoro — offline, journal local")]
struct Cli {
    /// Focus minutes (default 25)
    #[arg(long)]
    focus: Option<u64>,

    /// Short break minutes (default 5)
    #[arg(long)]
    short: Option<u64>,

    /// Long break minutes (default 15)
    #[arg(long)]
    long: Option<u64>,

    /// Cycles before long break (default 4)
    #[arg(long)]
    cycles: Option<u8>,

    /// Task label (<=80 chars)
    #[arg(long)]
    task: Option<String>,

    /// Play a beep on transitions
    #[arg(long, default_value_t = false)]
    beep: bool,

    /// Use desktop notifications (feature notify)
    #[arg(long, default_value_t = false)]
    notify: bool,

    /// Export today's journal to markdown
    #[arg(long)]
    export_md: bool,

    /// Export today's journal to csv
    #[arg(long)]
    export_csv: bool,

    /// Preset file (yaml/json) path
    #[arg(long)]
    preset: Option<std::path::PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // init tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    info!("Starting pomodoro");

    // --- map clap's `Cli` into the library-level `CliArgs` DTO
    let lib_cli = CliArgs {
        focus: cli.focus,
        short: cli.short,
        long: cli.long,
        cycles: cli.cycles,
        task: cli.task.clone(),
        preset: cli.preset.clone(),
    };

    // call lib API with the mapped type
    let cfg = Config::from_cli_and_preset(&lib_cli)
        .context("Failed to build configuration from CLI/preset")?;

    let journal = Journal::open_default().context("opening journal")?;
    let mut runner = SessionRunner::new(cfg, journal, cli.beep, cli.notify);

    // ctrlc handling: ensure save on interrupt
    runner.install_ctrlc_handler()?;

    let result = runner.run().await;

    match result {
        Ok(meta) => {
            info!("Session finished: {:?}", meta);
        }
        Err(e) => {
            warn!("Session ended with error: {:?}", e);
        }
    }

    // exports if requested
    if cli.export_md {
        runner.export_markdown().context("export md")?;
    }
    if cli.export_csv {
        runner.export_csv().context("export csv")?;
    }

    Ok(())
}
