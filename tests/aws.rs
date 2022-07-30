use std::rc::Rc;

use awsctx::{aws::AWS, configs::Configs, ctx};
use rstest::*;
use tempfile::NamedTempFile;

mod common;
use common::*;

#[rstest(configs, input, expect)]
#[case(
    configs(),
    "foo",
    Ok(ctx::Context {name: "foo".to_string(), active: true}),
)]
#[case(
    configs(),
    "bar",
    Err(ctx::CTXError::InvalidConfigurations {
        message: "failed to execute an auth script of profile (bar), check configurations".to_string(),
        source: None
    }),
)]
//  baz is not defined in configs.auth_commands and default is set
#[case(
    configs(),
    "baz",
    Ok(ctx::Context {name: "baz".to_string(), active: true}),
)]
//  baz is not defined in configs.auth_commands and default is not set
#[case(
    configs_without_default(),
    "baz",
    Err(ctx::CTXError::NoAuthConfiguration{ profile: "baz".to_string(), source: None }),
)]
fn test_aws_auth(
    configs: Rc<Configs>,
    aws_credentials: NamedTempFile,
    input: &str,
    expect: Result<ctx::Context, ctx::CTXError>,
) {
    let aws: &dyn ctx::CTX = &AWS::new(configs, aws_credentials.path()).unwrap();
    let actual = aws.auth(input);
    match (&expect, &actual) {
        (Ok(expect), Ok(actual)) => {
            assert_eq!(expect, actual);
            assert_eq!(expect, &aws.get_active_context().unwrap())
        }
        (Err(expect), Err(actual)) => match (&expect, &actual) {
            (
                ctx::CTXError::InvalidConfigurations {
                    message: expect_message,
                    source: _expect_source,
                },
                ctx::CTXError::InvalidConfigurations {
                    message: actual_message,
                    source: _actual_source,
                },
            ) => {
                assert_eq!(expect_message, actual_message);
            }
            (
                ctx::CTXError::NoAuthConfiguration {
                    profile: expect_profile,
                    source: _expect_source,
                },
                ctx::CTXError::NoAuthConfiguration {
                    profile: actual_profile,
                    source: _actual_source,
                },
            ) => {
                assert_eq!(expect_profile, actual_profile);
            }
            _ => panic!("unexpected error: {}", actual),
        },
        _ => panic!(
            "expect and actual are not match: expect: {:?}, actual: {:?}",
            &expect, &actual
        ),
    }
}

#[rstest(aws_credentials, expect)]
#[case(aws_credentials(aws_credentials_text()), contexts())]
#[case(
    aws_credentials(aws_credentials_text_without_default()),
    contexts_without_default()
)]
fn test_aws_list_contexts(
    configs: Rc<Configs>,
    aws_credentials: NamedTempFile,
    expect: Vec<ctx::Context>,
) {
    let aws: &dyn ctx::CTX = &AWS::new(configs, aws_credentials.path()).unwrap();
    let actual = aws.list_contexts().unwrap();
    assert_eq!(expect, actual);
}

#[rstest(aws_credentials, expect)]
#[case(
    aws_credentials(aws_credentials_text()),
    Ok(ctx::Context {name: "foo".to_string(),active: true,}),
)]
#[case(
    aws_credentials(aws_credentials_text_without_default()),
    Err(ctx::CTXError::NoActiveContext { source: None }),
)]
fn test_aws_get_active_context(
    configs: Rc<Configs>,
    aws_credentials: NamedTempFile,
    expect: Result<ctx::Context, ctx::CTXError>,
) {
    let aws: &dyn ctx::CTX = &AWS::new(configs, aws_credentials.path()).unwrap();
    let actual = aws.get_active_context();
    match (expect, actual) {
        (Ok(expect), Ok(actual)) => {
            assert_eq!(expect, actual);
        }
        (Err(expect), Err(actual)) => match (expect, actual) {
            (
                ctx::CTXError::NoActiveContext { source: _ },
                ctx::CTXError::NoActiveContext { source: _ },
            ) => (),
            _ => panic!("unexpected error"),
        },
        _ => panic!("expect and actual are not match"),
    }
}

#[rstest(input, expect)]
#[case(
    "bar",
    Ok(ctx::Context {name: "bar".to_string(), active: true}),
)]
#[case(
    "unknown",
    Err(ctx::CTXError::NoSuchProfile{ profile: "unknown".to_string(), source: None }),
)]
fn test_aws_use_context(
    configs: Rc<Configs>,
    aws_credentials: NamedTempFile,
    input: &str,
    expect: Result<ctx::Context, ctx::CTXError>,
) {
    let aws: &dyn ctx::CTX = &AWS::new(configs, aws_credentials.path()).unwrap();
    let actual = aws.use_context(input);
    match (expect, actual) {
        (Ok(expect), Ok(actual)) => {
            assert_eq!(expect, actual);
            assert_eq!(expect, aws.get_active_context().unwrap())
        }
        (Err(expect), Err(actual)) => match (&expect, &actual) {
            (
                ctx::CTXError::NoSuchProfile {
                    profile: expect_profile,
                    source: _expect_source,
                },
                ctx::CTXError::NoSuchProfile {
                    profile: actual_profile,
                    source: _actual_source,
                },
            ) => {
                assert_eq!(expect_profile, actual_profile);
            }
            _ => panic!("unexpected error: {}", actual),
        },
        _ => panic!("expect and actual are not match"),
    }
}
