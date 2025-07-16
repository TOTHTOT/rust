use super::AppState;
use eframe::egui;

// 渲染仪表盘
pub fn render_dashboard(app: &AppState, ui: &mut egui::Ui) {
    ui.heading("Dashboard");

    // 修正：这里应该是显示用户信息，而不是一个空的text_edit
    if let Some(user) = &app.user {
        ui.label(format!("Welcome, {}", user.username));
    }

    ui.label("Welcome to the dashboard!");
    ui.label("This is a protected page that requires authentication.");
}