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
        Editor {}

    }
}

#[component]
pub fn Hero() -> Element {
    rsx! {
        div {
            id: "hero",
            img { src: HEADER_SVG, id: "header" }
            div { id: "links",
                a { href: "https://dioxuslabs.com/learn/0.7/", "📚 Learn Dioxus" }
                a { href: "https://dioxuslabs.com/awesome", "🚀 Awesome Dioxus" }
                a { href: "https://github.com/dioxus-community/", "📡 Community Libraries" }
                a { href: "https://github.com/DioxusLabs/sdk", "⚙️ Dioxus Development Kit" }
                a { href: "https://marketplace.visualstudio.com/items?itemName=DioxusLabs.dioxus", "💫 VSCode Extension" }
                a { href: "https://discord.gg/XgGxMSkvUM", "👋 Community Discord" }
            }
        }
    }
}

fn create_config(frames: Vec<[[bool; 44]; 11]>, padding: u8, speed: u8) -> String {
    let mut bitstring = String::new();
    for y in 0..11 {
        for frame in &frames {
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
bitstring = """
{bitstring}""""#
    )
}

fn load_config(config: &str, old_padding: u8) -> (Vec<[[bool; 44]; 11]>, u8, u8) {
    let mut frames = Vec::new();
    let mut speed = 5;
    let mut padding = old_padding;
    let mut in_bitstring = false;
    let mut current_frame: Vec<[[bool; 44]; 11]> = vec![];
    let mut current_y = 0;
    for line in config.lines() {
        if line.starts_with("speed =") {
            if let Some(s) = line.split('=').nth(1) {
                speed = s.trim().parse().unwrap_or(5);
            }
        } else if line.starts_with("padding =") {
            if let Some(s) = line.split('=').nth(1) {
                padding = s.trim().parse().unwrap_or(old_padding);
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
pub fn Editor() -> Element {
    let mut frames = use_signal(|| vec![[[false; 44]; 11]]);
    let mut adding = use_signal(|| true);
    let mut active = use_signal(|| false);
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
                        let (new_frames, new_padding, new_speed) = load_config(&config, 0);
                        if !new_frames.is_empty() {
                            *frames.write() = new_frames;
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
        Some(create_config(frames(), padding(), speed()))
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
                        frames.write()[fi][fy][fx] = !frames()[fi][fy][fx];
                    }
                    _ => {}
                }
            },
            label {
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
            for frame_index in 0..frames.read().len() {
                div {
                    class: "flex",
                    div{
                        for y in 0..11 {
                            div {
                                class: "flex",
                                for x in 0..44 {
                                    button {
                                        class: "w-4 h-4 border",
                                        class: if focused_frame() == frame_index && focused_x() == x && focused_y() == y { "border-green-500 border-2" } else { "border-gray-300" },
                                        class: if frames.read()[frame_index][y][x] { " bg-white" } else { "bg-black" },
                                        onmousedown: move |_| {
                                            *focused_frame.write() = frame_index;
                                            *focused_x.write() = x;
                                            *focused_y.write() = y;
                                            *adding.write() = !frames()[frame_index][y][x];
                                            *active.write() = true;
                                            frames.write()[frame_index][y][x] = adding();
                                        },
                                        onmouseenter: move |_| { if active() { frames.write()[frame_index][y][x] = adding() }},
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
                            onclick: {
                                move |_| {
                                    for y in 0..11 {
                                        for x in 0..44{
                                            frames.write()[frame_index][y][x] = !frames()[frame_index][y][x];
                                        }
                                    }
                                }
                            },
                            "invert"
                        }
                        button {
                            class: "p-2 bg-blue-500 text-white btn",
                            onclick: {
                                move |_| {
                                    for y in 0..11 {
                                        for x in 0..44{
                                            frames.write()[frame_index][y][x] = false;
                                        }
                                    }
                                }
                            },
                            "clear"
                        }
                        button {
                            class: "p-2 bg-red-500 text-white btn",
                            onclick: {
                                move |_| {
                                    frames.remove(frame_index);
                                }
                            },
                            "X"
                        }
                    }
                }

            },
            button {
                class: "p-2 bg-blue-500 text-white btn",
                onclick: move |_| {
                    let last_frame = frames.read().last().cloned().unwrap_or([[false; 44]; 11]);
                    frames.write().push(last_frame);
                },
                "Add Frame"
            },
            button {
                class: "p-2 bg-purple-500 text-white btn",
                onclick: move |_| {
                    let current = frames();
                    let mut reversed: Vec<_> = current.iter().rev().cloned().collect();
                    frames.write().append(&mut reversed);
                },
                "Make Cycle"
            },
            button {
                onclick: move |_| async move {
                    let file = create_config(frames(), padding(), speed());
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
            input {
                r#type: "file",
                accept: ".toml",
                onchange: move |e| async move {
                    let files = e.files();
                    if let Some(file) = files.first() {
                        if let Ok(contents) = file.read_string().await {
                            let (new_frames, new_padding, new_speed) = load_config(&contents, padding());
                            if !new_frames.is_empty() {
                                *frames.write() = new_frames;
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
