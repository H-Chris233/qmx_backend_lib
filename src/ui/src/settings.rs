use crate::window_manager::{WindowConfig, WindowManager};

pub fn show_settings_window(ctx: &egui::Context) {
    let settings_content = |ui: &mut egui::Ui| {
        ui.heading("设置");
        ui.checkbox(&mut true, "启用自动保存");
        ui.add(egui::Slider::new(&mut 50, 0..=100).text("音量"));
    };

    let config = WindowConfig {
        title: "设置".to_string(),
        size: [400.0, 300.0],
        resizable: false,
        movable: true,
        collapsible: true,
    };

    let mut window = WindowManager::new(config, settings_content);
    window.show(ctx);
}
