use badge_designer::run_ui;

fn main() -> Result<(), slint::PlatformError> {
    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
    }

    run_ui()
}
