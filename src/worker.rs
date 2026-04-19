use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Instant;

use ratatui::layout::Rect;
use ratatui_image::protocol::StatefulProtocol;

use crate::viewer::ImageViewer;

pub enum ImageWork {
    Generate {
        scale: f32,
        offset_x: i32,
        offset_y: i32,
        area: Rect,
    },
}

pub enum ImageResult {
    Ready(StatefulProtocol),
    Error(String),
}

pub struct ImageWorker {
    tx: Sender<ImageWork>,
    rx: Receiver<ImageResult>,
}

impl ImageWorker {
    pub fn new(viewer: ImageViewer, debug: bool) -> Self {
        let (work_tx, work_rx) = channel::<ImageWork>();
        let (result_tx, result_rx) = channel::<ImageResult>();

        thread::spawn(move || {
            while let Ok(work) = work_rx.recv() {
                // Skip any stale work that accumulated while we were processing
                let mut latest = work;
                while let Ok(newer) = work_rx.try_recv() {
                    latest = newer;
                }

                match latest {
                    ImageWork::Generate {
                        scale,
                        offset_x,
                        offset_y,
                        area,
                    } => {
                        let work_start = Instant::now();
                        match viewer.create_stateful(scale, offset_x, offset_y, area) {
                            Ok(protocol) => {
                                let work_ms = work_start.elapsed().as_secs_f64() * 1000.0;
                                if debug && work_ms > 1.0 {
                                    eprintln!("[IRIS-BENCH] worker-thread  scale={:.1}x  area={}x{}  took={:.2}ms", scale, area.width, area.height, work_ms);
                                }
                                let _ = result_tx.send(ImageResult::Ready(protocol));
                            }
                            Err(e) => {
                                let work_ms = work_start.elapsed().as_secs_f64() * 1000.0;
                                if debug {
                                    eprintln!("[IRIS-BENCH] worker-thread  scale={:.1}x  area={}x{}  ERROR took={:.2}ms err={}", scale, area.width, area.height, work_ms, e);
                                }
                                let _ = result_tx.send(ImageResult::Error(e.to_string()));
                            }
                        }
                    }
                }
            }
        });

        Self {
            tx: work_tx,
            rx: result_rx,
        }
    }

    pub fn request(&self, scale: f32, offset_x: i32, offset_y: i32, area: Rect) {
        let work = ImageWork::Generate {
            scale,
            offset_x,
            offset_y,
            area,
        };
        let _ = self.tx.send(work);
    }

    pub fn try_recv(&self) -> Option<ImageResult> {
        self.rx.try_recv().ok()
    }
}
