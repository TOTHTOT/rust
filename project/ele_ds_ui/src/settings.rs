use super::AppState;
use eframe::egui;

// 渲染设置页面
pub fn render_settings(_app: &AppState, ui: &mut egui::Ui) {
    ui.heading("Settings");
    ui.label("Here you can change your settings.");

    // 示例：添加一个设置选项
    let mut dark_mode = false; // 实际应用中应从app状态获取
    ui.checkbox(&mut dark_mode, "Dark Mode");
}