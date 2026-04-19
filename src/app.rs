use ratatui_image::protocol::StatefulProtocol;

pub struct App {
    pub scale: f32,
    pub offset_x: i32,
    pub offset_y: i32,
    pub image_state: Option<StatefulProtocol>,
    pub original_width: u32,
    pub original_height: u32,
    pub should_quit: bool,
    pub status_message: String,
}

impl App {
    pub fn new(original_width: u32, original_height: u32) -> Self {
        Self {
            scale: 1.0,
            offset_x: 0,
            offset_y: 0,
            image_state: None,
            original_width,
            original_height,
            should_quit: false,
            status_message: String::new(),
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
        self.scale = 1.0;
        self.offset_x = 0;
        self.offset_y = 0;
        self.update_status();
    }

    fn update_status(&mut self) {
        self.status_message = format!(
            "Scale: {:.0}% | Offset: ({}, {})",
            self.scale * 100.0,
            self.offset_x,
            self.offset_y
        );
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = msg.into();
    }
}
