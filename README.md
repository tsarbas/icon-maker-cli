# icon-maker

Generate Apple `AppIcon.appiconset` assets with OpenAI image models.

## What it does

- Calls the OpenAI image API to generate a 1024x1024 master icon.
- Resizes and writes all required Apple icon sizes.
- Creates `<out>/AppIcon.appiconset/Contents.json`.
- Produces 28 PNG files plus `Contents.json`.

## Requirements

- Rust + Cargo
- OpenAI API key (`OPENAI_API_KEY`) or config key
- Network access to OpenAI image API

## Installation (build.sh first)

From the repo root:

```bash
./build.sh
```

By default, this script:

- Builds `icon-maker` in release mode
- Installs binary to `~/.local/bin`
- Writes config at `$XDG_CONFIG_HOME/icon-maker/config.toml` (or `~/.config/icon-maker/config.toml`)
- Updates your shell profile PATH block when needed

Options:

```bash
./build.sh --debug
./build.sh --no-path
./build.sh --no-config
./build.sh --force-config
./build.sh --install-dir /custom/path
```

Verify:

```bash
icon-maker --help
```

## Alternative: run locally without install

```bash
cargo run -- --help
cargo run -- generate --help
```

## Configuration

Config file lookup:

- `$XDG_CONFIG_HOME/icon-maker/config.toml`
- Fallback: `~/.config/icon-maker/config.toml`

Example config:

```toml
openai_key = "sk-..."
model = "gpt-image-1.5"
```

Resolution precedence:

- API key: `OPENAI_API_KEY` -> `config.openai_key`
- Model: `--model` -> `OPENAI_MODEL` -> `config.model` -> `gpt-image-1.5`

## Usage

```bash
icon-maker generate [OPTIONS] --app-name <APP_NAME> --subject <SUBJECT> --colors <COLORS> --out <OUT>
```

Required flags:

- `--app-name`
- `--subject`
- `--colors`
- `--out`

Options:

- `--style <flat|outlined|3D|gradient|glyph>` (default: `gradient`)
- `--background <BACKGROUND>`
- `--model <MODEL>`
- `--seed <SEED>`
- `--force`
- `--dry-run`
- `--verbose`

## Examples

Dry run (prints prompt and planned output files only):

```bash
cargo run -- generate \
  --app-name "Orbit" \
  --subject "rocket" \
  --colors "blue, orange" \
  --style gradient \
  --out ./build/icons \
  --dry-run
```

Generate and overwrite existing icon set:

```bash
cargo run -- generate \
  --app-name "Orbit" \
  --subject "rocket" \
  --colors "blue, orange" \
  --style gradient \
  --out ./build/icons \
  --seed 42 \
  --force \
  --verbose
```

## Output contract

Expected under `<out>/AppIcon.appiconset/`:

- `Contents.json`
- 28 PNG files
- Includes `icon-ios-marketing-1024pt@1x.png`

`Contents.json` includes:

- `info.author = "xcode"`
- `info.version = 1`

## Troubleshooting

`OpenAI API key is missing (set OPENAI_API_KEY or config openai_key)`:

- Set `OPENAI_API_KEY`, or set `openai_key` in config TOML.

`output directory already exists: ... Use --force to overwrite.`:

- Re-run with `--force` only if replacing existing output is intended.

`OpenAI image API error 429` or `5xx`:

- Retry the command.
- The client already retries up to 3 attempts with backoff.
- Reduce request frequency.

Config parse error:

- Fix invalid TOML in the config file path and run again.

## Development

Run tests:

```bash
cargo test
```

Quick command shape and output planning check:

```bash
cargo run -- generate \
  --app-name "Test" \
  --subject "star" \
  --colors "blue, white" \
  --out ./tmp/icons \
  --dry-run
```
