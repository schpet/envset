# envset

`envset` is a command-line tool for setting environment variables in a .env file. It allows you to easily add or update environment variables without manually editing the .env file.

## Installation

You can install `envset` using Cargo:

```
cargo install envset
```

## Usage

The basic syntax for using `envset` is:

```
envset [OPTIONS] <KEY1=value1> [KEY2=value2]...
```

### Options

- `-f, --force`: Overwrite existing variables
- `-o, --output <FILE>`: Specify the output file (default: .env)

### Examples

1. Set a new environment variable:
   ```
   envset API_KEY=myapikey123
   ```

2. Set multiple environment variables:
   ```
   envset DB_HOST=localhost DB_PORT=5432
   ```

3. Overwrite an existing variable:
   ```
   envset --force API_KEY=newapikey456
   ```

4. Set variables in a specific file:
   ```
   envset --output .env.production API_URL=https://api.example.com
   ```

## Features

- Set one or multiple environment variables at once
- Option to overwrite existing variables
- Specify a custom output file
- Preserves existing variables in the .env file
- Trims whitespace and quotes from values

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the ISC License.
