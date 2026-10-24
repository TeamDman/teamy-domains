use super::add::RecordAddArgs;
use super::delete::RecordDeleteArgs;
use super::list::RecordListArgs;
use super::update::RecordUpdateArgs;
use crate::cli::output::CliOutput;
use arbitrary::Arbitrary;
use facet::Facet;
use figue as args;

/// Offline record commands.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct RecordArgs {
    #[facet(args::subcommand)]
    pub command: RecordCommand,
}

/// Available record commands.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[repr(u8)]
pub enum RecordCommand {
    /// list record information from an offline snapshot.
    List(RecordListArgs),
    /// add record information from an offline snapshot.
    Add(RecordAddArgs),
    /// update record information from an offline snapshot.
    Update(RecordUpdateArgs),
    /// delete record information from an offline snapshot.
    Delete(RecordDeleteArgs),
}

impl RecordArgs {
    /// # Errors
    /// Returns an error for invalid offline input or planning conflicts.
    pub fn invoke(self) -> eyre::Result<CliOutput> {
        match self.command {
            RecordCommand::List(args) => args.invoke(),
            RecordCommand::Add(args) => args.invoke(),
            RecordCommand::Update(args) => args.invoke(),
            RecordCommand::Delete(args) => args.invoke(),
        }
    }
}
