use gpui::{App, actions};

actions!(set_menus, [Quit]);

pub fn quit(_: &Quit, cx: &mut App) {
    println!("Gracefully quitting the application . . .");
    cx.quit();
}
