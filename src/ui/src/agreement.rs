use eframe::egui;

use crate::window_manager::WindowConfig;
use crate::window_manager::WindowManager;
use crate::window_manager::GLOBAL_WINDOW;
use crate::window_manager::init_global_window;
/// 显示用户协议窗口（无参数调用）
pub fn show_agreement_window() {
    init_global_window();

    let agreement_content = |ui: &mut egui::Ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.label(
                "欢迎使用本软件。请仔细阅读以下条款：\n\n\
                1. 本软件按\"现状\"提供，不提供任何形式的明示或暗示担保。\n\
                2. 使用本软件的风险由您自行承担。\n\
                3. 我们保留随时修改服务条款的权利。\n\
                4. 您不得对本软件进行逆向工程或反编译。\n\
                5. 本协议受中国法律管辖。\n\n\
                点击\"同意\"表示您已阅读并接受所有条款。",
            );
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("不同意").clicked() {
                println!("用户不同意协议");
            }
            if ui.button("同意").clicked() {
                println!("用户同意协议");
            }
        });
    };

    let config = WindowConfig {
        title: "用户协议".to_string(),
        size: [600.0, 400.0],
        resizable: false,
        movable: true,
        collapsible: false,
    };

    let mut window = WindowManager::new(config, agreement_content);
    window.set_visible(true);

    if let Some(global_window) = GLOBAL_WINDOW.get() {
        *global_window.lock().unwrap() = Some(window);
    }
}
