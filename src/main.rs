use badge_designer::{BadgeDesigner, BadgeState};
use iced::widget::{
    button, canvas, canvas::Cache, column, container, row, scrollable, text, Column,
};
use iced::widget::{space, Action};
use iced::{event, mouse, Color, Element, Length, Point, Rectangle, Renderer, Size, Theme};

const ROWS: usize = 11;
const COLS: usize = 44;
const CELL_SIZE: f32 = 20.0;

pub fn main() -> iced::Result {
    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
    }

    iced::application(BadgeApp::default, BadgeApp::update, BadgeApp::view)
        // .theme(|_| Theme::Dark)
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    PaddingDecrement,
    PaddingIncrement,
    SpeedDecrement,
    SpeedIncrement,
    AddFrame,
    MakeCycle,
    InvertFrame(usize),
    ClearFrame(usize),
    CloneFrame(usize),
    DeleteFrame(usize),
    ExportConfig,
    ImportConfig,
    PixelGridEvent(usize, PixelGridMessage),
}

#[derive(Debug, Clone)]
enum PixelGridMessage {
    StartDrawing(Point),
    ContinueDrawing(Point),
    StopDrawing,
}

struct BadgeApp {
    designer: BadgeDesigner,
    padding: u8,
    speed: u8,
    pixel_grid_caches: Vec<Cache>,
}

impl Default for BadgeApp {
    fn default() -> Self {
        let state = BadgeState::load();
        let designer = BadgeDesigner::new();
        let frame_count = designer.frame_count();

        Self {
            designer,
            padding: state.padding,
            speed: state.speed,
            pixel_grid_caches: (0..frame_count).map(|_| Cache::default()).collect(),
        }
    }
}

impl BadgeApp {
    fn update(&mut self, message: Message) {
        match message {
            Message::PaddingDecrement => {
                if self.padding > 0 {
                    self.padding -= 1;
                    self.save_state();
                }
            }
            Message::PaddingIncrement => {
                if self.padding < 20 {
                    self.padding += 1;
                    self.save_state();
                }
            }
            Message::SpeedDecrement => {
                if self.speed > 1 {
                    self.speed -= 1;
                    self.save_state();
                }
            }
            Message::SpeedIncrement => {
                if self.speed < 7 {
                    self.speed += 1;
                    self.save_state();
                }
            }
            Message::AddFrame => {
                self.designer.add_frame();
                self.pixel_grid_caches.push(Cache::default());
                self.save_state();
            }
            Message::MakeCycle => {
                let original_count = self.designer.frame_count();
                self.designer.make_cycle();
                let new_count = self.designer.frame_count();
                for _ in original_count..new_count {
                    self.pixel_grid_caches.push(Cache::default());
                }
                self.invalidate_all_caches();
                self.save_state();
            }
            Message::InvertFrame(idx) => {
                self.designer.invert_frame(idx);
                if idx < self.pixel_grid_caches.len() {
                    self.pixel_grid_caches[idx].clear();
                }
                self.save_state();
            }
            Message::ClearFrame(idx) => {
                self.designer.clear_frame(idx);
                if idx < self.pixel_grid_caches.len() {
                    self.pixel_grid_caches[idx].clear();
                }
                self.save_state();
            }
            Message::CloneFrame(idx) => {
                self.designer.clone_frame(idx);
                self.pixel_grid_caches.insert(idx + 1, Cache::default());
                self.save_state();
            }
            Message::DeleteFrame(idx) => {
                if self.designer.frame_count() > 1 {
                    self.designer.delete_frame(idx);
                    if idx < self.pixel_grid_caches.len() {
                        self.pixel_grid_caches.remove(idx);
                    }
                    self.save_state();
                }
            }
            Message::ExportConfig => {
                let config = self.designer.export_config(self.padding, self.speed);
                self.export_file(config);
            }
            Message::ImportConfig => {
                self.import_file();
            }
            Message::PixelGridEvent(frame_idx, grid_msg) => {
                self.handle_pixel_grid_event(frame_idx, grid_msg);
            }
        }
    }

    fn handle_pixel_grid_event(&mut self, frame_idx: usize, msg: PixelGridMessage) {
        match msg {
            PixelGridMessage::StartDrawing(point) => {
                let x = (point.x / CELL_SIZE) as usize;
                let y = (point.y / CELL_SIZE) as usize;
                if x < COLS && y < ROWS {
                    self.designer.start_drawing(frame_idx, x, y);
                    if frame_idx < self.pixel_grid_caches.len() {
                        self.pixel_grid_caches[frame_idx].clear();
                    }
                }
            }
            PixelGridMessage::ContinueDrawing(point) => {
                let x = (point.x / CELL_SIZE) as usize;
                let y = (point.y / CELL_SIZE) as usize;
                if x < COLS && y < ROWS && self.designer.drawing {
                    self.designer.continue_drawing(frame_idx, x, y);
                    if frame_idx < self.pixel_grid_caches.len() {
                        self.pixel_grid_caches[frame_idx].clear();
                    }
                }
            }
            PixelGridMessage::StopDrawing => {
                self.designer.stop_drawing();
                self.save_state();
            }
        }
    }

    fn save_state(&self) {
        self.designer.save(self.padding, self.speed);
    }

    fn invalidate_all_caches(&mut self) {
        for cache in &mut self.pixel_grid_caches {
            cache.clear();
        }
    }

    fn export_file(&self, config: String) {
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
            wasm_bindgen_futures::spawn_local(async move {
                let task = rfd::AsyncFileDialog::new()
                    .add_filter("TOML", &["toml"])
                    .set_file_name("badge.toml")
                    .save_file();
                if let Some(handle) = task.await {
                    let _ = handle.write(config.as_bytes()).await;
                }
            });
        }
    }

    fn import_file(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("TOML", &["toml"])
                .pick_file()
            {
                if let Ok(contents) = std::fs::read_to_string(path) {
                    if let Ok((padding, speed)) = self.designer.import_config(&contents) {
                        self.padding = padding;
                        self.speed = speed;
                        let new_count = self.designer.frame_count();
                        self.pixel_grid_caches = (0..new_count).map(|_| Cache::default()).collect();
                        self.save_state();
                    }
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            // For WASM, we need to spawn async task
            // This is a simplified version - in production would need proper message passing
            wasm_bindgen_futures::spawn_local(async move {
                let task = rfd::AsyncFileDialog::new()
                    .add_filter("TOML", &["toml"])
                    .pick_file();
                if let Some(handle) = task.await {
                    let contents = handle.read().await;
                    if let Ok(text) = String::from_utf8(contents) {
                        // Would need to send message back to app here
                        // For now this is a limitation of the direct conversion
                        eprintln!("Import on WASM needs message channel: {}", text);
                    }
                }
            });
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let header = column![
            text("Badge Designer").size(28),
            text("Design animations for LED badges. Export configs to flash with badgemagic-rs.")
                .size(12),
        ]
        .spacing(5);

        let controls = row![
            text("Padding between frames:"),
            button("-").on_press(Message::PaddingDecrement),
            text(self.padding.to_string()).width(20),
            button("+").on_press(Message::PaddingIncrement),
            text("   "),
            text("Speed:"),
            button("-").on_press(Message::SpeedDecrement),
            text(self.speed.to_string()).width(20),
            button("+").on_press(Message::SpeedIncrement),
        ]
        .spacing(10);

        let mut frames_column = Column::new().spacing(20);
        for frame_idx in 0..self.designer.frame_count() {
            let frame_view = self.view_frame(frame_idx);
            frames_column = frames_column.push(frame_view);
        }

        let frames_scroll = row![
            space::horizontal(),
            scrollable(frames_column).height(Length::Fill),
            space::horizontal(),
        ];

        let bottom_controls = row![
            button("Add Frame").on_press(Message::AddFrame),
            button("Make Cycle").on_press(Message::MakeCycle),
            space::horizontal(),
            button("Export").on_press(Message::ExportConfig),
            button("Import").on_press(Message::ImportConfig),
        ]
        .spacing(10);

        container(
            column![header, controls, frames_scroll, bottom_controls]
                .spacing(10)
                .padding(10),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn view_frame(&self, frame_idx: usize) -> Element<'_, Message> {
        let pixels = self.designer.get_frame_pixels(frame_idx);

        let grid = canvas(PixelGrid {
            pixels,
            frame_idx,
            cache: &self.pixel_grid_caches[frame_idx],
        })
        .width(COLS as f32 * CELL_SIZE)
        .height(ROWS as f32 * CELL_SIZE);

        let buttons = column![
            space::vertical(),
            button("Invert").on_press(Message::InvertFrame(frame_idx)),
            button("Clear").on_press(Message::ClearFrame(frame_idx)),
            button("Clone").on_press(Message::CloneFrame(frame_idx)),
            button("Delete").on_press(Message::DeleteFrame(frame_idx)),
            space::vertical(),
        ]
        .spacing(5);

        container(row![grid, buttons].spacing(10).padding(10))
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.15, 0.15, 0.15))),
                border: iced::Border {
                    color: Color::from_rgb(0.3, 0.3, 0.3),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            })
            .into()
    }
}

struct PixelGrid<'a> {
    pixels: Vec<bool>,
    frame_idx: usize,
    cache: &'a Cache,
}

impl<'a> canvas::Program<Message> for PixelGrid<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let background = canvas::Path::rectangle(Point::ORIGIN, bounds.size());
            frame.fill(&background, Color::from_rgb(0.1, 0.1, 0.1));

            for row in 0..ROWS {
                for col in 0..COLS {
                    let idx = row * COLS + col;
                    let is_on = idx < self.pixels.len() && self.pixels[idx];

                    let x = col as f32 * CELL_SIZE;
                    let y = row as f32 * CELL_SIZE;

                    let cell = canvas::Path::rectangle(
                        Point::new(x, y),
                        Size::new(CELL_SIZE - 1.0, CELL_SIZE - 1.0),
                    );

                    let color = if is_on { Color::WHITE } else { Color::BLACK };

                    frame.fill(&cell, color);
                }
            }
        });

        vec![geometry]
    }

    fn update(
        &self,
        _state: &mut Self::State,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        if let Some(position) = cursor.position_in(bounds) {
            match event {
                canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                    return Some(Action::publish(Message::PixelGridEvent(
                        self.frame_idx,
                        PixelGridMessage::StartDrawing(position),
                    )));
                }
                canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                    return Some(Action::publish(Message::PixelGridEvent(
                        self.frame_idx,
                        PixelGridMessage::ContinueDrawing(position),
                    )));
                }
                canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                    return Some(Action::publish(Message::PixelGridEvent(
                        self.frame_idx,
                        PixelGridMessage::StopDrawing,
                    )));
                }
                _ => {}
            }
        } else if let canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) =
            event
        {
            return Some(Action::publish(Message::PixelGridEvent(
                self.frame_idx,
                PixelGridMessage::StopDrawing,
            )));
        }

        None
    }
}
