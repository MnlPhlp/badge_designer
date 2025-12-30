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

fn create_config(frames: Vec<[[bool; 44]; 11]>) -> String {
    let mut bitstring = String::new();
    for y in 0..11 {
        for frame in &frames {
            for x in 0..44 {
                bitstring.push(if frame[y][x] { 'X' } else { '_' });
            }
        }
    }
    bitstring
}

#[component]
pub fn Editor() -> Element {
    let mut frames = use_signal(|| vec![[[false; 44]; 11]]);
    let mut adding = use_signal(|| true);
    let mut active = use_signal(|| false);
    rsx! {
        div {
            class: "flex flex-col gap-4",
            for frame_index in 0..frames.read().len() {
                div {
                    class: "flex",
                    div{
                        for y in 0..11 {
                            div {
                                class: "flex",
                                for x in 0..44 {
                                    button {
                                        class: "w-4 h-4 border border-gray-300",
                                        class: if frames.read()[frame_index][y][x] { " bg-white" } else { "bg-black" },
                                        onmousedown: move |_| {
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
                        class: "p-2 bg-red-500 text-white btn",
                        onclick: {
                            move |_| {
                                frames.remove(frame_index);
                            }
                        },
                        "X"
                    }
                }

            },
            button {
                class: "p-2 bg-blue-500 text-white btn",
                onclick: move |_| {
                    frames.push([[false; 44]; 11]);
                },
                "Add Frame"
            },
            button {
                onclick: move |_| {
                    let file = create_config(frames());
                    // trigger download

                },
                "Export"
            }

        }
    }
}
