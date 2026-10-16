use std::path::Path;
use gpui::{div, px, rgb, Div, ParentElement, Styled};
use crate::theme::ThemeColors;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageMetadata {
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub file_size: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum MediaError {
    #[error("File not found: {0}")]
    NotFound(String),
    #[error("Image decode error: {0}")]
    Decode(String),
    #[error("IO error: {0}")]
    Io(String),
}

/// Compute aspect-ratio preserved dimensions fitting inside max bounds
pub fn compute_thumbnail_dimensions(
    orig_w: u32,
    orig_h: u32,
    max_w: u32,
    max_h: u32,
) -> (u32, u32) {
    if orig_w == 0 || orig_h == 0 || max_w == 0 || max_h == 0 {
        return (0, 0);
    }

    let w_ratio = max_w as f64 / orig_w as f64;
    let h_ratio = max_h as f64 / orig_h as f64;
    let scale = w_ratio.min(h_ratio).min(1.0);

    let scaled_w = (orig_w as f64 * scale).round() as u32;
    let scaled_h = (orig_h as f64 * scale).round() as u32;

    (scaled_w.max(1), scaled_h.max(1))
}

pub fn read_image_metadata(path: &Path) -> Result<ImageMetadata, MediaError> {
    if !path.exists() {
        return Err(MediaError::NotFound(path.display().to_string()));
    }

    let metadata = std::fs::metadata(path)
        .map_err(|e| MediaError::Io(e.to_string()))?;

    let reader = image::ImageReader::open(path)
        .map_err(|e| MediaError::Io(e.to_string()))?
        .with_guessed_format()
        .map_err(|e| MediaError::Decode(e.to_string()))?;

    let format_str = reader
        .format()
        .map(|f| format!("{:?}", f))
        .unwrap_or_else(|| "Unknown".to_string());

    let (width, height) = reader
        .into_dimensions()
        .map_err(|e| MediaError::Decode(e.to_string()))?;

    Ok(ImageMetadata {
        width,
        height,
        format: format_str,
        file_size: metadata.len(),
    })
}

pub fn render_image_preview(
    path: Option<&str>,
    caption: Option<&str>,
    dimensions: Option<(u32, u32)>,
) -> Div {
    let (w, h) = dimensions.unwrap_or((280, 180));
    let (scaled_w, scaled_h) = compute_thumbnail_dimensions(w, h, 320, 240);

    let display_caption = caption.unwrap_or_else(|| {
        path.and_then(|p| Path::new(p).file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("Photo")
    });

    div()
        .flex()
        .flex_col()
        .w(px(scaled_w as f32))
        .max_w(px(320.0))
        .rounded_md()
        .overflow_hidden()
        .bg(rgb(ThemeColors::SURFACE))
        .border_1()
        .border_color(rgb(ThemeColors::BORDER))
        // Thumbnail Image Surface Placeholder
        .child(
            div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .w_full()
                .h(px(scaled_h as f32))
                .bg(rgb(ThemeColors::CRUST))
                .child(
                    div()
                        .text_size(px(32.0))
                        .child("🖼️"),
                )
                .child(
                    div()
                        .pt_1()
                        .text_size(px(11.0))
                        .text_color(ThemeColors::text_muted())
                        .child(format!("{} × {}", w, h)),
                ),
        )
        // Caption
        .child(
            div()
                .px_2()
                .py_1()
                .text_size(px(12.0))
                .text_color(ThemeColors::text_primary())
                .child(display_caption.to_string()),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[core::prelude::v1::test]
    fn test_compute_thumbnail_dimensions_landscape() {
        let (w, h) = compute_thumbnail_dimensions(1920, 1080, 320, 240);
        assert_eq!(w, 320);
        assert_eq!(h, 180);
    }

    #[core::prelude::v1::test]
    fn test_compute_thumbnail_dimensions_portrait() {
        let (w, h) = compute_thumbnail_dimensions(1080, 1920, 320, 240);
        assert_eq!(w, 135);
        assert_eq!(h, 240);
    }

    #[core::prelude::v1::test]
    fn test_compute_thumbnail_dimensions_smaller_than_max() {
        let (w, h) = compute_thumbnail_dimensions(100, 100, 320, 240);
        assert_eq!(w, 100);
        assert_eq!(h, 100);
    }

    #[core::prelude::v1::test]
    fn test_compute_thumbnail_dimensions_zero() {
        assert_eq!(compute_thumbnail_dimensions(0, 100, 320, 240), (0, 0));
        assert_eq!(compute_thumbnail_dimensions(100, 0, 320, 240), (0, 0));
    }

    #[core::prelude::v1::test]
    fn test_read_image_metadata_nonexistent() {
        let path = Path::new("/nonexistent/photo.jpg");
        let result = read_image_metadata(path);
        assert!(matches!(result, Err(MediaError::NotFound(_))));
    }

    #[core::prelude::v1::test]
    fn test_read_image_metadata_valid_png() {
        // Create 2x2 test PNG image in temp file
        let img = image::RgbImage::new(2, 2);
        let temp = NamedTempFile::new().unwrap();
        img.save_with_format(temp.path(), image::ImageFormat::Png).unwrap();

        let meta = read_image_metadata(temp.path()).unwrap();
        assert_eq!(meta.width, 2);
        assert_eq!(meta.height, 2);
        assert_eq!(meta.format, "Png");
        assert!(meta.file_size > 0);
    }

    #[core::prelude::v1::test]
    fn test_render_image_preview_creation() {
        let _div = render_image_preview(Some("/tmp/pic.jpg"), Some("Vacation"), Some((800, 600)));
    }
}
