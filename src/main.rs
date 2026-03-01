mod cli;
mod config;
mod error;
mod iconset;
mod image_ops;
mod openai;
mod prompt;

use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use cli::{Cli, Commands, GenerateArgs};
use error::IconMakerError;
use prompt::PromptInput;

const DEFAULT_MODEL: &str = "gpt-image-1.5";

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("error: {err:#}");
        std::process::exit(1);
    }
}

async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Generate(args) => generate(args).await?,
    }
    Ok(())
}

async fn generate(args: GenerateArgs) -> anyhow::Result<()> {
    let appiconset_dir = args.out.join("AppIcon.appiconset");
    let specs = iconset::icon_specs();
    let config = config::load().context("failed to load config")?;
    let resolved_model = resolve_model(
        args.model.as_deref(),
        env::var("OPENAI_MODEL").ok().as_deref(),
        config.as_ref(),
    );

    let composed_prompt = prompt::compose_prompt(PromptInput {
        app_name: &args.app_name,
        style: &args.style,
        subject: &args.subject,
        background: args.background.as_deref(),
        colors: &args.colors,
    });

    if args.dry_run {
        print_plan(
            &appiconset_dir,
            &specs,
            &resolved_model,
            args.seed,
            &composed_prompt,
        );
        return Ok(());
    }

    prepare_output_dir(&appiconset_dir, args.force)?;

    let api_key = resolve_api_key(env::var("OPENAI_API_KEY").ok().as_deref(), config.as_ref())
        .map_err(anyhow::Error::from)
        .context("missing required OpenAI API key")?;

    if args.verbose {
        println!(
            "Generating 1024x1024 master icon with model {}...",
            resolved_model
        );
    }

    let client = openai::OpenAiClient::new(api_key);
    let master_png = client
        .generate_master_icon(&openai::GenerationRequest {
            model: resolved_model.clone(),
            prompt: composed_prompt.clone(),
            seed: args.seed,
            size: "1024x1024".to_string(),
        })
        .await
        .context("failed to generate master icon")?;

    let master_image = image_ops::decode_png(&master_png).context("failed to decode returned image")?;
    let flattened = image_ops::ensure_opaque_square(master_image);
    image_ops::write_icon_set(&flattened, &specs, &appiconset_dir)
        .context("failed to write icon PNG files")?;

    let contents_json = iconset::build_contents_json(&specs);
    let contents_path = appiconset_dir.join("Contents.json");
    let payload = serde_json::to_vec_pretty(&contents_json)?;
    fs::write(&contents_path, payload)
        .with_context(|| format!("failed to write {}", contents_path.display()))?;

    println!("Generated icon set in {}", appiconset_dir.display());
    println!(
        "Model: {}, seed: {}",
        resolved_model,
        args.seed
            .map(|v| v.to_string())
            .unwrap_or_else(|| "none".to_string())
    );
    println!("Wrote {} icon files + Contents.json", specs.len());
    Ok(())
}

fn first_non_empty<'a>(values: impl IntoIterator<Item = Option<&'a str>>) -> Option<String> {
    values
        .into_iter()
        .flatten()
        .map(str::trim)
        .find(|v| !v.is_empty())
        .map(ToOwned::to_owned)
}

fn resolve_model(
    cli_model: Option<&str>,
    env_model: Option<&str>,
    config: Option<&config::AppConfig>,
) -> String {
    first_non_empty([
        cli_model,
        env_model,
        config.and_then(|cfg| cfg.model.as_deref()),
    ])
    .unwrap_or_else(|| DEFAULT_MODEL.to_string())
}

fn resolve_api_key(
    env_api_key: Option<&str>,
    config: Option<&config::AppConfig>,
) -> Result<String, IconMakerError> {
    first_non_empty([
        env_api_key,
        config.and_then(|cfg| cfg.openai_key.as_deref()),
    ])
    .ok_or(IconMakerError::MissingApiKey)
}

fn prepare_output_dir(appiconset_dir: &PathBuf, force: bool) -> Result<(), IconMakerError> {
    if appiconset_dir.exists() {
        if !force {
            return Err(IconMakerError::OutputExists(appiconset_dir.clone()));
        }
        fs::remove_dir_all(appiconset_dir)?;
    }
    fs::create_dir_all(appiconset_dir)?;
    Ok(())
}

fn print_plan(
    appiconset_dir: &PathBuf,
    specs: &[iconset::IconSpec],
    model: &str,
    seed: Option<u64>,
    composed_prompt: &str,
) {
    println!("Dry run");
    println!("Output directory: {}", appiconset_dir.display());
    println!("Model: {model}");
    println!(
        "Seed: {}",
        seed.map(|v| v.to_string())
            .unwrap_or_else(|| "none".to_string())
    );
    println!("Prompt:\n{composed_prompt}");
    println!("Files to write:");
    println!("  {}", appiconset_dir.join("Contents.json").display());
    for spec in specs {
        println!("  {}", appiconset_dir.join(&spec.filename).display());
    }
}

#[cfg(test)]
mod tests {
    use super::{prepare_output_dir, resolve_api_key, resolve_model};
    use crate::config::AppConfig;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn prepare_output_dir_fails_without_force() {
        let temp_root = std::env::temp_dir().join("icon_maker_test_prepare");
        let dir = temp_root.join("AppIcon.appiconset");
        let _ = fs::remove_dir_all(&temp_root);
        fs::create_dir_all(&dir).expect("create test dir");

        let err = prepare_output_dir(&PathBuf::from(&dir), false).expect_err("expected error");
        assert!(err.to_string().contains("already exists"));

        fs::remove_dir_all(&temp_root).expect("cleanup");
    }

    #[test]
    fn resolve_model_prefers_cli_then_env_then_config_then_default() {
        let cfg = AppConfig {
            openai_key: None,
            model: Some("cfg-model".to_string()),
        };

        assert_eq!(
            resolve_model(Some("cli-model"), Some("env-model"), Some(&cfg)),
            "cli-model"
        );
        assert_eq!(resolve_model(None, Some("env-model"), Some(&cfg)), "env-model");
        assert_eq!(resolve_model(None, None, Some(&cfg)), "cfg-model");
        assert_eq!(resolve_model(None, None, None), "gpt-image-1.5");
    }

    #[test]
    fn resolve_api_key_prefers_env_then_config() {
        let cfg = AppConfig {
            openai_key: Some("cfg-key".to_string()),
            model: None,
        };

        assert_eq!(
            resolve_api_key(Some("env-key"), Some(&cfg)).expect("env key"),
            "env-key"
        );
        assert_eq!(
            resolve_api_key(None, Some(&cfg)).expect("config key"),
            "cfg-key"
        );
        assert!(resolve_api_key(None, None).is_err());
    }
}
