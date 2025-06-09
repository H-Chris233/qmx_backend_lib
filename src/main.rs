

use eframe::egui;

use ui::agreement::show_agreement_window;
use ui::window_manager::init_global_window;
use ui::window_manager::GLOBAL_WINDOW;



struct MyApplication {}

impl Default for MyApplication {
    fn default() -> Self {
        Self {}
    }
}

impl eframe::App for MyApplication {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 检查并显示全局窗口
        if let Some(global_window) = GLOBAL_WINDOW.get().cloned() {
            let mut guard = global_window.lock().unwrap();
            if let Some(window) = guard.as_mut() {
                window.show(ctx);
            }
        }
    }
}
fn main() {
    init_global_window();
    println!("hello");
    show_agreement_window();
    let options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "管理工具",
        options,
        Box::new(|_cc| Ok(Box::new(MyApplication::default()))),
    );
    
}
