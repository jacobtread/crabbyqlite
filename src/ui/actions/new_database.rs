use gpui::{App, actions};

actions!(file, [NewDatabase]);

pub fn new_database(_: &NewDatabase, _cx: &mut App) {}
