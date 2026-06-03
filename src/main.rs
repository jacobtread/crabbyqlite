#![cfg_attr(
    all(target_os = "windows", not(debug_assertions),),
    windows_subsystem = "windows"
)]

use gpui::*;
use gpui_component::{Root, Theme, ThemeMode, TitleBar};
use gpui_component_assets::Assets;

use crate::{
    assets::{CombinedAssetSource, CustomAssets},
    keybindings::init_keybindings,
    lsp::init_sql_language,
    state::AppState,
    ui::{actions::register_actions, app::MainApp, menus::register_app_menus},
    utils::gpui_tokio::init_tokio,
};

mod assets;
mod database;
mod keybindings;
mod logging;
mod lsp;
mod state;
mod ui;
mod utils;

rust_i18n::i18n!("locales");

fn init(cx: &mut App) {
    gpui_component::init(cx);
    init_theme(cx);
    init_keybindings(cx);
    init_tokio(cx);
}

fn init_theme(cx: &mut App) {
    // Set the theme to dark
    Theme::change(ThemeMode::Dark, None, cx);

    // Move the notifications to the bottom right
    let theme = Theme::global_mut(cx);
    theme.notification.placement = Anchor::BottomRight;
}

fn main() {
    logging::init_logging();

    init_sql_language();

    let app = gpui_platform::application()
        //
        .with_assets(CombinedAssetSource {
            assets: Assets,
            custom_assets: CustomAssets,
        });

    app.run(move |cx| {
        init(cx);

        // Setup global state
        let app_state = AppState::new(cx);
        cx.set_global(app_state);

        // Bring the menu bar to the foreground (so you can see the menu bar)
        cx.activate(true);

        // Register actions and menus
        register_actions(cx);
        register_app_menus(cx);

        let bounds = Bounds::centered(None, size(px(800.0), px(600.0)), cx);

        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions {
                    titlebar: Some(TitleBar::title_bar_options()),
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |window, cx| {
                    let view = MainApp::new(window, cx);
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
