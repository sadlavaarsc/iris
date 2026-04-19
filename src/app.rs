use ratatui::layout::Rect;
use ratatui_image::protocol::StatefulProtocol;

pub struct App {
    pub scale: f32,
    pub offset_x: i32,
    pub offset_y: i32,
    pub image_state: Option<StatefulProtocol>,
    #[allow(dead_code)]
    pub original_width: u32,
    #[allow(dead_code)]
    pub original_height: u32,
    pub should_quit: bool,
    pub status_message: String,
    // Cache of last parameters used to create image_state
    pub last_scale: f32,
    pub last_offset_x: i32,
    pub last_offset_y: i32,
    pub last_area: Rect,
    pub has_pending_work: bool,
    /// The actual area where the image is rendered (may be smaller than terminal)
    pub image_area: Rect,
    pub debug: bool,
    /// Last known drag position for mouse panning
    pub drag_last_col: Option<u16>,
    pub drag_last_row: Option<u16>,
    /// Scale used on first load / after reset (fit-to-area)
    pub initial_scale: f32,
    pub filename: String,
    pub image_path: std::path::PathBuf,
}

impl App {
    pub fn new(original_width: u32, original_height: u32, debug: bool, initial_scale: f32, filename: impl Into<String>, image_path: std::path::PathBuf) -> Self {
        Self {
            scale: initial_scale,
            offset_x: 0,
            offset_y: 0,
            image_state: None,
            original_width,
            original_height,
            should_quit: false,
            status_message: String::new(),
            last_scale: initial_scale,
            last_offset_x: 0,
            last_offset_y: 0,
            last_area: Rect::default(),
            has_pending_work: false,
            image_area: Rect::default(),
            debug,
            drag_last_col: None,
            drag_last_row: None,
            initial_scale,
            filename: filename.into(),
            image_path,
        }
    }

    pub fn zoom_in(&mut self) {
        self.scale = (self.scale * 1.25).min(10.0);
        self.update_status();
    }

    pub fn zoom_out(&mut self) {
        self.scale = (self.scale / 1.25).max(0.1);
        self.update_status();
    }

    pub fn pan(&mut self, dx: i32, dy: i32) {
        self.offset_x += dx;
        self.offset_y += dy;
    }

    pub fn reset_view(&mut self) {
        self.scale = self.initial_scale;
        self.offset_x = 0;
        self.offset_y = 0;
        self.update_status();
    }

    fn update_status(&mut self) {
        self.status_message = format!(
            "Scale: {:.0}% | Offset: ({}, {}) cells",
            self.scale * 100.0,
            self.offset_x,
            self.offset_y
        );
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = msg.into();
    }
}
