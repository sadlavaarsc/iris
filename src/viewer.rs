use anyhow::{Context, Result};
use image::{imageops, DynamicImage};
use ratatui::layout::Rect;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use ratatui_image::Resize;
use std::path::Path;

pub struct ImageViewer {
    picker: Picker,
    original_image: DynamicImage,
}

impl ImageViewer {
    pub fn new(path: &Path) -> Result<Self> {
        let original_image = image::open(path)
            .with_context(|| format!("Failed to open image: {}", path.display()))?;

        let picker = Picker::from_fontsize((8, 16));

        Ok(Self {
            picker,
            original_image,
        })
    }

    pub fn query_stdio(mut self) -> Self {
        if let Ok(picker) = Picker::from_query_stdio() {
            self.picker = picker;
        }
        self
    }

    pub fn original_size(&self) -> (u32, u32) {
        (self.original_image.width(), self.original_image.height())
    }

    pub fn create_stateful(
        &self,
        scale: f32,
        offset_x: i32,
        offset_y: i32,
        area: Rect,
    ) -> Result<StatefulProtocol> {
        let scaled = self.scaled_image(scale, offset_x, offset_y, area);
        let protocol = self.picker.new_resize_protocol(scaled);
        Ok(protocol)
    }

    pub fn create_static(
        &self,
        scale: f32,
        offset_x: i32,
        offset_y: i32,
        area: Rect,
    ) -> Result<ratatui_image::protocol::Protocol> {
        let scaled = self.scaled_image(scale, offset_x, offset_y, area);
        let protocol = self
            .picker
            .new_protocol(scaled, area, Resize::Fit(None))
            .context("Failed to create image protocol")?;
        Ok(protocol)
    }

    fn scaled_image(
        &self,
        scale: f32,
        offset_x: i32,
        offset_y: i32,
        area: Rect,
    ) -> DynamicImage {
        let (orig_w, orig_h) = (self.original_image.width(), self.original_image.height());

        if scale == 1.0 && offset_x == 0 && offset_y == 0 {
            return self.original_image.clone();
        }

        let font_size = self.picker.font_size();
        let cell_w = font_size.0 as u32;
        let cell_h = font_size.1 as u32;

        let view_width_px = (area.width as u32 * cell_w).max(1);
        let view_height_px = (area.height as u32 * cell_h).max(1);

        let scaled_w = ((orig_w as f32) * scale).max(1.0) as u32;
        let scaled_h = ((orig_h as f32) * scale).max(1.0) as u32;

        let resized: DynamicImage = if scaled_w != orig_w || scaled_h != orig_h {
            DynamicImage::ImageRgba8(imageops::resize(
                &self.original_image,
                scaled_w,
                scaled_h,
                imageops::FilterType::Lanczos3,
            ))
        } else {
            self.original_image.clone()
        };

        let crop_x = offset_x.clamp(0, scaled_w.saturating_sub(1) as i32) as u32;
        let crop_y = offset_y.clamp(0, scaled_h.saturating_sub(1) as i32) as u32;
        let crop_w = view_width_px.min(scaled_w.saturating_sub(crop_x));
        let crop_h = view_height_px.min(scaled_h.saturating_sub(crop_y));

        if crop_w == 0 || crop_h == 0 {
            return resized;
        }

        if crop_w == scaled_w && crop_h == scaled_h && crop_x == 0 && crop_y == 0 {
            return resized;
        }

        imageops::crop_imm(&resized, crop_x, crop_y, crop_w, crop_h)
            .to_image()
            .into()
    }
}
