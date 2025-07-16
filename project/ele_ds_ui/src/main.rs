use eframe::egui;

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };
    eframe::run_native(
        "电子卓搭桌面程序",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))), // 修改点：添加 Ok() 包装
    ).expect("Failed to launch eframe application");
}

#[derive(Default)]
struct MyApp {
    name: String,
    age: u32,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
            ui.text_edit_singleline(&mut self.name);
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));

            if ui.button("Submit").clicked() {
                println!("Name: {}, Age: {}", self.name, self.age);
            }
        });
    }
}