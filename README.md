# ctxm
Context manager for CLI tools

## Support
- [awscli](https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-welcome.html)

## Installation
```console
$ git clone https://github.com/hiro-o918/ctxm
$ cd ctxm
$ cargo build --release
$ cp ./target/release/ctxm /usr/local/bin
```

## How to Use
### Login AWS CLI
Login an AWS account by some ways with specified profile name.
Then, you get `~/.aws/credentials` like
```
[foo]
aws_access_key_id = XXXXXXXXXXX
aws_secret_access_key = XXXXXXXXXXX
aws_session_token = XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX

[bar]
aws_access_key_id = YYYYYYYYYYY
aws_secret_access_key = YYYYYYYYYYY
aws_session_token = YYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY
```

### List Profiles
`list-contexts` shows all the named profiles as the below.
```console
$ ctxm list-contexts
  foo
  bar
```

### Use Context
`use-context` update `~/.aws/credentials` to contain `default` profile that the values are same as specified in the option.

```console
$ ctxm use-context -p foo
```
e.g. the above commands updates credentials as the below.
```
[foo]
aws_access_key_id = XXXXXXXXXXX
aws_secret_access_key = XXXXXXXXXXX
aws_session_token = XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX

[bar]
aws_access_key_id = YYYYYYYYYYY
aws_secret_access_key = YYYYYYYYYYY
aws_session_token = YYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY

[default]
aws_access_key_id = XXXXXXXXXXX
aws_secret_access_key = XXXXXXXXXXX
aws_session_token = XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
```
