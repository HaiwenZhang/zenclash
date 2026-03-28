use std::sync::Arc;

use gpui::Application;
use gpui_component_assets::Assets;
use parking_lot::RwLock;
use zenclash_core::config::AppConfig;
use zenclash_core::core::{CoreManager, CoreManagerConfig};
use zenclash_ui::{app, shortcuts, window};

static TOKIO_RUNTIME: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();

pub fn get_tokio_runtime() -> &'static tokio::runtime::Runtime {
    TOKIO_RUNTIME
        .get_or_init(|| tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime"))
}

fn main() {
    tracing_subscriber::fmt::init();

    let rt = get_tokio_runtime();
    let _guard = rt.enter();

    let app = Application::new().with_assets(Assets);

    app.run(move |cx| {
        app::init(cx);
        window::setup_keybindings(cx);
        shortcuts::ShortcutManager::init(cx);

        let app_config = AppConfig::load().unwrap_or_default();
        let manager_config = CoreManagerConfig::from_app_config(&app_config);
        let core_manager = Arc::new(RwLock::new(CoreManager::new(manager_config)));
        let config = Arc::new(RwLock::new(app_config));

        app::create_main_window(core_manager, config, app::WindowConfig::default(), cx)
            .expect("Failed to create main window");

        cx.activate(true);
    });
}
