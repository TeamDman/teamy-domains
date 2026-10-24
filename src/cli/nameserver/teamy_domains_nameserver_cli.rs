use super::list::NameserverListArgs;
use super::set::NameserverSetArgs;
use crate::cli::output::CliOutput;
use arbitrary::Arbitrary;
use facet::Facet;
use figue as args;

/// Offline nameserver commands.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct NameserverArgs {
    #[facet(args::subcommand)]
    pub command: NameserverCommand,
}

/// Available nameserver commands.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[repr(u8)]
pub enum NameserverCommand {
    /// list nameserver information from an offline snapshot.
    List(NameserverListArgs),
    /// set nameserver information from an offline snapshot.
    Set(NameserverSetArgs),
}

impl NameserverArgs {
    /// # Errors
    /// Returns an error for invalid offline input or planning conflicts.
    pub fn invoke(self) -> eyre::Result<CliOutput> {
        match self.command {
            NameserverCommand::List(args) => args.invoke(),
            NameserverCommand::Set(args) => args.invoke(),
        }
    }
}
