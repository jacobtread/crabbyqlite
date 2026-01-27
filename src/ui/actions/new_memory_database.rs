use gpui::{App, actions};

actions!(file, [NewMemoryDatabase]);

pub fn new_memory_database(_: &NewMemoryDatabase, _cx: &mut App) {}
