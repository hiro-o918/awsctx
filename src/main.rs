use std::{io, rc::Rc};

use awsctx::{
    aws::AWS,
    configs::Configs,
    ctx::{CTXError, CTX},
    view::{fatal_ctxerr, show_context, show_contexts},
};

use clap::{IntoApp, Parser, Subcommand};
use clap_complete::{generate, Generator, Shell};
use simplelog as sl;

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
    /// Enable verbose output
    #[clap(long, short = 'v', parse(from_occurrences), global = true)]
    verbose: i8,
}

#[derive(Subcommand, Debug)]
enum Opts {
    /// Show active context in the credentials.
    #[clap(arg_required_else_help = false)]
    ActiveContext {},
    /// Auth awscli with the specified profile by pre-defined scripts, then make it active.
    ///
    /// This function requires the configuration set up for the specified profile before use.
    #[clap(arg_required_else_help = true)]
    Auth {
        #[clap(long, short, help = "profile name")]
        profile: String,
    },
    /// List all the contexts in the credentials.
    #[clap(arg_required_else_help = false)]
    ListContexts {},
    /// Auth awscli for the active profile by pre-defined scripts
    ///
    /// This function requires the configuration set up for the specified profile before use.
    #[clap(arg_required_else_help = false)]
    Refresh {},
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

fn level_enum(verbosity: i8) -> log::Level {
    match verbosity {
        std::i8::MIN..=-1 => log::Level::Info,
        0 => log::Level::Info,
        1 => log::Level::Debug,
        2 => log::Level::Trace,
        3..=std::i8::MAX => log::Level::Trace,
    }
}

fn main() {
    let cli = Cli::parse();
    sl::TermLogger::init(
        level_enum(cli.verbose).to_level_filter(),
        sl::ConfigBuilder::new()
            .set_time_level(log::LevelFilter::Off)
            .set_target_level(log::LevelFilter::Debug)
            .set_max_level(log::LevelFilter::Debug)
            .set_write_log_enable_colors(true)
            .build(),
        sl::TerminalMode::Mixed,
        sl::ColorChoice::Auto,
    )
    .unwrap();

    let configs = Rc::new(fatal_ctxerr(Configs::initialize_default_configs()));
    let aws = AWS::new(Rc::clone(&configs)).unwrap();
    let opts = cli.opts.unwrap_or(Opts::UseContextByInteractiveFinder {});
    match opts {
        Opts::ActiveContext {} => {
            let context = fatal_ctxerr(aws.get_active_context());
            show_context(&context)
        }
        Opts::Auth { profile } => {
            let context = fatal_ctxerr(aws.auth(profile.as_str()));
            sl::info!(
                "<green>successfully auth with profile ({}) and make it active</>",
                context.name
            );
        }
        Opts::ListContexts {} => {
            let contexts = fatal_ctxerr(aws.list_contexts());
            show_contexts(&contexts)
        }
        Opts::UseContext { profile } => {
            let context = fatal_ctxerr(aws.use_context(profile.as_str()));
            sl::info!("<green>switch to profile ({})</>", context.name);
        }
        Opts::UseContextByInteractiveFinder {} => {
            let context = match aws.use_context_interactive() {
                Ok(context) => Some(context),
                Err(err) => match err {
                    CTXError::NoContextIsSelected { source: _ } => None,
                    _ => fatal_ctxerr(Err(err)),
                },
            };
            if let Some(context) = context {
                sl::info!("<green>switch to profile ({})</>", context.name);
            }
        }
        Opts::Refresh {} => {
            let active_context = fatal_ctxerr(aws.get_active_context());
            fatal_ctxerr(aws.auth(active_context.name.as_str()));
            sl::info!(
                "<green>successfully refresh credentials for profile ({})</>",
                active_context.name
            );
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
