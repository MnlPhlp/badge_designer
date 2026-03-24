use serde::{Deserialize, Serialize};

pub type FrameData = [[bool; 44]; 11];

#[cfg(target_arch = "wasm32")]
const STORAGE_KEY: &str = "badge_designer_state";

#[derive(Serialize, Deserialize, Clone, Debug)]
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

    pub fn load() -> Self {
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

    pub fn save(&self) {
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
    pub frames: Vec<FrameData>,
    pub drawing: bool,
    pub draw_value: bool,
}

impl BadgeDesigner {
    pub fn new() -> Self {
        let state = BadgeState::load();
        let frames = state.to_frames();
        let frames = if frames.is_empty() {
            vec![[[false; 44]; 11]]
        } else {
            frames
        };

        Self {
            frames,
            drawing: false,
            draw_value: true,
        }
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

impl Default for BadgeDesigner {
    fn default() -> Self {
        Self::new()
    }
}
