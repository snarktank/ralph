//! Dependency inference from target files

#![allow(dead_code)]

use crate::parallel::dependency::StoryNode;
use glob::Pattern;

/// Infers dependencies between stories based on overlapping targetFiles patterns.
///
/// When two stories have overlapping targetFiles patterns, the story with higher
/// priority (lower priority number) becomes a dependency of the story with lower
/// priority (higher priority number). This ensures that higher-priority stories
/// execute first when they might modify the same files.
///
/// # Arguments
///
/// * `stories` - A slice of `StoryNode` references to analyze
///
/// # Returns
///
/// A vector of tuples `(dependent_id, dependency_id)` where `dependent_id` depends
/// on `dependency_id` due to overlapping target files.
pub fn infer_from_files(stories: &[StoryNode]) -> Vec<(String, String)> {
    let mut dependencies = Vec::new();

    // Compare each pair of stories
    for i in 0..stories.len() {
        for j in (i + 1)..stories.len() {
            let story_a = &stories[i];
            let story_b = &stories[j];

            // Check if target files overlap
            if patterns_overlap(&story_a.target_files, &story_b.target_files) {
                // Higher priority (lower number) becomes dependency
                // Lower priority (higher number) becomes dependent
                if story_a.priority < story_b.priority {
                    // story_a has higher priority, story_b depends on story_a
                    dependencies.push((story_b.id.clone(), story_a.id.clone()));
                } else if story_b.priority < story_a.priority {
                    // story_b has higher priority, story_a depends on story_b
                    dependencies.push((story_a.id.clone(), story_b.id.clone()));
                }
                // If priorities are equal, no dependency is inferred
            }
        }
    }

    dependencies
}

/// Checks if two sets of glob patterns have any overlap.
///
/// Two pattern sets overlap if any pattern from set A matches any pattern from set B,
/// or vice versa. This handles cases like:
/// - `src/**/*.rs` and `src/lib.rs` (glob matches literal)
/// - `src/**/*.rs` and `src/main.rs` (glob matches literal)
/// - `src/**/*.rs` and `src/**/*.rs` (same pattern)
fn patterns_overlap(patterns_a: &[String], patterns_b: &[String]) -> bool {
    for pattern_a in patterns_a {
        for pattern_b in patterns_b {
            if patterns_match(pattern_a, pattern_b) {
                return true;
            }
        }
    }
    false
}

/// Checks if two patterns could potentially match the same files.
///
/// This is a conservative check that returns true if:
/// - The patterns are identical
/// - Pattern A (as a glob) matches pattern B (as a literal path)
/// - Pattern B (as a glob) matches pattern A (as a literal path)
/// - Both patterns share a common prefix that could lead to matching files
fn patterns_match(pattern_a: &str, pattern_b: &str) -> bool {
    // Identical patterns always match
    if pattern_a == pattern_b {
        return true;
    }

    // Try treating pattern_a as glob and pattern_b as literal
    if let Ok(glob_a) = Pattern::new(pattern_a) {
        if glob_a.matches(pattern_b) {
            return true;
        }
    }

    // Try treating pattern_b as glob and pattern_a as literal
    if let Ok(glob_b) = Pattern::new(pattern_b) {
        if glob_b.matches(pattern_a) {
            return true;
        }
    }

    // Check for overlapping glob patterns by examining common prefixes
    // For patterns like "src/**/*.rs" and "src/lib.rs", we need to check
    // if the literal path could match the glob
    if let (Ok(glob_a), Ok(glob_b)) = (Pattern::new(pattern_a), Pattern::new(pattern_b)) {
        // Check if patterns could match the same directory tree
        // by testing if they share a common non-glob prefix
        let prefix_a = get_literal_prefix(pattern_a);
        let prefix_b = get_literal_prefix(pattern_b);

        // If one prefix starts with the other (or they share a common ancestor),
        // the patterns might overlap
        if !prefix_a.is_empty()
            && !prefix_b.is_empty()
            && (prefix_a.starts_with(&prefix_b)
                || prefix_b.starts_with(&prefix_a)
                || prefix_a == prefix_b)
        {
            // Both are globs that share a prefix - check if they could match same files
            // This is conservative: if both could match files in same directory, assume overlap
            return glob_a.matches(pattern_b)
                || glob_b.matches(pattern_a)
                || globs_could_overlap(pattern_a, pattern_b);
        }
    }

    false
}

/// Gets the literal (non-glob) prefix of a pattern.
///
/// For example:
/// - `src/**/*.rs` -> `src/`
/// - `src/lib.rs` -> `src/lib.rs`
/// - `*.rs` -> `` (empty)
fn get_literal_prefix(pattern: &str) -> String {
    let glob_chars = ['*', '?', '[', ']'];
    let mut prefix = String::new();

    for ch in pattern.chars() {
        if glob_chars.contains(&ch) {
            break;
        }
        prefix.push(ch);
    }

    prefix
}

/// Checks if two glob patterns could potentially match the same files.
///
/// This is used when both patterns contain wildcards and we need to determine
/// if they could match overlapping sets of files.
fn globs_could_overlap(pattern_a: &str, pattern_b: &str) -> bool {
    let prefix_a = get_literal_prefix(pattern_a);
    let prefix_b = get_literal_prefix(pattern_b);

    // If prefixes are compatible (one contains the other or they're equal)
    // and both patterns have wildcards, they might overlap
    if prefix_a.starts_with(&prefix_b) || prefix_b.starts_with(&prefix_a) {
        // Check if the suffix patterns could match similar files
        // For simplicity, if they share a directory prefix and both have wildcards,
        // we assume they could overlap (conservative approach)
        let has_wildcard_a = pattern_a.contains('*') || pattern_a.contains('?');
        let has_wildcard_b = pattern_b.contains('*') || pattern_b.contains('?');

        if has_wildcard_a && has_wildcard_b {
            // Both have wildcards in a shared directory - assume overlap
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_story_node(id: &str, priority: u32, target_files: Vec<&str>) -> StoryNode {
        StoryNode {
            id: id.to_string(),
            priority,
            passes: false,
            depends_on: vec![],
            target_files: target_files.into_iter().map(String::from).collect(),
        }
    }

    #[test]
    fn test_infer_from_files_overlapping_glob_patterns() {
        // Story A (priority 1) targets src/**/*.rs
        // Story B (priority 2) targets src/lib.rs
        // Story C (priority 3) targets src/**/*.rs
        // Story B and C should depend on A (highest priority with overlap)
        // Story C should depend on B (B has higher priority than C)
        let stories = vec![
            make_story_node("US-001", 1, vec!["src/**/*.rs"]),
            make_story_node("US-002", 2, vec!["src/lib.rs"]),
            make_story_node("US-003", 3, vec!["src/**/*.rs"]),
        ];

        let deps = infer_from_files(&stories);

        // US-002 depends on US-001 (src/lib.rs overlaps with src/**/*.rs)
        assert!(
            deps.contains(&("US-002".to_string(), "US-001".to_string())),
            "US-002 should depend on US-001"
        );

        // US-003 depends on US-001 (same pattern)
        assert!(
            deps.contains(&("US-003".to_string(), "US-001".to_string())),
            "US-003 should depend on US-001"
        );

        // US-003 depends on US-002 (src/**/*.rs overlaps with src/lib.rs)
        assert!(
            deps.contains(&("US-003".to_string(), "US-002".to_string())),
            "US-003 should depend on US-002"
        );
    }

    #[test]
    fn test_infer_from_files_no_overlap() {
        // Stories target completely different files
        let stories = vec![
            make_story_node("US-001", 1, vec!["src/**/*.rs"]),
            make_story_node("US-002", 2, vec!["tests/**/*.rs"]),
            make_story_node("US-003", 3, vec!["docs/**/*.md"]),
        ];

        let deps = infer_from_files(&stories);
        assert!(
            deps.is_empty(),
            "No dependencies should be inferred for non-overlapping patterns"
        );
    }

    #[test]
    fn test_infer_from_files_identical_files() {
        // Two stories target the exact same file
        let stories = vec![
            make_story_node("US-001", 1, vec!["Cargo.toml"]),
            make_story_node("US-002", 2, vec!["Cargo.toml"]),
        ];

        let deps = infer_from_files(&stories);
        assert_eq!(deps.len(), 1);
        assert!(deps.contains(&("US-002".to_string(), "US-001".to_string())));
    }

    #[test]
    fn test_infer_from_files_empty_target_files() {
        // Stories with empty target files should not create dependencies
        let stories = vec![
            make_story_node("US-001", 1, vec![]),
            make_story_node("US-002", 2, vec![]),
        ];

        let deps = infer_from_files(&stories);
        assert!(deps.is_empty());
    }

    #[test]
    fn test_infer_from_files_equal_priority() {
        // Stories with equal priority should not create dependencies
        let stories = vec![
            make_story_node("US-001", 1, vec!["src/lib.rs"]),
            make_story_node("US-002", 1, vec!["src/lib.rs"]),
        ];

        let deps = infer_from_files(&stories);
        assert!(
            deps.is_empty(),
            "Equal priority stories should not create dependencies"
        );
    }

    #[test]
    fn test_patterns_match_identical() {
        assert!(patterns_match("src/lib.rs", "src/lib.rs"));
        assert!(patterns_match("src/**/*.rs", "src/**/*.rs"));
    }

    #[test]
    fn test_patterns_match_glob_literal() {
        assert!(patterns_match("src/**/*.rs", "src/lib.rs"));
        assert!(patterns_match("src/lib.rs", "src/**/*.rs"));
        assert!(patterns_match("*.rs", "main.rs"));
    }

    #[test]
    fn test_patterns_match_no_overlap() {
        assert!(!patterns_match("src/**/*.rs", "tests/**/*.rs"));
        assert!(!patterns_match("Cargo.toml", "README.md"));
    }

    #[test]
    fn test_get_literal_prefix() {
        assert_eq!(get_literal_prefix("src/**/*.rs"), "src/");
        assert_eq!(get_literal_prefix("src/lib.rs"), "src/lib.rs");
        assert_eq!(get_literal_prefix("*.rs"), "");
        assert_eq!(get_literal_prefix("tests/"), "tests/");
    }
}
