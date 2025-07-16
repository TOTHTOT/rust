slint::include_modules!();
fn main() -> Result<(), slint::PlatformError> {
    let main_window = MainWindow::new()?;
    main_window.run()
}

struct Student {
    name: String,
    age: u32,
}

impl Student {
    fn new(name: String, age: u32) -> Self {
        Student { name, age }
    }
}
