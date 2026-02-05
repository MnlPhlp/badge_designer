use serde::{Deserialize, Serialize};
use slint::Model;
use std::cell::RefCell;
use std::rc::Rc;

slint::include_modules!();

pub type FrameData = [[bool; 44]; 11];

#[cfg(target_arch = "wasm32")]
const STORAGE_KEY: &str = "badge_designer_state";

#[derive(Serialize, Deserialize, Clone)]
pub struct BadgeState {
    pub frames: Vec<Vec<Vec<bool>>>,
    pub padding: u8,
    pub speed: u8,
}

impl Default for BadgeState {
    fn default() -> Self {
        Self {
            frames: vec![vec![vec![false; 44]; 11]],
            padding: 0,
            speed: 5,
        }
    }
}

impl BadgeState {
    pub fn from_frames(frames: &[FrameData], padding: u8, speed: u8) -> Self {
        Self {
            frames: frames
                .iter()
                .map(|frame| frame.iter().map(|row| row.to_vec()).collect())
                .collect(),
            padding,
            speed,
        }
    }

    pub fn to_frames(&self) -> Vec<FrameData> {
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

    fn load() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(proj_dirs) =
                directories::ProjectDirs::from("com", "badge-designer", "badge-designer")
            {
                let config_dir = proj_dirs.config_dir();
                let state_file = config_dir.join("state.json");
                if let Ok(contents) = std::fs::read_to_string(state_file) {
                    if let Ok(state) = serde_json::from_str(&contents) {
                        return state;
                    }
                }
            }
            Self::default()
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    if let Ok(Some(data)) = storage.get_item(STORAGE_KEY) {
                        if let Ok(state) = serde_json::from_str(&data) {
                            return state;
                        }
                    }
                }
            }
            Self::default()
        }
    }

    fn save(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(proj_dirs) =
                directories::ProjectDirs::from("com", "badge-designer", "badge-designer")
            {
                let config_dir = proj_dirs.config_dir();
                let _ = std::fs::create_dir_all(config_dir);
                let state_file = config_dir.join("state.json");
                if let Ok(json) = serde_json::to_string(self) {
                    let _ = std::fs::write(state_file, json);
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    if let Ok(json) = serde_json::to_string(self) {
                        let _ = storage.set_item(STORAGE_KEY, &json);
                    }
                }
            }
        }
    }
}

pub struct BadgeDesigner {
    frames: Vec<FrameData>,
    drawing: bool,
    draw_value: bool,
}

impl BadgeDesigner {
    pub fn new() -> Rc<RefCell<Self>> {
        let state = BadgeState::load();
        let frames = state.to_frames();
        let frames = if frames.is_empty() {
            vec![[[false; 44]; 11]]
        } else {
            frames
        };

        Rc::new(RefCell::new(Self {
            frames,
            drawing: false,
            draw_value: true,
        }))
    }

    pub fn save(&self, padding: u8, speed: u8) {
        let state = BadgeState::from_frames(&self.frames, padding, speed);
        state.save();
    }

    pub fn get_frame_pixels(&self, frame_idx: usize) -> Vec<bool> {
        if frame_idx >= self.frames.len() {
            return vec![false; 11 * 44];
        }

        let frame = &self.frames[frame_idx];
        let mut pixels = Vec::with_capacity(11 * 44);
        for y in 0..11 {
            for x in 0..44 {
                pixels.push(frame[y][x]);
            }
        }
        pixels
    }

    pub fn set_pixel(&mut self, frame_idx: usize, x: usize, y: usize, value: bool) {
        if frame_idx < self.frames.len() && x < 44 && y < 11 {
            self.frames[frame_idx][y][x] = value;
        }
    }

    pub fn start_drawing(&mut self, frame_idx: usize, x: usize, y: usize) {
        if frame_idx >= self.frames.len() || x >= 44 || y >= 11 {
            return;
        }
        self.draw_value = !self.frames[frame_idx][y][x];
        self.drawing = true;
        self.frames[frame_idx][y][x] = self.draw_value;
    }

    pub fn continue_drawing(&mut self, frame_idx: usize, x: usize, y: usize) {
        if !self.drawing {
            return;
        }
        if frame_idx < self.frames.len() && x < 44 && y < 11 {
            self.frames[frame_idx][y][x] = self.draw_value;
        }
    }

    pub fn stop_drawing(&mut self) {
        self.drawing = false;
    }

    pub fn add_frame(&mut self) {
        let last = self.frames.last().copied().unwrap_or([[false; 44]; 11]);
        self.frames.push(last);
    }

    pub fn make_cycle(&mut self) {
        let reversed: Vec<FrameData> = self.frames.iter().rev().copied().collect();
        self.frames.extend(reversed);
    }

    pub fn invert_frame(&mut self, frame_idx: usize) {
        if frame_idx >= self.frames.len() {
            return;
        }
        for y in 0..11 {
            for x in 0..44 {
                self.frames[frame_idx][y][x] = !self.frames[frame_idx][y][x];
            }
        }
    }

    pub fn clear_frame(&mut self, frame_idx: usize) {
        if frame_idx >= self.frames.len() {
            return;
        }
        self.frames[frame_idx] = [[false; 44]; 11];
    }

    pub fn clone_frame(&mut self, frame_idx: usize) {
        if frame_idx >= self.frames.len() {
            return;
        }
        let cloned = self.frames[frame_idx];
        self.frames.insert(frame_idx + 1, cloned);
    }

    pub fn delete_frame(&mut self, frame_idx: usize) {
        if frame_idx >= self.frames.len() || self.frames.len() <= 1 {
            return;
        }
        self.frames.remove(frame_idx);
    }

    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    pub fn export_config(&self, padding: u8, speed: u8) -> String {
        let mut bitstring = String::new();
        for y in 0..11 {
            for frame in &self.frames {
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

    pub fn import_config(&mut self, config: &str) -> Result<(u8, u8), String> {
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

        if frames.is_empty() {
            return Err("No frames found in config".to_string());
        }

        self.frames = frames;
        Ok((padding, speed))
    }
}

pub fn run_ui() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;
    let designer = BadgeDesigner::new();

    // Load initial state
    let state = BadgeState::load();

    // Initialize UI state
    {
        let d = designer.borrow();
        ui.set_frame_count(d.frame_count() as i32);
        ui.set_frame_padding(state.padding as i32);
        ui.set_speed(state.speed as i32);
    }

    // Helper to update frames data in UI
    let update_frames = {
        let ui_weak = ui.as_weak();
        let designer_weak = Rc::downgrade(&designer);
        move || {
            if let (Some(ui), Some(designer)) = (ui_weak.upgrade(), designer_weak.upgrade()) {
                let d = designer.borrow();
                let frames_vec: Vec<slint::ModelRc<bool>> = (0..d.frame_count())
                    .map(|i| {
                        let pixels = d.get_frame_pixels(i);
                        slint::ModelRc::new(slint::VecModel::from(pixels))
                    })
                    .collect();
                ui.set_frames_data(slint::ModelRc::new(slint::VecModel::from(frames_vec)));
            }
        }
    };

    // Initial frames update
    update_frames();

    // Helper to save state
    let save_state = {
        let ui_weak = ui.as_weak();
        let designer_weak = Rc::downgrade(&designer);
        move || {
            if let (Some(ui), Some(designer)) = (ui_weak.upgrade(), designer_weak.upgrade()) {
                let padding = ui.get_frame_padding() as u8;
                let speed = ui.get_speed() as u8;
                designer.borrow().save(padding, speed);
            }
        }
    };

    // Add frame callback
    {
        let ui_weak = ui.as_weak();
        let designer_weak = Rc::downgrade(&designer);
        let update_frames = update_frames.clone();
        ui.on_add_frame(move || {
            if let (Some(ui), Some(designer)) = (ui_weak.upgrade(), designer_weak.upgrade()) {
                designer.borrow_mut().add_frame();
                ui.set_frame_count(designer.borrow().frame_count() as i32);
                update_frames();
            }
        });
    }

    // Make cycle callback
    {
        let ui_weak = ui.as_weak();
        let designer_weak = Rc::downgrade(&designer);
        let update_frames = update_frames.clone();
        ui.on_make_cycle(move || {
            if let (Some(ui), Some(designer)) = (ui_weak.upgrade(), designer_weak.upgrade()) {
                designer.borrow_mut().make_cycle();
                ui.set_frame_count(designer.borrow().frame_count() as i32);
                update_frames();
            }
        });
    }

    // Pixel drawing callbacks (modify UI model directly)
    {
        let ui_weak = ui.as_weak();
        let designer_weak = Rc::downgrade(&designer);

        ui.on_start_drawing(move |frame_idx, x, y| {
            println!("Start drawing at frame {}, x {}, y {}", frame_idx, x, y);
            if let (Some(ui), Some(designer)) = (ui_weak.upgrade(), designer_weak.upgrade()) {
                let frames_data = ui.get_frames_data();
                if frame_idx >= 0 && (frame_idx as usize) < frames_data.row_count() {
                    let frame_model = frames_data.row_data(frame_idx as usize).unwrap();
                    let index = (y * 44 + x) as usize;
                    if index < frame_model.row_count() {
                        let current = frame_model.row_data(index).unwrap();
                        let new_value = !current;
                        frame_model.set_row_data(index, new_value);

                        // Store draw state in designer
                        let mut d = designer.borrow_mut();
                        d.draw_value = new_value;
                        d.drawing = true;
                    }
                }
            }
        });

        let ui_weak2 = ui.as_weak();
        let designer_weak2 = Rc::downgrade(&designer);

        ui.on_continue_drawing(move |frame_idx, x, y| {
            if let (Some(ui), Some(designer)) = (ui_weak2.upgrade(), designer_weak2.upgrade()) {
                let d = designer.borrow();
                if !d.drawing {
                    return;
                }
                println!("Continue drawing at frame {}, x {}, y {}", frame_idx, x, y);
                let draw_value = d.draw_value;
                drop(d);

                let frames_data = ui.get_frames_data();
                if frame_idx >= 0 && (frame_idx as usize) < frames_data.row_count() {
                    let frame_model = frames_data.row_data(frame_idx as usize).unwrap();
                    let index = (y * 44 + x) as usize;
                    if index < frame_model.row_count() {
                        frame_model.set_row_data(index, draw_value);
                    }
                }
            }
        });

        ui.on_stop_drawing({
            let designer = designer.clone();
            move || {
                println!("Stop drawing");
                designer.borrow_mut().drawing = false;
            }
        });
    }

    // Invert frame callback
    {
        let designer_weak = Rc::downgrade(&designer);
        let update_frames = update_frames.clone();
        ui.on_invert_frame(move |frame_idx| {
            if let Some(designer) = designer_weak.upgrade() {
                designer.borrow_mut().invert_frame(frame_idx as usize);
                update_frames();
            }
        });
    }

    // Clear frame callback
    {
        let designer_weak = Rc::downgrade(&designer);
        let update_frames = update_frames.clone();
        ui.on_clear_frame(move |frame_idx| {
            if let Some(designer) = designer_weak.upgrade() {
                designer.borrow_mut().clear_frame(frame_idx as usize);
                update_frames();
            }
        });
    }

    // Clone frame callback
    {
        let ui_weak = ui.as_weak();
        let designer_weak = Rc::downgrade(&designer);
        let update_frames = update_frames.clone();
        ui.on_clone_frame(move |frame_idx| {
            if let (Some(ui), Some(designer)) = (ui_weak.upgrade(), designer_weak.upgrade()) {
                designer.borrow_mut().clone_frame(frame_idx as usize);
                ui.set_frame_count(designer.borrow().frame_count() as i32);
                update_frames();
            }
        });
    }

    // Delete frame callback
    {
        let ui_weak = ui.as_weak();
        let designer_weak = Rc::downgrade(&designer);
        let update_frames = update_frames.clone();
        ui.on_delete_frame(move |frame_idx| {
            if let (Some(ui), Some(designer)) = (ui_weak.upgrade(), designer_weak.upgrade()) {
                designer.borrow_mut().delete_frame(frame_idx as usize);
                ui.set_frame_count(designer.borrow().frame_count() as i32);
                update_frames();
            }
        });
    }

    // Export callback
    {
        let ui_weak = ui.as_weak();
        let designer_weak = Rc::downgrade(&designer);
        ui.on_export_config(move || {
            if let (Some(ui), Some(designer)) = (ui_weak.upgrade(), designer_weak.upgrade()) {
                let padding = ui.get_frame_padding() as u8;
                let speed = ui.get_speed() as u8;
                let config = designer.borrow().export_config(padding, speed);

                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("TOML", &["toml"])
                        .set_file_name("badge.toml")
                        .save_file()
                    {
                        let _ = std::fs::write(path, config);
                    }
                }

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
            }
        });
    }

    // Import callback
    {
        let ui_weak = ui.as_weak();
        let designer_weak = Rc::downgrade(&designer);
        let update_frames = update_frames.clone();
        ui.on_import_config(move || {
            #[cfg(not(target_arch = "wasm32"))]
            {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("TOML", &["toml"])
                    .pick_file()
                {
                    if let Ok(contents) = std::fs::read_to_string(path) {
                        if let (Some(ui), Some(designer)) =
                            (ui_weak.upgrade(), designer_weak.upgrade())
                        {
                            let result = designer.borrow_mut().import_config(&contents);
                            if let Ok((padding, speed)) = result {
                                ui.set_frame_padding(padding as i32);
                                ui.set_speed(speed as i32);
                                ui.set_frame_count(designer.borrow().frame_count() as i32);
                                update_frames();
                            }
                        }
                    }
                }
            }

            #[cfg(target_arch = "wasm32")]
            {
                let ui_weak_clone = ui_weak.clone();
                let designer_weak_clone = designer_weak.clone();
                let update_frames_clone = update_frames.clone();
                let task = rfd::AsyncFileDialog::new()
                    .add_filter("TOML", &["toml"])
                    .pick_file();
                wasm_bindgen_futures::spawn_local(async move {
                    if let Some(handle) = task.await {
                        let contents = handle.read().await;
                        if let Ok(contents) = String::from_utf8(contents) {
                            if let (Some(ui), Some(designer)) =
                                (ui_weak_clone.upgrade(), designer_weak_clone.upgrade())
                            {
                                let result = designer.borrow_mut().import_config(&contents);
                                if let Ok((padding, speed)) = result {
                                    ui.set_frame_padding(padding as i32);
                                    ui.set_speed(speed as i32);
                                    ui.set_frame_count(designer.borrow().frame_count() as i32);
                                    update_frames_clone();
                                }
                            }
                        }
                    }
                });
            }
        });
    }

    ui.run()
}
