use std::{io, rc::Rc};

use awsctx::{
    aws::AWS,
    configs::Configs,
    ctx::{CTXError, CTX},
    view::{show_active_context, show_contexts},
};

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
    /// Show active context in the credentials.
    #[clap(arg_required_else_help = false)]
    ActiveContext {},
    /// Auth awscli with the specified profile.
    /// This function requires the configuration set up for the specified profile before use.
    #[clap(arg_required_else_help = true)]
    Auth {
        #[clap(long, short, help = "profile name")]
        profile: String,
    },
    /// List all the contexts in the credentials.
    #[clap(arg_required_else_help = false)]
    ListContexts {},
    /// Updates a default profile by a profile name.
    #[clap(arg_required_else_help = true)]
    UseContext {
        #[clap(long, short, help = "profile name")]
        profile: String,
    },
    /// Update a default profile by interactive finder.
    #[clap(skip = true)]
    UseContextByInteractiveFinder {},
    /// Generate completion script.
    Completion {
        #[clap(long, short, arg_enum)]
        shell: Shell,
    },
}

fn main() {
    let cli = Cli::parse();
    let configs = Rc::new(Configs::initialize_default_configs().unwrap());
    let aws = AWS::new(Rc::clone(&configs)).unwrap();
    let opts = cli.opts.unwrap_or(Opts::UseContextByInteractiveFinder {});
    match opts {
        Opts::ActiveContext {} => {
            let contexts = aws.list_contexts().unwrap();
            show_active_context(&contexts)
        }
        Opts::Auth { profile } => {
            let context = aws.auth(profile.as_str()).unwrap();
            println!("{:?}", context)
        }
        Opts::ListContexts {} => {
            let contexts = aws.list_contexts().unwrap();
            show_contexts(&contexts)
        }
        Opts::UseContext { profile } => {
            aws.use_context(profile.as_str()).unwrap();
        }
        Opts::UseContextByInteractiveFinder {} => match aws.use_context_interactive() {
            Ok(_) => Ok(()),
            Err(err) => match err {
                CTXError::NoContextIsSelected {} => Ok(()),
                _ => Err(err),
            },
        }
        .unwrap(),

        Opts::Completion { shell } => {
            print_completions(shell);
        }
    }
}

fn print_completions<G: Generator>(gen: G) {
    let cmd = &mut Cli::command();
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}
