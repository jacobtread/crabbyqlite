use std::rc::Rc;

use gpui_component::input::CompletionProvider;

use crate::database::sqlite::SqliteDatabase;
use crate::lsp::sqlite::SqliteLsp;

use crate::database::Database;

pub mod sqlite;

pub trait SqlLsp: CompletionProvider + 'static {
    fn into_completion_provider(self: Rc<Self>) -> Rc<dyn CompletionProvider + 'static>;
}

pub fn create_sql_lsp(database: Rc<dyn Database>) -> anyhow::Result<Rc<dyn SqlLsp>> {
    let database = database.as_any();

    if let Ok(database) = database.downcast::<SqliteDatabase>() {
        let lsp: Rc<dyn SqlLsp> = SqliteLsp::new(database)?;
        return Ok(lsp);
    }

    Err(anyhow::anyhow!("unknown database backend"))
}
