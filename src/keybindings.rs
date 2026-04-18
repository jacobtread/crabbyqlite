use gpui::{App, KeyBinding};

use crate::ui::actions::open_file::OpenFile;

pub fn init_keybindings(cx: &mut App) {
    #[cfg(target_os = "macos")]
    cx.bind_keys([KeyBinding::new(
        "cmd-o",
        OpenFile { read_only: false },
        None,
    )]);

    #[cfg(not(target_os = "macos"))]
    cx.bind_keys([KeyBinding::new("ctrl-o", OpenFile::default(), None)]);
}
