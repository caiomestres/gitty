use clap::Parser;

#[derive(Parser)]
#[command(name = "gitty")]
#[command(about = "Workspace synchronization and orchestration for Git repositories")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Show version information
    Version,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Version) => {
            println!("gitty {}", env!("CARGO_PKG_VERSION"));
        }
        None => {
            println!("gitty {} — run `gitty --help` for usage", env!("CARGO_PKG_VERSION"));
        }
    }
}
