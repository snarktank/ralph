//! Blog post generation module for Ralph.
//!
//! This module generates blog posts from templates, typically used to document
//! completed features, releases, or significant development milestones.

#![allow(dead_code)]

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Context data for generating a blog post.
///
/// Contains all the dynamic content that will be substituted into the template.
#[derive(Debug, Clone, Default)]
pub struct BlogContext {
    /// Title of the blog post
    pub title: String,
    /// Date of the post (e.g., "2026-01-17")
    pub date: String,
    /// Author of the post
    pub author: String,
    /// Brief summary of the feature or release
    pub summary: String,
    /// Description of the problem being solved
    pub problem: String,
    /// Description of the solution implemented
    pub solution: String,
    /// Challenges faced during development
    pub challenges: String,
    /// Wins and achievements
    pub wins: String,
    /// Lessons learned
    pub lessons: String,
    /// What's planned next
    pub next_steps: String,
    /// Technical implementation details
    pub technical_details: String,
}

impl BlogContext {
    /// Creates a new BlogContext with the required fields.
    pub fn new(
        title: impl Into<String>,
        problem: impl Into<String>,
        solution: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            problem: problem.into(),
            solution: solution.into(),
            ..Default::default()
        }
    }

    /// Sets the date field.
    pub fn with_date(mut self, date: impl Into<String>) -> Self {
        self.date = date.into();
        self
    }

    /// Sets the author field.
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = author.into();
        self
    }

    /// Sets the summary field.
    pub fn with_summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = summary.into();
        self
    }

    /// Sets the challenges field.
    pub fn with_challenges(mut self, challenges: impl Into<String>) -> Self {
        self.challenges = challenges.into();
        self
    }

    /// Sets the wins field.
    pub fn with_wins(mut self, wins: impl Into<String>) -> Self {
        self.wins = wins.into();
        self
    }

    /// Sets the lessons field.
    pub fn with_lessons(mut self, lessons: impl Into<String>) -> Self {
        self.lessons = lessons.into();
        self
    }

    /// Sets the next_steps field.
    pub fn with_next_steps(mut self, next_steps: impl Into<String>) -> Self {
        self.next_steps = next_steps.into();
        self
    }

    /// Sets the technical_details field.
    pub fn with_technical_details(mut self, technical_details: impl Into<String>) -> Self {
        self.technical_details = technical_details.into();
        self
    }
}

/// Error type for blog generation operations.
#[derive(Debug)]
pub enum BlogGeneratorError {
    /// Template file not found
    TemplateNotFound(PathBuf),
    /// Failed to read template file
    TemplateReadError(io::Error),
    /// Failed to write output file
    WriteError(io::Error),
    /// Output directory does not exist
    OutputDirNotFound(PathBuf),
    /// Invalid file name
    InvalidFileName(String),
}

impl std::fmt::Display for BlogGeneratorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlogGeneratorError::TemplateNotFound(path) => {
                write!(f, "Template not found: {}", path.display())
            }
            BlogGeneratorError::TemplateReadError(e) => {
                write!(f, "Failed to read template: {}", e)
            }
            BlogGeneratorError::WriteError(e) => {
                write!(f, "Failed to write blog post: {}", e)
            }
            BlogGeneratorError::OutputDirNotFound(path) => {
                write!(f, "Output directory not found: {}", path.display())
            }
            BlogGeneratorError::InvalidFileName(name) => {
                write!(f, "Invalid file name: {}", name)
            }
        }
    }
}

impl std::error::Error for BlogGeneratorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BlogGeneratorError::TemplateReadError(e) => Some(e),
            BlogGeneratorError::WriteError(e) => Some(e),
            _ => None,
        }
    }
}

/// Result type for blog generator operations.
pub type BlogResult<T> = Result<T, BlogGeneratorError>;

/// Blog post generator that renders templates with context data.
pub struct BlogGenerator {
    /// Path to the template file
    template_path: PathBuf,
    /// Path to the output directory for blog posts
    output_dir: PathBuf,
}

impl BlogGenerator {
    /// Creates a new BlogGenerator with the specified template and output directory.
    pub fn new(template_path: impl Into<PathBuf>, output_dir: impl Into<PathBuf>) -> Self {
        Self {
            template_path: template_path.into(),
            output_dir: output_dir.into(),
        }
    }

    /// Creates a BlogGenerator with default paths relative to project root.
    ///
    /// Template: `docs/blog/templates/feature-release.md`
    /// Output: `docs/blog/posts/`
    pub fn with_defaults(project_root: impl AsRef<Path>) -> Self {
        let root = project_root.as_ref();
        Self {
            template_path: root.join("docs/blog/templates/feature-release.md"),
            output_dir: root.join("docs/blog/posts"),
        }
    }

    /// Generates a blog post by rendering the template with the provided context.
    ///
    /// Returns the rendered content as a String.
    pub fn generate(&self, context: &BlogContext) -> BlogResult<String> {
        // Read the template
        if !self.template_path.exists() {
            return Err(BlogGeneratorError::TemplateNotFound(
                self.template_path.clone(),
            ));
        }

        let template = fs::read_to_string(&self.template_path)
            .map_err(BlogGeneratorError::TemplateReadError)?;

        // Render the template by replacing placeholders
        let rendered = render_template(&template, context);

        Ok(rendered)
    }

    /// Saves a blog post to the output directory.
    ///
    /// The filename should not include the extension (`.md` will be added).
    /// Returns the path to the saved file.
    pub fn save(&self, content: &str, filename: &str) -> BlogResult<PathBuf> {
        // Validate output directory exists
        if !self.output_dir.exists() {
            return Err(BlogGeneratorError::OutputDirNotFound(
                self.output_dir.clone(),
            ));
        }

        // Validate filename
        if filename.is_empty() || filename.contains('/') || filename.contains('\\') {
            return Err(BlogGeneratorError::InvalidFileName(filename.to_string()));
        }

        // Create the full path
        let filename_with_ext = if filename.ends_with(".md") {
            filename.to_string()
        } else {
            format!("{}.md", filename)
        };
        let output_path = self.output_dir.join(&filename_with_ext);

        // Write the content
        fs::write(&output_path, content).map_err(BlogGeneratorError::WriteError)?;

        Ok(output_path)
    }

    /// Generates and saves a blog post in one operation.
    ///
    /// Returns the path to the saved file.
    pub fn generate_and_save(&self, context: &BlogContext, filename: &str) -> BlogResult<PathBuf> {
        let content = self.generate(context)?;
        self.save(&content, filename)
    }

    /// Returns the template path.
    pub fn template_path(&self) -> &Path {
        &self.template_path
    }

    /// Returns the output directory path.
    pub fn output_dir(&self) -> &Path {
        &self.output_dir
    }
}

/// Renders a template string by substituting `{{ placeholder }}` with context values.
fn render_template(template: &str, context: &BlogContext) -> String {
    template
        .replace("{{ title }}", &context.title)
        .replace("{{ date }}", &context.date)
        .replace("{{ author }}", &context.author)
        .replace("{{ summary }}", &context.summary)
        .replace("{{ problem }}", &context.problem)
        .replace("{{ solution }}", &context.solution)
        .replace("{{ challenges }}", &context.challenges)
        .replace("{{ wins }}", &context.wins)
        .replace("{{ lessons }}", &context.lessons)
        .replace("{{ next_steps }}", &context.next_steps)
        .replace("{{ technical_details }}", &context.technical_details)
}

/// Generates a slug from a title for use as a filename.
///
/// Converts to lowercase, replaces spaces and special characters with hyphens,
/// and removes consecutive hyphens.
pub fn slugify(title: &str) -> String {
    let slug: String = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();

    // Remove consecutive hyphens and trim leading/trailing hyphens
    let mut result = String::new();
    let mut last_was_hyphen = true; // Start true to skip leading hyphens

    for c in slug.chars() {
        if c == '-' {
            if !last_was_hyphen {
                result.push(c);
                last_was_hyphen = true;
            }
        } else {
            result.push(c);
            last_was_hyphen = false;
        }
    }

    // Remove trailing hyphen
    if result.ends_with('-') {
        result.pop();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_blog_context_new() {
        let ctx = BlogContext::new("Test Title", "The problem", "The solution");
        assert_eq!(ctx.title, "Test Title");
        assert_eq!(ctx.problem, "The problem");
        assert_eq!(ctx.solution, "The solution");
        assert!(ctx.date.is_empty());
    }

    #[test]
    fn test_blog_context_builder_pattern() {
        let ctx = BlogContext::new("Title", "Problem", "Solution")
            .with_date("2026-01-17")
            .with_author("Ralph")
            .with_summary("A brief summary")
            .with_challenges("Some challenges")
            .with_wins("Big wins")
            .with_lessons("Lessons learned")
            .with_next_steps("What's next")
            .with_technical_details("Technical stuff");

        assert_eq!(ctx.title, "Title");
        assert_eq!(ctx.date, "2026-01-17");
        assert_eq!(ctx.author, "Ralph");
        assert_eq!(ctx.summary, "A brief summary");
        assert_eq!(ctx.challenges, "Some challenges");
        assert_eq!(ctx.wins, "Big wins");
        assert_eq!(ctx.lessons, "Lessons learned");
        assert_eq!(ctx.next_steps, "What's next");
        assert_eq!(ctx.technical_details, "Technical stuff");
    }

    #[test]
    fn test_blog_context_default() {
        let ctx = BlogContext::default();
        assert!(ctx.title.is_empty());
        assert!(ctx.problem.is_empty());
        assert!(ctx.solution.is_empty());
    }

    #[test]
    fn test_render_template() {
        let template = "# {{ title }}\n\n**Date:** {{ date }}\n\n{{ problem }}";
        let ctx = BlogContext::new("My Title", "The problem here", "The solution")
            .with_date("2026-01-17");

        let rendered = render_template(template, &ctx);
        assert!(rendered.contains("# My Title"));
        assert!(rendered.contains("**Date:** 2026-01-17"));
        assert!(rendered.contains("The problem here"));
    }

    #[test]
    fn test_render_template_empty_fields() {
        let template = "{{ title }} - {{ challenges }}";
        let ctx = BlogContext::new("Title", "Problem", "Solution");

        let rendered = render_template(template, &ctx);
        assert_eq!(rendered, "Title - ");
    }

    #[test]
    fn test_slugify_basic() {
        assert_eq!(slugify("Hello World"), "hello-world");
    }

    #[test]
    fn test_slugify_special_characters() {
        assert_eq!(
            slugify("Feature: Add User Authentication!"),
            "feature-add-user-authentication"
        );
    }

    #[test]
    fn test_slugify_multiple_spaces() {
        assert_eq!(slugify("Hello   World"), "hello-world");
    }

    #[test]
    fn test_slugify_leading_trailing_special() {
        assert_eq!(slugify("--Hello World--"), "hello-world");
    }

    #[test]
    fn test_slugify_empty() {
        assert_eq!(slugify(""), "");
    }

    #[test]
    fn test_blog_generator_new() {
        let gen = BlogGenerator::new("/path/to/template.md", "/path/to/output");
        assert_eq!(gen.template_path(), Path::new("/path/to/template.md"));
        assert_eq!(gen.output_dir(), Path::new("/path/to/output"));
    }

    #[test]
    fn test_blog_generator_with_defaults() {
        let gen = BlogGenerator::with_defaults("/project/root");
        assert_eq!(
            gen.template_path(),
            Path::new("/project/root/docs/blog/templates/feature-release.md")
        );
        assert_eq!(gen.output_dir(), Path::new("/project/root/docs/blog/posts"));
    }

    #[test]
    fn test_blog_generator_template_not_found() {
        let gen = BlogGenerator::new("/nonexistent/template.md", "/tmp");
        let ctx = BlogContext::new("Title", "Problem", "Solution");

        let result = gen.generate(&ctx);
        assert!(matches!(
            result,
            Err(BlogGeneratorError::TemplateNotFound(_))
        ));
    }

    #[test]
    fn test_blog_generator_generate() {
        let temp_dir = TempDir::new().unwrap();
        let template_path = temp_dir.path().join("template.md");
        fs::write(&template_path, "# {{ title }}\n\n{{ problem }}").unwrap();

        let gen = BlogGenerator::new(&template_path, temp_dir.path());
        let ctx = BlogContext::new("Test Feature", "We had a problem", "We solved it");

        let result = gen.generate(&ctx).unwrap();
        assert!(result.contains("# Test Feature"));
        assert!(result.contains("We had a problem"));
    }

    #[test]
    fn test_blog_generator_save() {
        let temp_dir = TempDir::new().unwrap();
        let gen = BlogGenerator::new("/template.md", temp_dir.path());

        let content = "# Test Post\n\nSome content";
        let path = gen.save(content, "test-post").unwrap();

        assert_eq!(path, temp_dir.path().join("test-post.md"));
        let saved_content = fs::read_to_string(&path).unwrap();
        assert_eq!(saved_content, content);
    }

    #[test]
    fn test_blog_generator_save_with_extension() {
        let temp_dir = TempDir::new().unwrap();
        let gen = BlogGenerator::new("/template.md", temp_dir.path());

        let content = "# Test Post";
        let path = gen.save(content, "test-post.md").unwrap();

        assert_eq!(path, temp_dir.path().join("test-post.md"));
    }

    #[test]
    fn test_blog_generator_save_invalid_filename() {
        let temp_dir = TempDir::new().unwrap();
        let gen = BlogGenerator::new("/template.md", temp_dir.path());

        let result = gen.save("content", "path/with/slash");
        assert!(matches!(
            result,
            Err(BlogGeneratorError::InvalidFileName(_))
        ));

        let result = gen.save("content", "");
        assert!(matches!(
            result,
            Err(BlogGeneratorError::InvalidFileName(_))
        ));
    }

    #[test]
    fn test_blog_generator_save_output_dir_not_found() {
        let gen = BlogGenerator::new("/template.md", "/nonexistent/dir");

        let result = gen.save("content", "test");
        assert!(matches!(
            result,
            Err(BlogGeneratorError::OutputDirNotFound(_))
        ));
    }

    #[test]
    fn test_blog_generator_generate_and_save() {
        let temp_dir = TempDir::new().unwrap();
        let template_path = temp_dir.path().join("template.md");
        let output_dir = temp_dir.path().join("posts");
        fs::create_dir(&output_dir).unwrap();

        fs::write(&template_path, "# {{ title }}\n\n{{ summary }}").unwrap();

        let gen = BlogGenerator::new(&template_path, &output_dir);
        let ctx =
            BlogContext::new("My Feature", "Problem", "Solution").with_summary("A great feature");

        let path = gen.generate_and_save(&ctx, "my-feature").unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("# My Feature"));
        assert!(content.contains("A great feature"));
    }

    #[test]
    fn test_blog_generator_error_display() {
        let err = BlogGeneratorError::TemplateNotFound(PathBuf::from("/path/to/template"));
        assert_eq!(err.to_string(), "Template not found: /path/to/template");

        let err = BlogGeneratorError::OutputDirNotFound(PathBuf::from("/output"));
        assert_eq!(err.to_string(), "Output directory not found: /output");

        let err = BlogGeneratorError::InvalidFileName("bad/name".to_string());
        assert_eq!(err.to_string(), "Invalid file name: bad/name");
    }
}
