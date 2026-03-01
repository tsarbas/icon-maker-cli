use serde::Serialize;
use serde_json::json;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IconSpec {
    pub idiom: String,
    pub size: String,
    pub scale: String,
    pub pixels: u32,
    pub filename: String,
}

#[derive(Debug, Serialize)]
struct ContentImageEntry<'a> {
    idiom: &'a str,
    size: &'a str,
    scale: &'a str,
    filename: &'a str,
}

pub fn icon_specs() -> Vec<IconSpec> {
    let mut specs = Vec::new();

    let iphone_points = [20.0, 29.0, 40.0, 60.0];
    for point in iphone_points {
        for scale in [2_u32, 3_u32] {
            specs.push(make_spec("iphone", point, scale));
        }
    }

    let ipad_points = [20.0, 29.0, 40.0, 76.0];
    for point in ipad_points {
        for scale in [1_u32, 2_u32] {
            specs.push(make_spec("ipad", point, scale));
        }
    }
    specs.push(make_spec("ipad", 83.5, 2));

    let mac_points = [16.0, 32.0, 128.0, 256.0, 512.0];
    for point in mac_points {
        for scale in [1_u32, 2_u32] {
            specs.push(make_spec("mac", point, scale));
        }
    }

    specs.push(IconSpec {
        idiom: "ios-marketing".to_string(),
        size: "1024x1024".to_string(),
        scale: "1x".to_string(),
        pixels: 1024,
        filename: "icon-ios-marketing-1024pt@1x.png".to_string(),
    });

    specs
}

pub fn build_contents_json(specs: &[IconSpec]) -> serde_json::Value {
    let images: Vec<_> = specs
        .iter()
        .map(|spec| ContentImageEntry {
            idiom: &spec.idiom,
            size: &spec.size,
            scale: &spec.scale,
            filename: &spec.filename,
        })
        .collect();

    json!({
      "images": images,
      "info": {
        "author": "xcode",
        "version": 1
      }
    })
}

fn make_spec(idiom: &str, points: f32, scale: u32) -> IconSpec {
    let points_label = format_points(points);
    let pixels = (points * scale as f32).round() as u32;
    IconSpec {
        idiom: idiom.to_string(),
        size: format!("{points_label}x{points_label}"),
        scale: format!("{scale}x"),
        pixels,
        filename: format!("icon-{idiom}-{points_label}pt@{scale}x.png"),
    }
}

fn format_points(points: f32) -> String {
    if points.fract() == 0.0 {
        format!("{}", points as u32)
    } else {
        let mut raw = format!("{points:.1}");
        while raw.ends_with('0') {
            raw.pop();
        }
        if raw.ends_with('.') {
            raw.pop();
        }
        raw
    }
}

#[cfg(test)]
mod tests {
    use super::{build_contents_json, icon_specs};

    #[test]
    fn has_expected_icon_count_and_edge_sizes() {
        let specs = icon_specs();
        assert_eq!(specs.len(), 28);
        assert!(specs.iter().any(|s| s.filename == "icon-ipad-83.5pt@2x.png" && s.pixels == 167));
        assert!(
            specs
                .iter()
                .any(|s| s.idiom == "ios-marketing" && s.pixels == 1024)
        );
    }

    #[test]
    fn contents_json_has_info_and_images() {
        let specs = icon_specs();
        let contents = build_contents_json(&specs);
        assert_eq!(contents["info"]["author"], "xcode");
        assert_eq!(contents["info"]["version"], 1);
        assert_eq!(
            contents["images"].as_array().expect("images array").len(),
            specs.len()
        );
    }
}
