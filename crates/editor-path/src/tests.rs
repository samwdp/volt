
use std::path::Path;

use super::{PathMatcher, PathPattern, grammar_install_root, normalize_extension, volt_data_dir};

#[test]
fn filter_parsing_preserves_extension_filename_and_glob_forms() {
    assert_eq!(
        PathPattern::from_filter(".rs"),
        Some(PathPattern::Extension("rs".to_owned()))
    );
    assert_eq!(
        PathPattern::from_filter("Makefile"),
        Some(PathPattern::FileName("Makefile".to_owned()))
    );
    assert_eq!(
        PathPattern::from_filter("Dockerfile.*"),
        Some(PathPattern::Glob("Dockerfile.*".to_owned()))
    );
}

#[test]
fn matcher_scores_filename_glob_and_extension_paths() {
    let matcher = PathMatcher::from_parts(["rs"], ["Makefile"], ["Dockerfile.*"]);
    let rust_main = Path::new("src").join("main.rs");

    let extension_score = matcher.best_match_score(&rust_main);
    let file_name_score = matcher.best_match_score(Path::new("Makefile"));
    let glob_score = matcher.best_match_score(Path::new("Dockerfile.dev"));

    assert!(extension_score.is_some());
    assert!(glob_score.is_some());
    assert!(file_name_score.is_some());
    assert!(file_name_score > glob_score);
    assert!(glob_score > extension_score);
}

#[test]
fn normalize_extension_strips_dots_and_lowercases() {
    assert_eq!(normalize_extension(".RS"), "rs");
}

#[test]
fn grammar_install_root_is_under_volt_data_dir_without_override() {
    // SAFETY: test process may already have VOLT_GRAMMAR_DIR; we only assert default shape
    // when the override is unset.
    if std::env::var_os("VOLT_GRAMMAR_DIR").is_none() {
        assert_eq!(grammar_install_root(), volt_data_dir().join("grammars"));
    }
}
