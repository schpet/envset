# envset

`envset` is a command-line tool for setting environment variables in a .env file. it allows you to easily add or update environment variables without manually editing the .env file.

this cli was thrown together quickly with [aider](https://aider.chat/), i also put up [a blog post](https://schpet.com/linklog/envset-updates-env-files) explaining why i made this.

## installation

avaialble via homebrew and cargo:

```bash
brew install schpet/tap/envset
```
```bash
cargo install envset
```

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

by default, `envset` will create or update a `.env` file in the current directory. if you want to use a different file, you can specify it with the `--file` option.

```bash
envset --file .env.test KEY1=value1
```

```bash
# .env.test file contents after running the command:
KEY1=value1
```

### get the value of a single environment variable

```bash
envset get KEY1
```

this command will print the value of KEY1 from the .env file. if the variable is not found, it will print an error message.

you can also specify a custom .env file location when using the `get` subcommand:

```bash
envset --file .env.test get KEY1
```

### print all environment variables

```bash
envset print
```

this command will print all environment variables from the .env file. the output will be colored, with keys in blue and values in green, when outputting to a terminal.

### print all keys

```bash
envset keys
```

this command will print all keys from the .env file, without their values.

### delete environment variables

```bash
envset delete KEY1 KEY2
```

this command will remove the specified environment variables (KEY1 and KEY2 in this example) from the .env file.

### prevent overwriting existing variables

```bash
envset --no-overwrite KEY1=newvalue1
```

this option prevents overwriting existing variables in the .env file. if a variable already exists, its value will not be changed.
