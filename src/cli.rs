use clap::Parser;

#[derive(Debug, Parser)]
#[clap(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
    arg_required_else_help = true,
)]
pub struct Cli {
    #[clap(subcommand)]
    pub sub_command: SubCommand,
}

#[derive(Debug, Parser)]
pub enum SubCommand {
    Init,
    Enter,
    Kill,
}
