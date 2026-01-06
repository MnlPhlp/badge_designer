#![allow(clippy::needless_range_loop)]

use eframe::egui;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{channel, Receiver, Sender};

pub type FrameData = [[bool; 44]; 11];

/// Persistent state that survives application reloads (uses Vec for serde compatibility)
#[derive(Serialize, Deserialize)]
struct PersistentState {
    frames: Vec<Vec<Vec<bool>>>,
    padding: u8,
    speed: u8,
}

impl PersistentState {
    fn from_frames(frames: &[FrameData], padding: u8, speed: u8) -> Self {
        Self {
            frames: frames
                .iter()
                .map(|frame| frame.iter().map(|row| row.to_vec()).collect())
                .collect(),
            padding,
            speed,
        }
    }

    fn to_frames(&self) -> Vec<FrameData> {
        self.frames
            .iter()
            .map(|frame| {
                let mut arr = [[false; 44]; 11];
                for (y, row) in frame.iter().enumerate().take(11) {
                    for (x, &val) in row.iter().enumerate().take(44) {
                        arr[y][x] = val;
                    }
                }
                arr
            })
            .collect()
    }
}

pub fn create_config(frames: &[FrameData], padding: u8, speed: u8) -> String {
    let mut bitstring = String::new();
    for y in 0..11 {
        for frame in frames {
            for x in 0..44 {
                bitstring.push(if frame[y][x] { 'X' } else { '_' });
            }
            for _ in 0..padding {
                bitstring.push('_');
            }
        }
        bitstring.push('\n');
    }
    format!(
        r#"[[message]]
speed = {speed}
mode = "fast"
# padding is not used by badgemagic-rs, we just store it for the editor
padding = {padding}
bitstring = """
{bitstring}""""#
    )
}

pub fn load_config(config: &str) -> (Vec<FrameData>, u8, u8) {
    let mut frames = Vec::new();
    let mut speed = 5;
    let mut padding = 0;
    let mut in_bitstring = false;
    let mut current_frame: Vec<FrameData> = vec![];
    let mut current_y = 0;
    for line in config.lines() {
        if line.starts_with("speed =") {
            if let Some(s) = line.split('=').nth(1) {
                speed = s.trim().parse().unwrap_or(5);
            }
        } else if line.starts_with("padding =") {
            if let Some(s) = line.split('=').nth(1) {
                padding = s.trim().parse().unwrap_or_default();
            }
        } else if line.starts_with("bitstring =") {
            in_bitstring = true;
        } else if in_bitstring {
            if line.trim() == "\"\"\"" {
                in_bitstring = false;
                continue;
            }
            let chars: Vec<char> = line.chars().collect();
            let row_len = 44 + padding as usize;
            let num_frames = chars.len() / row_len;
            for frame_index in 0..num_frames {
                if current_frame.len() <= frame_index {
                    current_frame.push([[false; 44]; 11]);
                }
                for x in 0..44 {
                    let char_index = frame_index * row_len + x;
                    if char_index < chars.len() {
                        current_frame[frame_index][current_y][x] = chars[char_index] == 'X';
                    }
                }
            }
            current_y += 1;
        }
    }
    for frame in current_frame {
        frames.push(frame);
    }
    (frames, padding, speed)
}

pub enum FileOp {
    Import(Vec<FrameData>, u8, u8),
    ExportReady(String),
}

pub struct BadgeDesigner {
    pub frames: Vec<FrameData>,
    pub padding: u8,
    pub speed: u8,
    pub focused_frame: usize,
    pub focused_x: usize,
    pub focused_y: usize,
    pub drawing: bool,
    pub draw_value: bool,
    pub file_rx: Receiver<FileOp>,
    pub file_tx: Sender<FileOp>,
}

const STORAGE_KEY: &str = "badge_designer_state";

impl BadgeDesigner {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rx) = channel();
        
        // Try to restore previous state
        let (frames, padding, speed) = if let Some(storage) = cc.storage {
            if let Some(state) = eframe::get_value::<PersistentState>(storage, STORAGE_KEY) {
                let frames = state.to_frames();
                let frames = if frames.is_empty() {
                    vec![[[false; 44]; 11]]
                } else {
                    frames
                };
                (frames, state.padding, state.speed)
            } else {
                (vec![[[false; 44]; 11]], 0, 5)
            }
        } else {
            (vec![[[false; 44]; 11]], 0, 5)
        };
        
        Self {
            frames,
            padding,
            speed,
            focused_frame: 0,
            focused_x: 0,
            focused_y: 0,
            drawing: false,
            draw_value: true,
            file_tx: tx,
            file_rx: rx,
        }
    }

    pub fn start_drawing(&mut self) {
        if self.drawing {
            return;
        }
        self.draw_value = !self.frames[self.focused_frame][self.focused_y][self.focused_x];
        self.drawing = true;
        self.draw_pixel();
    }

    pub fn draw_pixel(&mut self) {
        if !self.drawing {
            return;
        }
        self.frames[self.focused_frame][self.focused_y][self.focused_x] = self.draw_value;
    }
}

impl eframe::App for BadgeDesigner {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let state = PersistentState::from_frames(&self.frames, self.padding, self.speed);
        eframe::set_value(storage, STORAGE_KEY, &state);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for file operation results
        if let Ok(op) = self.file_rx.try_recv() {
            match op {
                FileOp::Import(new_frames, new_padding, new_speed) => {
                    if !new_frames.is_empty() {
                        self.frames = new_frames;
                        self.padding = new_padding;
                        self.speed = new_speed;
                    }
                }
                FileOp::ExportReady(config) => {
                    #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
                    std::thread::spawn(move || {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("TOML", &["toml"])
                            .set_file_name("badge.toml")
                            .save_file()
                        {
                            let _ = std::fs::write(path, config);
                        }
                    });
                    #[cfg(target_arch = "wasm32")]
                    {
                        let task = rfd::AsyncFileDialog::new()
                            .add_filter("TOML", &["toml"])
                            .set_file_name("badge.toml")
                            .save_file();
                        wasm_bindgen_futures::spawn_local(async move {
                            if let Some(handle) = task.await {
                                let _ = handle.write(config.as_bytes()).await;
                            }
                        });
                    }
                    #[cfg(target_os = "android")]
                    {
                        log::info!("Export not yet supported on Android");
                        let _ = config;
                    }
                }
            }
        }

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.heading("Badge Designer");
            ui.horizontal_wrapped(|ui| {
                ui.label("Design animations for LED badges. Export configs to flash with");
                ui.hyperlink_to("badgemagic-rs", "https://github.com/fossasia/badgemagic-rs");
                ui.label(".");
            });
        });

        egui::TopBottomPanel::bottom("controls").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui.button("Add Frame").clicked() {
                    let last = self.frames.last().copied().unwrap_or([[false; 44]; 11]);
                    self.frames.push(last);
                }

                if ui.button("Make Cycle").clicked() {
                    let reversed: Vec<FrameData> = self.frames.iter().rev().copied().collect();
                    self.frames.extend(reversed);
                }

                ui.separator();

                #[cfg(not(target_os = "android"))]
                {
                    if ui.button("Export").clicked() {
                        let config = create_config(&self.frames, self.padding, self.speed);
                        let _ = self.file_tx.send(FileOp::ExportReady(config));
                    }

                    if ui.button("Import").clicked() {
                        let tx = self.file_tx.clone();
                        let ctx = ctx.clone();
                        #[cfg(not(target_arch = "wasm32"))]
                        std::thread::spawn(move || {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("TOML", &["toml"])
                                .pick_file()
                            {
                                if let Ok(contents) = std::fs::read_to_string(path) {
                                    let (new_frames, new_padding, new_speed) = load_config(&contents);
                                    let _ = tx.send(FileOp::Import(new_frames, new_padding, new_speed));
                                    ctx.request_repaint();
                                }
                            }
                        });
                        #[cfg(target_arch = "wasm32")]
                        {
                            let task = rfd::AsyncFileDialog::new()
                                .add_filter("TOML", &["toml"])
                                .pick_file();
                            wasm_bindgen_futures::spawn_local(async move {
                                if let Some(handle) = task.await {
                                    let contents = handle.read().await;
                                    if let Ok(contents) = String::from_utf8(contents) {
                                        let (new_frames, new_padding, new_speed) =
                                            load_config(&contents);
                                        let _ =
                                            tx.send(FileOp::Import(new_frames, new_padding, new_speed));
                                        ctx.request_repaint();
                                    }
                                }
                            });
                        }
                    }
                }
            });
            ui.add_space(4.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Padding between frames:");
                ui.add(egui::DragValue::new(&mut self.padding).range(0..=20));
            });

            ui.horizontal(|ui| {
                ui.label("Speed:");
                ui.add(egui::DragValue::new(&mut self.speed).range(1..=7));
            });

            ui.separator();

            // Handle keyboard navigation
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) && self.focused_y > 0 {
                self.focused_y -= 1;
                self.draw_pixel();
            }
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) && self.focused_y < 10 {
                self.focused_y += 1;
                self.draw_pixel();
            }
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) && self.focused_x > 0 {
                self.focused_x -= 1;
                self.draw_pixel();
            }
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) && self.focused_x < 43 {
                self.focused_x += 1;
                self.draw_pixel();
            }
            if ctx.input(|i| i.key_down(egui::Key::Space)) {
                self.start_drawing();
            } else if ctx.input(|i| i.key_released(egui::Key::Space)) {
                self.drawing = false;
            }

            // Release drawing on mouse release
            if ctx.input(|i| i.pointer.any_released()) {
                self.drawing = false;
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut frame_to_remove: Option<usize> = None;
                let mut frame_to_clone: Option<usize> = None;

                for frame_index in 0..self.frames.len() {
                    ui.horizontal(|ui| {
                        let is_focused = self.focused_frame == frame_index;

                        // Draw frame grid
                        let cell_size = 12.0;
                        let (response, painter) = ui.allocate_painter(
                            egui::vec2(44.0 * cell_size, 11.0 * cell_size),
                            egui::Sense::click_and_drag(),
                        );
                        let rect = response.rect;

                        for y in 0..11 {
                            for x in 0..44 {
                                let cell_rect = egui::Rect::from_min_size(
                                    rect.min
                                        + egui::vec2(x as f32 * cell_size, y as f32 * cell_size),
                                    egui::vec2(cell_size, cell_size),
                                );

                                let is_on = self.frames[frame_index][y][x];
                                let fill = if is_on {
                                    egui::Color32::WHITE
                                } else {
                                    egui::Color32::BLACK
                                };

                                painter.rect_filled(cell_rect, 0.0, fill);

                                let stroke =
                                    if is_focused && self.focused_x == x && self.focused_y == y {
                                        egui::Stroke::new(2.0, egui::Color32::GREEN)
                                    } else {
                                        egui::Stroke::new(0.5, egui::Color32::GRAY)
                                    };
                                painter.rect_stroke(
                                    cell_rect,
                                    0.0,
                                    stroke,
                                    egui::StrokeKind::Inside,
                                );
                            }
                        }

                        // Handle mouse/touch interaction
                        if let Some(pos) = response.interact_pointer_pos() {
                            let local_pos = pos - rect.min;
                            let x = (local_pos.x / cell_size) as usize;
                            let y = (local_pos.y / cell_size) as usize;
                            if x < 44 && y < 11 && response.is_pointer_button_down_on() {
                                self.focused_frame = frame_index;
                                self.focused_x = x;
                                self.focused_y = y;

                                if self.drawing {
                                    self.draw_pixel();
                                } else {
                                    self.start_drawing();
                                }
                            }
                        }

                        // Frame control buttons
                        ui.vertical(|ui| {
                            if ui.button("Invert").clicked() {
                                for y in 0..11 {
                                    for x in 0..44 {
                                        self.frames[frame_index][y][x] =
                                            !self.frames[frame_index][y][x];
                                    }
                                }
                            }
                            if ui.button("Clear").clicked() {
                                self.frames[frame_index] = [[false; 44]; 11];
                            }
                            if ui.button("Clone").clicked() {
                                frame_to_clone = Some(frame_index);
                            }
                            if ui.button("Delete").clicked() && self.frames.len() > 1 {
                                frame_to_remove = Some(frame_index);
                            }
                        });
                    });
                    ui.add_space(10.0);
                }

                // Apply deferred mutations
                if let Some(idx) = frame_to_clone {
                    let cloned = self.frames[idx];
                    self.frames.insert(idx + 1, cloned);
                }
                if let Some(idx) = frame_to_remove {
                    self.frames.remove(idx);
                    if self.focused_frame >= self.frames.len() {
                        self.focused_frame = self.frames.len().saturating_sub(1);
                    }
                }
            });
        });
    }
}

// Android entry point
#[cfg(target_os = "android")]
use eframe::Renderer;

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: eframe::native::android::AndroidApp) {
    use eframe::native::android::android_activity;
    
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );

    let options = eframe::NativeOptions {
        renderer: Renderer::Glow,
        ..Default::default()
    };
    
    eframe::run_native(
        "Badge Designer",
        options,
        Box::new(|cc| Ok(Box::new(BadgeDesigner::new(cc)))),
    ).unwrap();
}
