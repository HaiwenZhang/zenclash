use std::sync::Arc;

use gpui::Application;
use gpui_component_assets::Assets;
use tokio::sync::RwLock;
use zenclash_core::{AppConfig, CoreManager};
use zenclash_ui::{app, window};

fn main() {
    tracing_subscriber::fmt::init();

    let app = gpui_platform::application().with_assets(Assets);

    app.run(move |cx| {
        app::init(cx);
        window::setup_keybindings(cx);

        let core_manager = Arc::new(RwLock::new(CoreManager));
        let config = Arc::new(RwLock::new(AppConfig {
            core: "mihomo".to_string(),
        }));

        app::create_main_window(core_manager, config, app::WindowConfig::default(), cx)
            .expect("Failed to create main window");

        cx.activate(true);
    });
}
