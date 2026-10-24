use super::info::DomainInfoArgs;
use super::list::DomainListArgs;
use crate::cli::output::CliOutput;
use arbitrary::Arbitrary;
use facet::Facet;
use figue as args;

/// Offline domain commands.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct DomainArgs {
    #[facet(args::subcommand)]
    pub command: DomainCommand,
}

/// Available domain commands.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[repr(u8)]
pub enum DomainCommand {
    /// list domain information from an offline snapshot.
    List(DomainListArgs),
    /// info domain information from an offline snapshot.
    Info(DomainInfoArgs),
}

impl DomainArgs {
    /// # Errors
    /// Returns an error for invalid offline input or planning conflicts.
    pub fn invoke(self) -> eyre::Result<CliOutput> {
        match self.command {
            DomainCommand::List(args) => args.invoke(),
            DomainCommand::Info(args) => args.invoke(),
        }
    }
}
