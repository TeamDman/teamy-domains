pub mod cache;
pub mod domain;
pub mod facet_shape;
pub mod global_args;
pub mod home;
pub mod nameserver;
pub mod output;
pub mod record;

use crate::cli::cache::CacheArgs;
use crate::cli::domain::DomainArgs;
use crate::cli::global_args::GlobalArgs;
use crate::cli::home::HomeArgs;
use crate::cli::nameserver::NameserverArgs;
use crate::cli::output::CliOutput;
use crate::cli::record::RecordArgs;
use arbitrary::Arbitrary;
use eyre::Context;
use facet::Facet;
use figue::FigueBuiltins;
use figue::{self as args};
use teamy_cancellation::CancellationScope;

/// Inspect synthetic registrar/DNS snapshots and plan domain changes offline.
/// No account access, broker connection, or live writes are available.
/// Namecheap XML decoding is not implemented; the provider adapter uses mock trees only.
///
/// Environment variables:
/// - `TEAMY_DOMAINS_HOME` overrides the resolved application home directory.
/// - `TEAMY_DOMAINS_CACHE` overrides the resolved cache directory.
/// - `RUST_LOG` provides a tracing filter when `--log-filter` is omitted.
#[derive(Facet, Arbitrary, Debug)]
pub struct Cli {
    /// Global arguments (`debug`, `log_filter`, `log_file`).
    #[facet(flatten)]
    pub global_args: GlobalArgs,

    /// Standard CLI options (help, version, completions).
    #[facet(flatten)]
    #[arbitrary(default)]
    pub builtins: FigueBuiltins,

    /// The command to run.
    #[facet(args::subcommand)]
    pub command: Command,
}

impl Cli {
    /// # Errors
    ///
    /// This function will return an error if the tokio runtime cannot be built or if the command fails.
    pub fn invoke(self, cancellation: &CancellationScope) -> eyre::Result<CliOutput> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .wrap_err("Failed to build tokio runtime")?;
        runtime.block_on(cancellation.cancellable_scope(|cancellation| async move {
            self.command.invoke(&cancellation).await
        }))
    }
}

/// A demonstration command line utility.
///
#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[repr(u8)]
pub enum Command {
    /// Cache-related commands.
    Cache(CacheArgs),
    /// Inspect an offline domain snapshot.
    Domain(DomainArgs),
    /// Home-related commands.
    Home(HomeArgs),
    /// Inspect or plan registrar nameserver delegation offline.
    Nameserver(NameserverArgs),
    /// Inspect or plan Namecheap hosted DNS records offline.
    Record(RecordArgs),
}

impl Command {
    /// # Errors
    ///
    /// This function will return an error if the subcommand fails.
    pub async fn invoke(self, _cancellation: &CancellationScope) -> eyre::Result<CliOutput> {
        match self {
            Command::Cache(args) => args.invoke().await,
            Command::Domain(args) => args.invoke(),
            Command::Home(args) => args.invoke().await,
            Command::Nameserver(args) => args.invoke(),
            Command::Record(args) => args.invoke(),
        }
    }
}
