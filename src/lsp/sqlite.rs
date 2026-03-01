use crate::{database::sqlite::SqliteDatabase, lsp::SqlLsp};
use anyhow::Context;
use gpui::{Task, Window};
use gpui_component::input::{CompletionProvider, InputState};
use lsp_types::{CompletionItem, CompletionItemKind, CompletionResponse};
use std::rc::Rc;
use std::sync::Mutex;

#[allow(unused)]
pub struct SqliteLsp {
    database: Rc<SqliteDatabase>,
    parser: Rc<Mutex<tree_sitter::Parser>>,
}

impl SqliteLsp {
    pub fn new(database: Rc<SqliteDatabase>) -> anyhow::Result<Rc<Self>> {
        let mut parser = tree_sitter::Parser::new();
        let language = tree_sitter_sequel::LANGUAGE;
        parser
            .set_language(&language.into())
            .context("failed to load sql parser")?;

        Ok(Rc::new(Self {
            database,
            parser: Rc::new(Mutex::new(parser)),
        }))
    }
}

impl SqlLsp for SqliteLsp {
    fn into_completion_provider(self: Rc<Self>) -> Rc<dyn CompletionProvider + 'static> {
        self
    }
}

impl CompletionProvider for SqliteLsp {
    fn completions(
        &self,
        _rope: &gpui_component::Rope,
        _offset: usize,
        trigger: lsp_types::CompletionContext,
        _window: &mut Window,
        _cx: &mut gpui::Context<InputState>,
    ) -> gpui::Task<gpui::Result<CompletionResponse>> {
        let trigger_character = trigger.trigger_character.unwrap_or_default();
        if trigger_character.is_empty() {
            return Task::ready(Ok(CompletionResponse::Array(vec![])));
        }

        let mut items: Vec<CompletionItem> = vec![];

        for keyword in KEYWORDS {
            // Exclude keywords that have already been completely typed
            // out or are different from the current words
            if !keyword.starts_with(&trigger_character) || trigger_character.eq(*keyword) {
                continue;
            }

            items.push(CompletionItem {
                label: keyword.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some(format!("SQLite keyword: {}", keyword)),
                sort_text: Some(format!("0{}", keyword)),
                ..Default::default()
            });
        }

        Task::ready(Ok(CompletionResponse::Array(items)))
    }

    fn is_completion_trigger(
        &self,
        _offset: usize,
        _new_text: &str,
        _cx: &mut gpui::Context<InputState>,
    ) -> bool {
        true
    }
}

static KEYWORDS: &[&str] = &[
    "ABORT",
    "ACTION",
    "ADD",
    "AFTER",
    "ALL",
    "ALTER",
    "ALWAYS",
    "ANALYZE",
    "AND",
    "AS",
    "ASC",
    "ATTACH",
    "AUTOINCREMENT",
    "BEFORE",
    "BEGIN",
    "BETWEEN",
    "BY",
    "CASCADE",
    "CASE",
    "CAST",
    "CHECK",
    "COLLATE",
    "COLUMN",
    "COMMIT",
    "CONFLICT",
    "CONSTRAINT",
    "CREATE",
    "CROSS",
    "CURRENT",
    "CURRENT_DATE",
    "CURRENT_TIME",
    "CURRENT_TIMESTAMP",
    "DATABASE",
    "DEFAULT",
    "DEFERRABLE",
    "DEFERRED",
    "DELETE",
    "DESC",
    "DETACH",
    "DISTINCT",
    "DO",
    "DROP",
    "EACH",
    "ELSE",
    "END",
    "ESCAPE",
    "EXCEPT",
    "EXCLUDE",
    "EXCLUSIVE",
    "EXISTS",
    "EXPLAIN",
    "FAIL",
    "FILTER",
    "FIRST",
    "FOLLOWING",
    "FOR",
    "FOREIGN",
    "FROM",
    "FULL",
    "GENERATED",
    "GLOB",
    "GROUP",
    "GROUPS",
    "HAVING",
    "IF",
    "IGNORE",
    "IMMEDIATE",
    "IN",
    "INDEX",
    "INDEXED",
    "INITIALLY",
    "INNER",
    "INSERT",
    "INSTEAD",
    "INTERSECT",
    "INTO",
    "IS",
    "ISNULL",
    "JOIN",
    "KEY",
    "LAST",
    "LEFT",
    "LIKE",
    "LIMIT",
    "MATCH",
    "MATERIALIZED",
    "NATURAL",
    "NO",
    "NOT",
    "NOTHING",
    "NOTNULL",
    "NULL",
    "NULLS",
    "OF",
    "OFFSET",
    "ON",
    "OR",
    "ORDER",
    "OTHERS",
    "OUTER",
    "OVER",
    "PARTITION",
    "PLAN",
    "PRAGMA",
    "PRECEDING",
    "PRIMARY",
    "QUERY",
    "RAISE",
    "RANGE",
    "RECURSIVE",
    "REFERENCES",
    "REGEXP",
    "REINDEX",
    "RELEASE",
    "RENAME",
    "REPLACE",
    "RESTRICT",
    "RETURNING",
    "RIGHT",
    "ROLLBACK",
    "ROW",
    "ROWS",
    "SAVEPOINT",
    "SELECT",
    "SET",
    "TABLE",
    "TEMP",
    "TEMPORARY",
    "THEN",
    "TIES",
    "TO",
    "TRANSACTION",
    "TRIGGER",
    "UNBOUNDED",
    "UNION",
    "UNIQUE",
    "UPDATE",
    "USING",
    "VACUUM",
    "VALUES",
    "VIEW",
    "VIRTUAL",
    "WHEN",
    "WHERE",
    "WINDOW",
    "WITH",
    "WITHOUT",
];
