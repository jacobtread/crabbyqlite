use gpui_component::highlighter::LanguageConfig;

pub fn create_sql_language_config() -> LanguageConfig {
    LanguageConfig {
        name: "SQL".into(),
        language: tree_sitter_sequel::LANGUAGE.into(),
        injection_languages: vec![],
        highlights: tree_sitter_sequel::HIGHLIGHTS_QUERY.into(),
        injections: "".into(),
        locals: "".into(),
    }
}
