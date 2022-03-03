# awsctx
Context manager for AWS Profiles

## Demo
[![asciicast](https://asciinema.org/a/5bpFGoV2AlptWM9lWvVaIieeQ.svg)](https://asciinema.org/a/5bpFGoV2AlptWM9lWvVaIieeQ)

## Installation
**NOTE: [jq](https://github.com/stedolan/jq) required**

### macOS
:arrow_down: Download a binary and move to `/usr/local/bin`
```console
$ curl -s https://api.github.com/repos/hiro-o918/awsctx/releases/latest \
  | jq -r '.assets[] | select(.name | test("^awsctx_v[0-9]+\\.[0-9]+\\.[0-9]+_x86_64-apple-darwin\\.tar\\.gz$")) | .browser_download_url' \
  | xargs wget -O - \
  | tar zxvf - \
  && mv awsctx /usr/local/bin
```

### Linux
:arrow_down: Download a binary and move to `/usr/local/bin`
```console
$ curl -s https://api.github.com/repos/hiro-o918/awsctx/releases/latest \
  | jq -r '.assets[] | select(.name | test("^awsctx_v[0-9]+\\.[0-9]+\\.[0-9]+_x86_64-unknown-linux-musl\\.tar\\.gz$")) | .browser_download_url' \
  | xargs wget -O - \
  | tar zxvf - \
  && mv awsctx /usr/local/bin
```

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
