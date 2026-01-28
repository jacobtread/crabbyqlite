use gpui::App;

pub mod close_database;
pub mod new_database;
pub mod new_memory_database;
pub mod open_file;
pub mod quit;

pub fn register_actions(cx: &mut App) {
    cx.on_action(quit::quit);
    cx.on_action(new_database::new_database);
    cx.on_action(new_memory_database::new_memory_database);
    cx.on_action(open_file::open_file);
    cx.on_action(close_database::close_database);
}
