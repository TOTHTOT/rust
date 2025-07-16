use eframe::egui;

// 应用状态
pub struct AppState {
    pub current_page: Page,
    pub user: Option<User>,
    pub login_username: String,
    pub login_password: String,
    pub login_error: Option<String>,
}

// 页面枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Login,
    Dashboard,
    Settings,
}

// 用户信息
#[derive(Debug, Clone)]
pub struct User {
    pub username: String,
    // 实际应用中应存储加密后的密码或令牌
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_page: Page::Login,
            user: None,
            login_username: String::new(),
            login_password: String::new(),
            login_error: None,
        }
    }
}