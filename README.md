# envset

`envset` is a command-line tool for setting environment variables in a .env file. it allows you to easily add or update environment variables without manually editing the .env file.

this cli was thrown together quickly with [aider](https://aider.chat/)

## usage

### set environment variables

```bash
envset KEY1=value1 KEY2=value2
```

```bash
# .env file contents after running the command:
KEY1=value1
KEY2=value2
```

### set environment variables from stdin

```bash
echo -e "KEY1=value1\nKEY2=value2" | envset
```

```bash
# .env file contents after running the command:
KEY1=value1
KEY2=value2
```

### use a custom .env file location

```bash
envset --file .env.test KEY1=value1
```

```bash
# .env.test file contents after running the command:
KEY1=value1
```

by default, `envset` will create or update a `.env` file in the current directory. if you want to use a different file, you can specify it with the `--file` option.

### get the value of a single environment variable

```bash
envset get KEY1
```

This command will print the value of KEY1 from the .env file. If the variable is not found, it will print an error message.

you can also specify a custom .env file location when using the `get` subcommand:

```bash
envset --file .env.test get KEY1
```

## installation

### homebrew

```bash
brew install schpet/tap/envset
```

### cargo

```bash
cargo install envset
```
