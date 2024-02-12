use clap::{Parser, Subcommand};

mod db;
mod bfs;
mod iddfs;

#[derive(Parser)]
/// Miscellaneous tools for InfiniteCraft routing
struct IcToolsCommand {
    #[clap(subcommand)]
    subcommand: IcToolsSubcommand,
}

#[derive(Subcommand)]
enum IcToolsSubcommand {
    Bfs(bfs::Config),
    Iddfs(iddfs::Config),
}

fn main() {
    match IcToolsCommand::parse().subcommand {
        IcToolsSubcommand::Bfs(config) => bfs::run(config),
        IcToolsSubcommand::Iddfs(config) => iddfs::run(config),
    }
}
