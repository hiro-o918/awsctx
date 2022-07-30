use std::{
    io::{Seek, SeekFrom, Write},
    rc::Rc,
};

use maplit::hashmap;
use rstest::*;
use tempfile::NamedTempFile;

use awsctx::{configs::Configs, creds::Credentials, ctx};

#[fixture]
pub fn aws_credentials_text() -> String {
    r#"[baz]
aws_access_key_id=ZZZZZZZZZZZ
aws_secret_access_key=ZZZZZZZZZZZ
aws_session_token=ZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ

[bar]
aws_access_key_id=YYYYYYYYYYY
aws_secret_access_key=YYYYYYYYYYY
aws_session_token=YYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY

[foo]
aws_access_key_id=XXXXXXXXXXX
aws_secret_access_key=XXXXXXXXXXX
aws_session_token=XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX

[default]
aws_access_key_id=XXXXXXXXXXX
aws_secret_access_key=XXXXXXXXXXX
aws_session_token=XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
"#
    .to_string()
}

#[fixture]
pub fn aws_credentials_text_without_default() -> String {
    r#"[bar]
aws_access_key_id=YYYYYYYYYYY
aws_secret_access_key=YYYYYYYYYYY
aws_session_token=YYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY

[foo]
aws_access_key_id=XXXXXXXXXXX
aws_secret_access_key=XXXXXXXXXXX
aws_session_token=XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
"#
    .to_string()
}

#[fixture(text = aws_credentials_text())]
pub fn aws_credentials(text: String) -> NamedTempFile {
    let mut f = NamedTempFile::new().unwrap();
    write!(f, "{}", text).unwrap();
    f.flush().unwrap();
    f.seek(SeekFrom::Start(0)).unwrap();
    f
}

#[fixture]
pub fn credentials(aws_credentials: NamedTempFile) -> Credentials {
    Credentials::load_credentials(aws_credentials.path()).unwrap()
}

#[fixture(aws_credentials = aws_credentials(aws_credentials_text_without_default()))]
pub fn credentials_without_default(aws_credentials: NamedTempFile) -> Credentials {
    Credentials::load_credentials(aws_credentials.path()).unwrap()
}

#[fixture]
pub fn contexts() -> Vec<ctx::Context> {
    vec![
        ctx::Context {
            name: "bar".to_string(),
            active: false,
        },
        ctx::Context {
            name: "baz".to_string(),
            active: false,
        },
        ctx::Context {
            name: "foo".to_string(),
            active: true,
        },
    ]
}

#[fixture]
pub fn contexts_without_default() -> Vec<ctx::Context> {
    vec![
        ctx::Context {
            name: "bar".to_string(),
            active: false,
        },
        ctx::Context {
            name: "foo".to_string(),
            active: false,
        },
    ]
}

#[fixture]
pub fn configs() -> Rc<Configs> {
    Rc::new(Configs {
        auth_commands: hashmap! {
           "foo".to_string() => "echo auth".to_string(),
              "bar".to_string() => "exit 1".to_string(),
              Configs::DEFAULT_AUTH_COMMAND_KEY.to_string() => "echo default auth".to_string(),
        },
    })
}

#[fixture]
pub fn configs_without_default() -> Rc<Configs> {
    Rc::new(Configs {
        auth_commands: hashmap! {
           "foo".to_string() => "echo auth".to_string(),
              "bar".to_string() => "exit 1".to_string(),
        },
    })
}
