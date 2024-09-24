# envset

`envset` is a CLI for setting environment variables in .env files.

this is something you may have never considered needing a specialized tool for.
and you don't.
but just like updating a git config it's more fun with a command.

## installation

currently available on homebrew and [crates.io](https://crates.io/crates/envset)

```bash
brew install schpet/tap/envset
```

```bash
cargo install envset
```

## usage

### set vars

```bash
# basic usage, FYI that it prints a diff of the changes to stdout
envset KEY1=value1 KEY2=value2

# some env vars are normally very annoying to set, like json or multiline strings.
# but envset has your back!
envset JSON="$(cat credentials.json)" PRIVATE_KEY="$(openssl genrsa -out /dev/stdout 2048)"

# pipe in stdin, useful for copying stuff from one env to another
echo -e "KEY1=value1\nKEY2=value2" | envset

# heroku users can easily copy parts of their config
heroku config -s | grep "^AWS_" | envset

# override the default path to a .env file
envset -f .env.test KEY1=value1
```

### read vars

```bash
# a few ways to print the current .env
envset
envset print --json

# grab a single value
envset get KEY1

# keys only, thanks
envset keys
```

### delete vars

```bash
envset delete KEY1 KEY2
```

## about

this cli was thrown together quickly with [aider](https://aider.chat/),
i also put up [a blog post](https://schpet.com/linklog/envset-updates-env-files) explaining why i made this.
