---
name: icon-maker-cli
description: Generate and troubleshoot Apple `AppIcon.appiconset` outputs with the `icon-maker` Rust CLI. Use when Codex needs to run or automate `icon-maker generate`, choose prompt/style/seed/model options, validate generated icon files and `Contents.json`, handle overwrite behavior (`--force`), or debug config/auth errors related to `OPENAI_API_KEY`, `OPENAI_MODEL`, and `~/.config/icon-maker/config.toml`.
---

# Icon Maker CLI

Use this skill to operate the local `icon-maker` binary end-to-end: build, dry-run, generate icons, validate output shape/count, and diagnose common runtime errors.

## Workflow

1. Confirm CLI shape and available flags with `cargo run -- --help` and `cargo run -- generate --help` when needed.
2. Resolve credentials and model source before generation:
- API key precedence: `OPENAI_API_KEY` then `config.openai_key`.
- Model precedence: `--model` then `OPENAI_MODEL` then `config.model` then `gpt-image-1.5`.
3. Run a dry run first to verify prompt text and target files:
- `cargo run -- generate ... --dry-run`
4. Run generation only after dry-run output looks correct:
- Add `--force` only when replacing an existing `AppIcon.appiconset`.
5. Validate outputs:
- Confirm `AppIcon.appiconset/Contents.json` exists.
- Confirm 28 PNG files exist and are non-empty.
- Spot-check `icon-ios-marketing-1024pt@1x.png` resolution is 1024x1024.
6. If generation fails, map the failure to targeted fixes using [icon-maker-cli-reference.md](references/icon-maker-cli-reference.md).

## Command Pattern

Use this baseline command and fill placeholders:

```bash
cargo run -- generate \
  --app-name "<AppName>" \
  --subject "<subject>" \
  --colors "<primary, secondary>" \
  --style gradient \
  --background "<optional background>" \
  --out "<output-dir>" \
  --seed <optional-u64> \
  --model <optional-model> \
  --dry-run
```

Drop `--dry-run` to perform the actual API call. Add `--force` only if output exists and replacement is intended.

## Output Validation Checklist

- Ensure output path ends with `AppIcon.appiconset`.
- Ensure exactly 28 PNG files are present.
- Ensure `Contents.json` includes `info.author = "xcode"` and `info.version = 1`.
- Ensure no transparent padding artifacts in resized icons when visual quality checks matter.

## Constraints

- Prefer deterministic CLI execution over speculative edits.
- Prefer `--dry-run` before any real generation call.
- Keep output directory explicit and avoid writing outside user-requested paths.
