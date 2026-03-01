use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "icon-maker", version, about = "Generate Apple app icon sets with GPT images")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Generate(GenerateArgs),
}

#[derive(Debug, Args)]
pub struct GenerateArgs {
    #[arg(long)]
    pub app_name: String,
    #[arg(long)]
    pub subject: String,
    #[arg(long)]
    pub colors: String,
    #[arg(long, value_enum, default_value_t = IconStyle::Gradient)]
    pub style: IconStyle,
    #[arg(long)]
    pub background: Option<String>,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long, help = "Image model (overrides OPENAI_MODEL and config model)")]
    pub model: Option<String>,
    #[arg(long)]
    pub seed: Option<u64>,
    #[arg(long, default_value_t = false)]
    pub force: bool,
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
    #[arg(long, default_value_t = false)]
    pub verbose: bool,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum IconStyle {
    Flat,
    Outlined,
    #[value(name = "3D")]
    ThreeD,
    Gradient,
    Glyph,
}

impl IconStyle {
    pub fn as_prompt_value(&self) -> &'static str {
        match self {
            Self::Flat => "flat",
            Self::Outlined => "outlined",
            Self::ThreeD => "3D",
            Self::Gradient => "gradient",
            Self::Glyph => "glyph",
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{Cli, Commands};

    #[test]
    fn missing_required_app_name_fails_parse() {
        let args = [
            "icon-maker",
            "generate",
            "--subject",
            "rocket",
            "--colors",
            "blue, orange",
            "--out",
            "/tmp/out",
        ];
        let err = Cli::try_parse_from(args).expect_err("expected parse error");
        let text = err.to_string();
        assert!(text.contains("--app-name"));
    }

    #[test]
    fn missing_required_subject_fails_parse() {
        let args = [
            "icon-maker",
            "generate",
            "--app-name",
            "Orbit",
            "--colors",
            "blue, orange",
            "--out",
            "/tmp/out",
        ];
        let err = Cli::try_parse_from(args).expect_err("expected parse error");
        let text = err.to_string();
        assert!(text.contains("--subject"));
    }

    #[test]
    fn missing_required_colors_fails_parse() {
        let args = [
            "icon-maker",
            "generate",
            "--app-name",
            "Orbit",
            "--subject",
            "rocket",
            "--out",
            "/tmp/out",
        ];
        let err = Cli::try_parse_from(args).expect_err("expected parse error");
        let text = err.to_string();
        assert!(text.contains("--colors"));
    }

    #[test]
    fn invalid_style_fails_parse() {
        let args = [
            "icon-maker",
            "generate",
            "--app-name",
            "Orbit",
            "--subject",
            "rocket",
            "--colors",
            "blue, orange",
            "--style",
            "neon",
            "--out",
            "/tmp/out",
        ];
        let err = Cli::try_parse_from(args).expect_err("expected parse error");
        let text = err.to_string();
        assert!(text.contains("possible values"));
    }

    #[test]
    fn default_style_is_gradient() {
        let args = [
            "icon-maker",
            "generate",
            "--app-name",
            "Orbit",
            "--subject",
            "rocket",
            "--colors",
            "blue, orange",
            "--out",
            "/tmp/out",
        ];
        let cli = Cli::try_parse_from(args).expect("parse command");
        let Commands::Generate(generate) = cli.command;
        assert_eq!(generate.style.as_prompt_value(), "gradient");
    }
}
