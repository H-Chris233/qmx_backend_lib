use eframe::egui;
use once_cell::sync::OnceCell;
use std::sync::{Arc, Mutex};

// 全局窗口状态容器：确保 Window Manager 能在线程间安全共享
pub static GLOBAL_WINDOW: OnceCell<Arc<Mutex<Option<WindowManager>>>> = OnceCell::new();
static INITIALIZED: OnceCell<()> = OnceCell::new();

// 初始化全局窗口容器
pub fn init_global_window() {
    GLOBAL_WINDOW.get_or_init(|| Arc::new(Mutex::new(None)));
}

/// 通用窗口配置
#[derive(Clone)]
pub struct WindowConfig {
    pub title: String,
    pub size: [f32; 2],
    pub resizable: bool,
    pub movable: bool,
    pub collapsible: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "窗口".to_string(),
            size: [600.0, 400.0],
            resizable: false,
            movable: true,
            collapsible: false,
        }
    }
}

/// 通用窗口管理器：支持线程安全的闭包（Send + Sync）
pub struct WindowManager {
    config: WindowConfig,
    visible: bool,
    content: Box<dyn FnMut(&mut egui::Ui) + Send + Sync + 'static>,
}

impl WindowManager {
    /// 创建新窗口：要求闭包必须线程安全
    pub fn new<F>(config: WindowConfig, content: F) -> Self
    where
        F: FnMut(&mut egui::Ui) + Send + Sync + 'static,
    {
        Self {
            config,
            visible: true,
            content: Box::new(content),
        }
    }

    /// 显示窗口内容：使用 default_size 替代 fixed_size
    pub fn show(&mut self, ctx: &egui::Context) {
        // 自动设置字体（只执行一次）
        setup_fonts(ctx);

        if self.visible {
            egui::Window::new(&self.config.title)
                .resizable(self.config.resizable)
                .movable(self.config.movable)
                .collapsible(self.config.collapsible)
                .default_size(egui::vec2(self.config.size[0], self.config.size[1]))
                .show(ctx, |ui| {
                    (self.content)(ui);
                });
        }
    }

    /// 控制窗口可见性
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// 检查窗口可见状态
    pub fn is_visible(&self) -> bool {
        self.visible
    }
}

pub fn setup_fonts(ctx: &egui::Context) {
    if INITIALIZED.get().is_none() {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "noto_sans".to_string(),
            egui::FontData::from_static(include_bytes!("../resources/NotoSerifCJKsc-SemiBold.otf"))
                .into(),
        );

        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "noto_sans".to_string());

        ctx.set_fonts(fonts);
        INITIALIZED.set(()).unwrap();
    }
}
