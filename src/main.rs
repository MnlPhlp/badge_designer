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

#[component]
pub fn Editor() -> Element {
    let mut frames = use_signal(|| vec![[[false; 44]; 11]]);
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
                                    div {
                                        class: "w-4 h-4 border border-gray-300",
                                    }
                                }

                            }
                        }
                    },
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
            button {
                class: "p-2 bg-blue-500 text-white btn",
                onclick: move |_| {
                    frames.push([[false; 44]; 11]);
                },
                "Add Frame"
            }

        }
    }
}
