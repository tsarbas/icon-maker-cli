# Icon Maker CLI Reference

## CLI Surface

Primary command:

```bash
icon-maker generate --app-name ... --subject ... --colors ... --out ...
```

Important flags:

- `--style <flat|outlined|3D|gradient|glyph>` (default: `gradient`)
- `--background <text>`
- `--model <name>`
- `--seed <u64>`
- `--force`
- `--dry-run`
- `--verbose`

Required flags:

- `--app-name`
- `--subject`
- `--colors`
- `--out`

## Environment And Config

Config path:

- `$XDG_CONFIG_HOME/icon-maker/config.toml`
- fallback: `$HOME/.config/icon-maker/config.toml`

Config keys:

- `openai_key = "sk-..."`
- `model = "gpt-image-1.5"`

Resolution precedence:

- API key: `OPENAI_API_KEY` -> `config.openai_key`
- Model: `--model` -> `OPENAI_MODEL` -> `config.model` -> `gpt-image-1.5`

## High-Signal Command Examples

Dry run to inspect prompt and planned files:

```bash
cargo run -- generate \
  --app-name "Orbit" \
  --subject "rocket" \
  --colors "blue, orange" \
  --style gradient \
  --out ./build/icons \
  --dry-run
```

Actual generation with overwrite:

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

## Output Contract

Expected under `<out>/AppIcon.appiconset`:

- `Contents.json`
- 28 PNG files
- Includes `icon-ios-marketing-1024pt@1x.png`

`Contents.json` should include:

- `info.author = "xcode"`
- `info.version = 1`

## Troubleshooting

`OpenAI API key is missing`:

- Set `OPENAI_API_KEY` or add `openai_key` in config TOML.

`output directory already exists ... Use --force to overwrite`:

- Rerun with `--force` only if replacing existing output is intended.

`OpenAI response was not a PNG image`:

- Retry with `--verbose`.
- Try an explicitly supported image model.

`OpenAI image API error 429` or 5xx:

- Retry; the client already retries up to 3 attempts with backoff.
- Reduce request frequency and keep 1 generation at a time.

Malformed config parse error:

- Fix invalid TOML at config path, then rerun.
