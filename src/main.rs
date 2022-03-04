use std::io;

use awsctx::{aws::AWS, ctx::CTX, view::show_contexts};

use clap::{IntoApp, Parser, Subcommand};
use clap_complete::{generate, Generator, Shell};

#[derive(Parser)]
#[clap(
    name = "awsctx",
    about = "Context Manager for AWS Profiles",
    long_about = "Manage profiles in a credentials of AWS CLI",
    version = env!("CARGO_PKG_VERSION"),
)]
struct Cli {
    #[clap(subcommand)]
    opts: Option<Opts>,
}

#[derive(Subcommand, Debug)]
enum Opts {
    /// Updates a default profile by a profile name
    #[clap(arg_required_else_help = true)]
    UseContext {
        #[clap(long, short, help = "profile name")]
        profile: String,
    },
    /// Update a default profile by interactive finder
    #[clap(skip = true)]
    UseContextByInteractiveFinder {},
    /// List all the contexts in the credentials
    #[clap(arg_required_else_help = false)]
    ListContexts {},
    /// Generate completion script
    Completion {
        #[clap(long, short, arg_enum)]
        shell: Shell,
    },
}

fn main() {
    let cli = Cli::parse();
    let aws = AWS::default();
    let opts = cli.opts.unwrap_or(Opts::UseContextByInteractiveFinder {});
    match opts {
        Opts::UseContext { profile } => {
            aws.use_context(profile.as_str()).unwrap();
        }
        Opts::UseContextByInteractiveFinder {} => {
            aws.use_context_interactive().unwrap();
        }
        Opts::ListContexts {} => {
            let contexts = aws.list_contexts().unwrap();
            show_contexts(&contexts)
        }
        Opts::Completion { shell } => {
            print_completions(shell);
        }
    }
}

fn print_completions<G: Generator>(gen: G) {
    let cmd = &mut Cli::command();
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}
