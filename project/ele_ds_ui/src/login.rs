use super::{AppState, Page, User};
use eframe::egui;

// 渲染登录页面
pub fn render_login_page(app: &mut AppState, ui: &mut egui::Ui) {
    ui.heading("Login");

    ui.text_edit_singleline(&mut app.login_username);
    ui.add(egui::TextEdit::singleline(&mut app.login_password).password(true));

    if ui.button("Login").clicked() {
        // 模拟登录验证
        if !app.login_username.is_empty() && !app.login_password.is_empty() {
            app.user = Some(User { username: app.login_username.clone() });
            app.current_page = Page::Dashboard;

            // 登录成功后清空表单
            app.login_username.clear();
            app.login_password.clear();
            app.login_error = None;
        } else {
            app.login_error = Some("Invalid credentials".to_string());
        }
    }

    // 显示错误信息
    if let Some(error) = &app.login_error {
        ui.colored_label(egui::Color32::RED, error);
    }
}