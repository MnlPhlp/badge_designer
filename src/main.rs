#![allow(clippy::needless_range_loop)]

use dioxus::prelude::*;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS } document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        Banner {}
        Editor {}

    }
}

#[component]
fn Banner() -> Element {
    rsx! {
        div {
            class: "p-3 mb-4 text-sm text-gray-300 bg-gray-800 border border-gray-700 rounded",
            "Design animations for LED badges. Export configs to flash with "
            a {
                href: "https://github.com/fossasia/badgemagic-rs",
                class: "text-blue-400 underline hover:text-blue-300",
                target: "_blank",
                "badgemagic-rs"
            }
            "."
        }
    }
}

fn create_config(frames: &[Signal<FrameData>], padding: u8, speed: u8) -> String {
    let mut bitstring = String::new();
    for y in 0..11 {
        for frame in frames {
            let f = frame.read();
            for x in 0..44 {
                bitstring.push(if f[y][x] { 'X' } else { '_' });
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
# padding is not used by badgemagic-rs, we just store it for the web editor 
padding = {padding}
bitstring = """
{bitstring}""""#
    )
}

type FrameData = [[bool; 44]; 11];

fn load_config(config: &str) -> (Vec<FrameData>, u8, u8) {
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

#[component]
fn FrameEditor(
    frame_index: usize,
    frame: Signal<FrameData>,
    is_focused: bool,
    focused_x: usize,
    focused_y: usize,
    on_focus: EventHandler<(usize, usize)>,
    on_remove: EventHandler<()>,
    on_clone: EventHandler<()>,
) -> Element {
    let mut adding = use_signal(|| true);
    let mut active = use_signal(|| false);

    rsx! {
        div {
            class: "flex",
            div {
                for y in 0..11 {
                    div {
                        class: "flex",
                        for x in 0..44 {
                            button {
                                class: "w-4 h-4 border",
                                class: if is_focused && focused_x == x && focused_y == y { "border-green-500 border-2" } else { "border-gray-300" },
                                class: if frame.read()[y][x] { " bg-white" } else { "bg-black" },
                                onmousedown: move |_| {
                                    on_focus.call((x, y));
                                    *adding.write() = !frame.read()[y][x];
                                    *active.write() = true;
                                    frame.write()[y][x] = adding();
                                },
                                onmouseenter: move |_| {
                                    if active() {
                                        frame.write()[y][x] = adding();
                                    }
                                },
                                onmouseup: move |_| { *active.write() = false }
                            }
                        }
                    }
                }
            },
            div {
                class: "flex gap-4",
                button {
                    class: "p-2 bg-blue-500 text-white btn",
                    onclick: move |_| {
                        let mut f = frame.write();
                        for y in 0..11 {
                            for x in 0..44 {
                                f[y][x] = !f[y][x];
                            }
                        }
                    },
                    "invert"
                }
                button {
                    class: "p-2 bg-blue-500 text-white btn",
                    onclick: move |_| {
                        *frame.write() = [[false; 44]; 11];
                    },
                    "clear"
                }
                button {
                    class: "p-2 bg-red-500 text-white btn",
                    onclick: move |_| on_remove.call(()),
                    "X"
                }
                button {
                    class: "p-2 bg-purple-500 text-white btn",
                    onclick: move |_| on_clone.call(()),
                    "Clone"
                }
            }
        }
    }
}

#[component]
pub fn Editor() -> Element {
    let mut frames: Signal<Vec<Signal<FrameData>>> =
        use_signal(|| vec![Signal::new([[false; 44]; 11])]);
    let mut padding = use_signal(|| 0u8);
    let mut speed = use_signal(|| 5u8);
    let mut focused_frame = use_signal(|| 0usize);
    let mut focused_x = use_signal(|| 0usize);
    let mut focused_y = use_signal(|| 0usize);
    let mut loaded = use_signal(|| false);

    let storage = web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten();

    // Load from localStorage on mount
    {
        let storage = storage.clone();
        use_effect(move || {
            if let Some(storage) = &storage {
                if let Ok(Some(config)) = storage.get_item("badge_designer_state") {
                    if !config.is_empty() {
                        let (new_frames, new_padding, new_speed) = load_config(&config);
                        if !new_frames.is_empty() {
                            *frames.write() = new_frames.into_iter().map(Signal::new).collect();
                            *padding.write() = new_padding;
                            *speed.write() = new_speed;
                        }
                    }
                }
            }
            *loaded.write() = true;
        });
    }

    // Memo that only produces a value after loaded
    let save_config = use_memo(move || {
        if !loaded() {
            return None;
        }
        Some(create_config(&frames.read(), padding(), speed()))
    });

    // Save to localStorage when config changes
    use_effect(move || {
        if let Some(config) = save_config() {
            if let Some(storage) = &storage {
                let _ = storage.set_item("badge_designer_state", &config);
            }
        }
    });

    rsx! {
        div {
            class: "flex flex-col gap-4",
            tabindex: "0",
            onkeydown: move |e| {
                match e.key() {
                    Key::ArrowUp => {
                        if focused_y() > 0 {
                            *focused_y.write() -= 1;
                        }
                    }
                    Key::ArrowDown => {
                        if focused_y() < 10 {
                            *focused_y.write() += 1;
                        }
                    }
                    Key::ArrowLeft => {
                        if focused_x() > 0 {
                            *focused_x.write() -= 1;
                        }
                    }
                    Key::ArrowRight => {
                        if focused_x() < 43 {
                            *focused_x.write() += 1;
                        }
                    }
                    Key::Character(c) if c == " " => {
                        let fi = focused_frame();
                        let fy = focused_y();
                        let fx = focused_x();
                        let mut frame = frames.read()[fi];
                        let current = frame.read()[fy][fx];
                        frame.write()[fy][fx] = !current;
                    }
                    _ => {}
                }
            },
            label {
                title: "Number of blank columns between frames. The original badge firmware uses a padding of 4.",
                "Padding between frames: ",
                input {
                    r#type: "number",
                    value: "{padding}",
                    oninput: move |e| {
                        if let Ok(value) = e.value().parse::<u8>() {
                            *padding.write() = value;
                        }
                    }
                }
            },
            label {
                "Speed: ",
                input {
                    r#type: "number",
                    value: "{speed}",
                    oninput: move |e| {
                        if let Ok(value) = e.value().parse::<u8>() {
                            if value >= 1 && value <= 7 {
                                *speed.write() = value;
                            }
                        }
                    }
                }
            },
            for (frame_index, frame) in frames.read().iter().copied().enumerate() {
                FrameEditor {
                    key: "{frame_index}",
                    frame_index,
                    frame,
                    is_focused: focused_frame() == frame_index,
                    focused_x: focused_x(),
                    focused_y: focused_y(),
                    on_focus: move |(x, y)| {
                        *focused_frame.write() = frame_index;
                        *focused_x.write() = x;
                        *focused_y.write() = y;
                    },
                    on_remove: move |()| {
                        frames.write().remove(frame_index);
                    },
                    on_clone: move |()| {
                        let f = *frame.read();
                        frames.write().insert(frame_index + 1, Signal::new(f));
                    }
                }
            },
            div {
                class: "flex gap-4 w-full",
                button {
                    class: "p-2 bg-blue-500 text-white btn",
                    onclick: move |_| {
                        let last_frame = frames.read().last().map(|f| *f.read()).unwrap_or([[false; 44]; 11]);
                        frames.write().push(Signal::new(last_frame));
                    },
                    "Add Frame"
                },
                button {
                    class: "p-2 bg-purple-500 text-white btn",
                    onclick: move |_| {
                        let current: Vec<FrameData> = frames.read().iter().map(|f| *f.read()).collect();
                        let reversed: Vec<Signal<FrameData>> = current.iter().rev().map(|f| Signal::new(*f)).collect();
                        frames.write().extend(reversed);
                    },
                    "Make Cycle"
                }
            },
            button {
                onclick: move |_| async move {
                    let file = create_config(&frames.read(), padding(), speed());
                    let js = format!(
                        r#"
                        const blob = new Blob([`{}`], {{ type: 'text/plain' }});
                        const url = URL.createObjectURL(blob);
                        const a = document.createElement('a');
                        a.href = url;
                        a.download = 'badge.toml';
                        a.click();
                        URL.revokeObjectURL(url);
                        "#,
                        file
                    );
                    document::eval(&js);
                },
                "Export"
            }
            label {
                class: "p-2 bg-blue-500 text-white btn cursor-pointer text-center",
                "Import"
                input {
                    r#type: "file",
                    accept: ".toml",
                    class: "hidden",
                    onchange: move |e| async move {
                        let files = e.files();
                        if let Some(file) = files.first() {
                            if let Ok(contents) = file.read_string().await {
                                let (new_frames, new_padding, new_speed) = load_config(&contents);
                                if !new_frames.is_empty() {
                                    *frames.write() = new_frames.into_iter().map(Signal::new).collect();
                                    *padding.write() = new_padding;
                                    *speed.write() = new_speed;
                                }
                            }
                        }
                    },
                }
            }
        }
    }
}
