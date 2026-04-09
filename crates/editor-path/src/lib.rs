#![doc = r#"Filename-aware path matching shared across syntax, LSP, and hook dispatch."#]

use std::path::Path;

const EXTENSION_SCORE_BASE: usize = 1_000;
const GLOB_SCORE_BASE: usize = 2_000;
const FILE_NAME_SCORE_BASE: usize = 3_000;

/// Human-readable summary of this crate's responsibility.
pub const ROLE: &str = "Filename-aware path matching shared across syntax, LSP, and hook dispatch.";

/// Returns the responsibility summary for this crate.
pub const fn role() -> &'static str {
    ROLE
}

/// One supported filename matching strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PathPatternKind {
    /// Matches a file extension without a leading dot.
    Extension,
    /// Matches an exact basename or filename.
    FileName,
    /// Matches a simple glob against the basename.
    Glob,
}

/// A single filename-aware matcher.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathPattern {
    Extension(String),
    FileName(String),
    Glob(String),
}

impl PathPattern {
    /// Creates an extension matcher.
    pub fn extension(extension: impl AsRef<str>) -> Option<Self> {
        let extension = normalize_extension(extension.as_ref());
        if extension.is_empty() {
            None
        } else {
            Some(Self::Extension(extension))
        }
    }

    /// Creates an exact filename matcher.
    pub fn file_name(file_name: impl AsRef<str>) -> Option<Self> {
        let file_name = normalize_text(file_name.as_ref())?;
        Some(Self::FileName(file_name))
    }

    /// Creates a glob matcher.
    pub fn glob(pattern: impl AsRef<str>) -> Option<Self> {
        let pattern = normalize_text(pattern.as_ref())?;
        Some(Self::Glob(pattern))
    }

    /// Parses a hook/detail filter using Volt's legacy extension convention.
    pub fn from_filter(filter: impl AsRef<str>) -> Option<Self> {
        let filter = filter.as_ref().trim();
        if filter.is_empty() {
            return None;
        }
        if contains_wildcards(filter) {
            return Self::glob(filter);
        }
        if filter.starts_with('.') {
            return Self::extension(filter);
        }
        Self::file_name(filter)
    }

    /// Returns the matcher kind.
    pub const fn kind(&self) -> PathPatternKind {
        match self {
            Self::Extension(_) => PathPatternKind::Extension,
            Self::FileName(_) => PathPatternKind::FileName,
            Self::Glob(_) => PathPatternKind::Glob,
        }
    }

    /// Returns whether the matcher applies to the provided path.
    pub fn matches_path(&self, path: &Path) -> bool {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| self.matches_file_name(name))
    }

    /// Returns whether the matcher applies to a basename-like value.
    pub fn matches_file_name(&self, file_name: &str) -> bool {
        match self {
            Self::Extension(extension) => Path::new(file_name)
                .extension()
                .and_then(|value| value.to_str())
                .map(|value| value.eq_ignore_ascii_case(extension))
                .unwrap_or(false),
            Self::FileName(expected) => file_name == expected,
            Self::Glob(pattern) => glob_matches(pattern, file_name),
        }
    }

    /// Returns a score that ranks more specific matches above broader ones.
    pub fn match_score_for_path(&self, path: &Path) -> Option<usize> {
        self.matches_path(path).then_some(self.score())
    }

    /// Returns a score that ranks more specific matches above broader ones.
    pub fn match_score_for_file_name(&self, file_name: &str) -> Option<usize> {
        self.matches_file_name(file_name).then_some(self.score())
    }

    fn score(&self) -> usize {
        match self {
            Self::Extension(extension) => EXTENSION_SCORE_BASE + extension.len(),
            Self::FileName(file_name) => FILE_NAME_SCORE_BASE + file_name.len(),
            Self::Glob(pattern) => GLOB_SCORE_BASE + glob_literal_count(pattern),
        }
    }
}

/// A reusable collection of path patterns.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PathMatcher {
    patterns: Vec<PathPattern>,
}

impl PathMatcher {
    /// Builds a matcher from extension, filename, and glob lists.
    pub fn from_parts<E, F, G, ES, FS, GS>(extensions: E, file_names: F, file_globs: G) -> Self
    where
        E: IntoIterator<Item = ES>,
        F: IntoIterator<Item = FS>,
        G: IntoIterator<Item = GS>,
        ES: AsRef<str>,
        FS: AsRef<str>,
        GS: AsRef<str>,
    {
        let mut patterns = Vec::new();
        patterns.extend(
            extensions
                .into_iter()
                .filter_map(|extension| PathPattern::extension(extension)),
        );
        patterns.extend(
            file_names
                .into_iter()
                .filter_map(|file_name| PathPattern::file_name(file_name)),
        );
        patterns.extend(
            file_globs
                .into_iter()
                .filter_map(|file_glob| PathPattern::glob(file_glob)),
        );
        Self { patterns }
    }

    /// Returns whether any pattern matches the path.
    pub fn matches_path(&self, path: &Path) -> bool {
        self.best_match_score(path).is_some()
    }

    /// Returns the best match score for the path, if any pattern matches.
    pub fn best_match_score(&self, path: &Path) -> Option<usize> {
        self.patterns
            .iter()
            .filter_map(|pattern| pattern.match_score_for_path(path))
            .max()
    }
}

/// Normalizes an extension by trimming whitespace, removing leading dots, and lowercasing.
pub fn normalize_extension(extension: &str) -> String {
    extension
        .trim()
        .trim_start_matches('.')
        .to_ascii_lowercase()
}

fn normalize_text(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

fn contains_wildcards(pattern: &str) -> bool {
    pattern.contains('*') || pattern.contains('?')
}

fn glob_literal_count(pattern: &str) -> usize {
    pattern
        .chars()
        .filter(|character| !matches!(character, '*' | '?'))
        .count()
}

fn glob_matches(pattern: &str, candidate: &str) -> bool {
    let pattern = pattern.chars().collect::<Vec<_>>();
    let candidate = candidate.chars().collect::<Vec<_>>();
    let mut pattern_index = 0;
    let mut candidate_index = 0;
    let mut star_index = None;
    let mut candidate_backtrack = 0;

    while candidate_index < candidate.len() {
        if pattern_index < pattern.len()
            && (pattern[pattern_index] == '?'
                || pattern[pattern_index] == candidate[candidate_index])
        {
            pattern_index += 1;
            candidate_index += 1;
            continue;
        }

        if pattern_index < pattern.len() && pattern[pattern_index] == '*' {
            star_index = Some(pattern_index);
            pattern_index += 1;
            candidate_backtrack = candidate_index;
            continue;
        }

        let Some(star_index) = star_index else {
            return false;
        };
        pattern_index = star_index + 1;
        candidate_backtrack += 1;
        candidate_index = candidate_backtrack;
    }

    while pattern_index < pattern.len() && pattern[pattern_index] == '*' {
        pattern_index += 1;
    }

    pattern_index == pattern.len()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{PathMatcher, PathPattern, normalize_extension};

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

        let extension_score = matcher.best_match_score(Path::new("src\\main.rs"));
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
}
