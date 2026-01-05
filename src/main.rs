#![allow(clippy::needless_range_loop)]

use badge_designer_lib::BadgeDesigner;

#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
fn main() -> eframe::Result<()> {
    use eframe::egui;
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 700.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Badge Designer",
        options,
        Box::new(|_cc| Ok(Box::new(BadgeDesigner::new()))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use wasm_bindgen::JsCast;

    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    wasm_bindgen_futures::spawn_local(async {
        let canvas = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("the_canvas_id")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();

        eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|_cc| Ok(Box::new(BadgeDesigner::new()))),
            )
            .await
            .expect("failed to start eframe");
    });
}

#[cfg(target_os = "android")]
fn main() {
    // Android uses android_main in lib.rs
}
