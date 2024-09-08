# envset

`envset` is a command-line tool for setting environment variables in a .env file. it allows you to easily add or update environment variables without manually editing the .env file.

this cli was thrown together quickly with [aider](https://aider.chat/)

## installation

### homebrew

```bash
brew install schpet/tap/envset
```

### cargo

```bash
cargo install envset
```

## usage

### set environment variables

```
envset KEY1=value1 KEY2=value2
```

### set environment variables from stdin

```
echo -e "KEY1=value1\nKEY2=value2" | envset
```

### use a custom .env file location

```
envset --file /path/to/.env KEY1=value1
```

by default, `envset` will create or update a `.env` file in the current directory. if you want to use a different file, you can specify it with the `--file` option.
