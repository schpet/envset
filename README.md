# envset

`envset` is a command-line tool for setting environment variables in a .env file. It allows you to easily add or update environment variables without manually editing the .env file.

this cli was thrown together quickly with [aider](https://aider.chat/)

## Installation

### Homebrew

```bash
brew install schpet/tap/envset
```

### Cargo

```bash
cargo install envset
```

## Usage

`envset` can be used in several ways:

### Set environment variables from command-line arguments

```
envset KEY1=value1 KEY2=value2
```

### Set environment variables from stdin

```
echo -e "KEY1=value1\nKEY2=value2" | envset
```

### Use a custom .env file location

```
envset --file /path/to/.env KEY1=value1
```

By default, `envset` will create or update a `.env` file in the current directory. If you want to use a different file, you can specify it with the `--file` option.

### Additional Options

For more information on available options, run:

```
envset --help
```
