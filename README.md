# awsctx
Context manager for AWS Profiles

## Installation
<!-- TODO -->

## Demo
[![asciicast](https://asciinema.org/a/5bpFGoV2AlptWM9lWvVaIieeQ.svg)](https://asciinema.org/a/5bpFGoV2AlptWM9lWvVaIieeQ)

## How it Works
### Login
Authorize your shell by some ways with specified profile name.
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
