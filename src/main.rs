use gpui::*;
use gpui_component::{Root, StyledExt, TitleBar, highlighter::LanguageRegistry};
use gpui_component_assets::Assets;

use crate::{
    state::{AppState, DatabaseStore},
    ui::{
        actions::register_actions,
        assets::{CombinedAssetSource, CustomAssets},
        database::DatabaseView,
        menus::register_app_menus,
        sql::create_sql_language_config,
        titlebar::AppTitleBar,
    },
};

mod database;
mod logging;
mod state;
mod ui;
mod utils;

rust_i18n::i18n!("locales");

pub struct MainApp {
    app_title_bar: Entity<AppTitleBar>,
    database_view: Entity<DatabaseView>,
}

impl Render for MainApp {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .v_flex()
            .size_full()
            .child(self.app_title_bar.clone())
            .child(div().size_full().child(self.database_view.clone()))
    }
}

fn main() {
    logging::init_logging();

    LanguageRegistry::singleton().register("sql", &create_sql_language_config());

    let app = Application::new()
        //
        .with_assets(CombinedAssetSource {
            assets: Assets,
            custom_assets: CustomAssets,
        });

    app.run(move |cx| {
        // This must be called before using any GPUI Component features.
        gpui_component::init(cx);

        ui::gpui_tokio::init(cx);

        let database_store = cx.new(|_| DatabaseStore::default());

        cx.set_global(AppState { database_store });

        // Bring the menu bar to the foreground (so you can see the menu bar)
        cx.activate(true);

        // Register actions
        register_actions(cx);

        // Register menu items
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
                    let view = cx.new(|cx| MainApp {
                        app_title_bar: AppTitleBar::new(window, cx),
                        database_view: DatabaseView::new(window, cx),
                    });
                    // This first level on the window, should be a Root.
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
