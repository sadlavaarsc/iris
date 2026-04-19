use anyhow::{Context, Result};
use image::{imageops, DynamicImage};
use ratatui::layout::Rect;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use ratatui_image::Resize;
use std::path::Path;

#[derive(Clone)]
pub struct ImageViewer {
    picker: Picker,
    original_image: DynamicImage,
}

impl ImageViewer {
    pub fn new(path: &Path) -> Result<Self> {
        let original_image = image::open(path)
            .with_context(|| format!("Failed to open image: {}", path.display()))?;

        let picker = Picker::halfblocks();

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

        let font_size = self.picker.font_size();
        let cell_w = font_size.0 as u32;
        let cell_h = font_size.1 as u32;

        let view_width_px = (area.width as u32 * cell_w).max(1);
        let view_height_px = (area.height as u32 * cell_h).max(1);

        // Fast path: no scale, no offset, image fits in view — return a clone
        if scale == 1.0 && offset_x == 0 && offset_y == 0 && orig_w <= view_width_px && orig_h <= view_height_px {
            return self.original_image.clone();
        }

        // Calculate the region of the original image that the viewport needs.
        // The viewport shows `view_width_px` pixels at `scale`, which maps to
        // `view_width_px / scale` pixels in the original image.
        let offset_x_px = (offset_x.max(0) as f32) * cell_w as f32;
        let offset_y_px = (offset_y.max(0) as f32) * cell_h as f32;

        let src_x = (offset_x_px / scale)
            .min(orig_w.saturating_sub(1) as f32) as u32;
        let src_y = (offset_y_px / scale)
            .min(orig_h.saturating_sub(1) as f32) as u32;

        let src_w = ((view_width_px as f32 / scale).ceil() as u32)
            .min(orig_w.saturating_sub(src_x))
            .max(1);
        let src_h = ((view_height_px as f32 / scale).ceil() as u32)
            .min(orig_h.saturating_sub(src_y))
            .max(1);

        // 1. Crop from the original image (O(src_w * src_h) instead of O(orig_w * orig_h)).
        let cropped = imageops::crop_imm(&self.original_image, src_x, src_y, src_w, src_h);

        // 2. Resize the cropped region to the viewport size.
        let dst_w = (src_w as f32 * scale) as u32;
        let dst_h = (src_h as f32 * scale) as u32;

        let dst_w = dst_w.min(view_width_px).max(1);
        let dst_h = dst_h.min(view_height_px).max(1);

        let cropped_buffer = cropped.to_image();

        let resized = if dst_w == src_w && dst_h == src_h {
            cropped_buffer
        } else {
            imageops::resize(&cropped_buffer, dst_w, dst_h, imageops::FilterType::Triangle)
        };

        // If the resized image is smaller than the viewport, pad it so that
        // the content is centered in the render area.
        if dst_w < view_width_px || dst_h < view_height_px {
            let mut padded = image::RgbaImage::from_pixel(
                view_width_px,
                view_height_px,
                image::Rgba([0, 0, 0, 0]),
            );
            let px = (view_width_px - dst_w) / 2;
            let py = (view_height_px - dst_h) / 2;
            imageops::overlay(&mut padded, &resized, px as i64, py as i64);
            return DynamicImage::ImageRgba8(padded);
        }

        DynamicImage::ImageRgba8(resized)
    }

    // Benchmark helpers — expose internals for latency measurement
    pub fn scaled_image_for_bench(
        &self,
        scale: f32,
        offset_x: i32,
        offset_y: i32,
        area: Rect,
    ) -> DynamicImage {
        self.scaled_image(scale, offset_x, offset_y, area)
    }

    pub fn create_protocol_from_image(&self, image: DynamicImage) -> StatefulProtocol {
        self.picker.new_resize_protocol(image)
    }

    pub fn protocol_name(&self) -> String {
        format!("{:?}", self.picker.protocol_type())
    }

    pub fn font_size(&self) -> (u16, u16) {
        self.picker.font_size()
    }

    /// Compute a scale that fits the entire image inside the given area,
    /// preserving aspect ratio.  Images smaller than the viewport are not
    /// enlarged (scale is capped at 1.0).
    pub fn fit_scale(&self, area: Rect) -> f32 {
        let (orig_w, orig_h) = self.original_size();
        let font_size = self.picker.font_size();
        let view_w = (area.width as u32 * font_size.0 as u32).max(1);
        let view_h = (area.height as u32 * font_size.1 as u32).max(1);

        let scale_x = view_w as f32 / orig_w as f32;
        let scale_y = view_h as f32 / orig_h as f32;
        scale_x.min(scale_y).min(1.0)
    }
}
