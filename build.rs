use slint_build::CompilerConfiguration;

fn main() {
    slint_build::compile_with_config("ui/appwindow.slint",CompilerConfiguration::default().with_style("fluent".to_string())).unwrap();
}
