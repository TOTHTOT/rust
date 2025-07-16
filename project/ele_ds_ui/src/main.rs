use eframe::egui;
use std::sync::{Arc, Mutex};

// ========== 模块定义 ==========
mod app_state;
mod login;
mod dashboard;
mod settings;

// 使用模块中的内容
use app_state::{AppState, Page, User};
use login::render_login_page;
use dashboard::render_dashboard;
use settings::render_settings;

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Multi-Page App",
        options,
        Box::new(|_cc| Ok(Box::new(AppState::default())))
    ).expect("Failed to launch app");
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 顶部导航栏（登录后显示）
        if self.user.is_some() {
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    if ui.button("Dashboard").clicked() {
                        self.current_page = Page::Dashboard;
                    }
                    if ui.button("Settings").clicked() {
                        self.current_page = Page::Settings;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Logout").clicked() {
                            self.user = None;
                            self.current_page = Page::Login;
                        }
                        if let Some(user) = &self.user {
                            ui.label(format!("Welcome, {}", user.username));
                        }
                    });
                });
            });
        }

        // 主内容区域
        egui::CentralPanel::default().show(ctx, |ui| {
            match &self.current_page {
                Page::Login => render_login_page(self, ui),
                Page::Dashboard => render_dashboard(self, ui),
                Page::Settings => render_settings(self, ui),
            }
        });
    }
}