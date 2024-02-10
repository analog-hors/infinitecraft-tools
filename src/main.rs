use clap::{Parser, Subcommand};

mod db;
mod bfs;

#[derive(Parser)]
/// Miscellaneous tools for InfiniteCraft routing
struct IcToolsCommand {
    #[clap(subcommand)]
    subcommand: IcToolsSubcommand,
}

#[derive(Subcommand)]
enum IcToolsSubcommand {
    Bfs(bfs::Config),
}

fn main() {
    match IcToolsCommand::parse().subcommand {
        IcToolsSubcommand::Bfs(config) => bfs::run(config),
    }
}
