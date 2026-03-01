use crate::cli::IconStyle;

pub struct PromptInput<'a> {
    pub app_name: &'a str,
    pub style: &'a IconStyle,
    pub subject: &'a str,
    pub background: Option<&'a str>,
    pub colors: &'a str,
}

pub fn compose_prompt(input: PromptInput<'_>) -> String {
    let app_name = input.app_name.trim();
    let subject = input.subject.trim();
    let colors = input.colors.trim();
    let resolved_background = input
        .background
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|v| v.to_string())
        .unwrap_or_else(|| format!("subtle gradient based on {colors}"));

    [
        format!("App icon for iOS: {app_name}."),
        format!("Style: {}.", input.style.as_prompt_value()),
        format!("Subject: {subject}, centered."),
        "Composition: single object, clean silhouette, large shape, minimal details.".to_string(),
        format!("Background: {resolved_background}."),
        format!("Colors: {colors}, high contrast."),
        "Constraints: no text, no letters, no numbers, no watermark, no frame, no rounded-square mask/container, no black/white matte background, no drop shadow outside the icon; artwork must extend to all canvas edges and corners.".to_string(),
        "Safe area: keep all important elements within ~80% of the canvas.".to_string(),
        "Output: square 1024x1024, crisp edges, vector-like, high clarity.".to_string(),
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use crate::cli::IconStyle;

    use super::{PromptInput, compose_prompt};

    #[test]
    fn compose_prompt_renders_exact_template_order() {
        let prompt = compose_prompt(PromptInput {
            app_name: "Orbit",
            style: &IconStyle::Flat,
            subject: "rocket",
            background: Some("solid navy"),
            colors: "blue, orange",
        });

        let expected = [
            "App icon for iOS: Orbit.",
            "Style: flat.",
            "Subject: rocket, centered.",
            "Composition: single object, clean silhouette, large shape, minimal details.",
            "Background: solid navy.",
            "Colors: blue, orange, high contrast.",
            "Constraints: no text, no letters, no numbers, no watermark, no frame, no rounded-square mask/container, no black/white matte background, no drop shadow outside the icon; artwork must extend to all canvas edges and corners.",
            "Safe area: keep all important elements within ~80% of the canvas.",
            "Output: square 1024x1024, crisp edges, vector-like, high clarity.",
        ]
        .join("\n");

        assert_eq!(prompt, expected);
    }

    #[test]
    fn compose_prompt_uses_default_style_gradient() {
        let prompt = compose_prompt(PromptInput {
            app_name: "Orbit",
            style: &IconStyle::Gradient,
            subject: "rocket",
            background: Some("solid navy"),
            colors: "blue, orange",
        });
        assert!(prompt.contains("\nStyle: gradient.\n"));
    }

    #[test]
    fn compose_prompt_uses_default_background_from_colors() {
        let prompt = compose_prompt(PromptInput {
            app_name: "Orbit",
            style: &IconStyle::Gradient,
            subject: "rocket",
            background: None,
            colors: "blue, orange",
        });
        assert!(prompt.contains("Background: subtle gradient based on blue, orange."));
    }

    #[test]
    fn compose_prompt_uses_explicit_background_when_given() {
        let prompt = compose_prompt(PromptInput {
            app_name: "Orbit",
            style: &IconStyle::Gradient,
            subject: "rocket",
            background: Some("solid black"),
            colors: "blue, orange",
        });
        assert!(prompt.contains("Background: solid black."));
    }
}
