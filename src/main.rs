use crate::aws::AWS;
use crate::ctxm::view::show_contexts;
use crate::ctxm::CTXM;
use clap::{App, AppSettings, Arg};

mod aws;
mod ctxm;

fn main() {
    let matches = App::new("Context Manager for CLIs")
        .version("0.0.1")
        .author("Hironori Yamamoto <mr.nikoru918@gmail.com>")
        .about("Mange Contexts for CLIs")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            App::new("use-context")
                .about("Updates default context by a profile name")
                .arg(
                    Arg::with_name("profile")
                        .short("p")
                        .takes_value(true)
                        .help("profile name")
                        .required(true),
                ),
        )
        .subcommand(App::new("list-contexts").about("Lists profiles"))
        .get_matches();

    let aws = AWS::default();
    match matches.subcommand() {
        ("use-context", Some(submatches)) => {
            let profile = submatches.value_of("profile").unwrap();
            aws.use_context(profile).unwrap();
        }
        ("list-contexts", Some(_)) => {
            let contexts = aws.list_contexts().unwrap();
            show_contexts(&contexts);
        }
        _ => (),
    }
}
