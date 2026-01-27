use gpui::{App, Menu, MenuItem, SystemMenuType};

use crate::ui::actions::quit::Quit;

pub fn register_app_menus(cx: &mut App) {
    cx.set_menus(vec![Menu {
        name: "Menu".into(),
        items: vec![
            MenuItem::os_submenu("Services", SystemMenuType::Services),
            MenuItem::separator(),
            MenuItem::action("Quit", Quit),
        ],
    }]);
}
