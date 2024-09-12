# envset

`envset` is a CLI for setting environment variables in .env files. this is something you may have never considered needing a specialized tool for. and you don't. but just like updating a git config it's just more fun doing it with a command.

## installation

available on homebrew and cargo:

```bash
brew install schpet/tap/envset
```
```bash
cargo install envset
```

## usage

### set variables

```bash
# basic usage
envset KEY1=value1 KEY2=value2

# pipe in stdin, useful for copying stuff from one env to another
echo -e "KEY1=value1\nKEY2=value2" | envset

# .env in the cwd is default, but you can use a different path
envset --file .env.test KEY1=value1

# avoid clobbering existing values
envset --no-overwrite KEY1=newvalue1
```

### get variables

```bash
# print all key value pairs 
envset

# print a single value
envset get KEY1

# print all keys
envset keys
```

### delete variables

```bash
envset delete KEY1 KEY2
```

## about

this cli was thrown together quickly with [aider](https://aider.chat/), i also put up [a blog post](https://schpet.com/linklog/envset-updates-env-files) explaining why i made this.
