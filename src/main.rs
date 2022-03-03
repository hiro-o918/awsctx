use awsctx::{aws::AWS, ctx::CTX, view::show_contexts};
use clap::{Arg, Command};

fn main() {
    let matches = Command::new("Context Manager for AWS Profiles")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Hironori Yamamoto <mr.nikoru918@gmail.com>")
        .about("Manage profiles in a credentials of AWS CLI")
        .subcommand(
            Command::new("use-context")
                .about("Updates default context by a profile name")
                .arg(
                    Arg::new("profile")
                        .short('p')
                        .takes_value(true)
                        .help("profile name")
                        .required(true),
                ),
        )
        .subcommand(Command::new("list-contexts").about("Lists profiles in AWS CLI"))
        .get_matches();

    let aws = AWS::default();
    match matches.subcommand() {
        Some(("use-context", submatches)) => {
            let profile = submatches.value_of("profile").unwrap();
            aws.use_context(profile).unwrap();
        }
        Some(("list-contexts", _)) => {
            let contexts = aws.list_contexts().unwrap();
            show_contexts(&contexts);
        }
        _ => {
            aws.use_context_interactive().unwrap();
        }
    }
}
