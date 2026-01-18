//! Styled CLI help and version display.
//!
//! Provides themed help output with ASCII art banner,
//! colored command lists, build information, and mascot animations.

#![allow(dead_code)]

use std::io::{self, Write};

use owo_colors::OwoColorize;

use super::colors::Theme;
use super::mascot::{AnimationConfig, Mascot, MascotRenderer, PeekAnimation};

/// ASCII art banner for Ralph CLI help.
pub const RALPH_BANNER: &str = r#"
  ╭─────────────────────────────────────────╮
  │                                         │
  │   ██████╗  █████╗ ██╗     ██████╗ ██╗  │
  │   ██╔══██╗██╔══██╗██║     ██╔══██╗██║  │
  │   ██████╔╝███████║██║     ██████╔╝███████║
  │   ██╔══██╗██╔══██║██║     ██╔═══╝ ██╔══██║
  │   ██║  ██║██║  ██║███████╗██║     ██║  ██║
  │   ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝╚═╝     ╚═╝  ╚═╝
  │                                         │
  ╰─────────────────────────────────────────╯
"#;

/// Compact ASCII art banner (single line).
pub const RALPH_BANNER_COMPACT: &str = r#"╭─ RALPH ─────────────────────────────────────╮"#;

/// Build information for version display.
#[derive(Debug, Clone)]
pub struct BuildInfo {
    /// Package version from Cargo.toml
    pub version: &'static str,
    /// Git commit hash (short)
    pub git_hash: Option<&'static str>,
    /// Build timestamp
    pub build_date: Option<&'static str>,
    /// Rust version used to compile
    pub rustc_version: Option<&'static str>,
}

impl Default for BuildInfo {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION"),
            git_hash: option_env!("RALPH_GIT_HASH"),
            build_date: option_env!("RALPH_BUILD_DATE"),
            rustc_version: option_env!("RALPH_RUSTC_VERSION"),
        }
    }
}

impl BuildInfo {
    /// Create new build info with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the target architecture using cfg attributes.
    pub fn target_arch() -> &'static str {
        #[cfg(target_arch = "x86_64")]
        return "x86_64";
        #[cfg(target_arch = "x86")]
        return "x86";
        #[cfg(target_arch = "aarch64")]
        return "aarch64";
        #[cfg(target_arch = "arm")]
        return "arm";
        #[cfg(not(any(
            target_arch = "x86_64",
            target_arch = "x86",
            target_arch = "aarch64",
            target_arch = "arm"
        )))]
        return "unknown";
    }

    /// Get the target OS using cfg attributes.
    pub fn target_os() -> &'static str {
        #[cfg(target_os = "macos")]
        return "macos";
        #[cfg(target_os = "linux")]
        return "linux";
        #[cfg(target_os = "windows")]
        return "windows";
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        return "unknown";
    }

    /// Get the full target string (arch-os).
    pub fn target() -> String {
        format!("{}-{}", Self::target_arch(), Self::target_os())
    }

    /// Format version string for display.
    pub fn version_string(&self) -> String {
        let mut version = format!("ralph {}", self.version);

        if let Some(hash) = self.git_hash {
            version.push_str(&format!(" ({})", hash));
        }

        version
    }

    /// Format full version info for --version output.
    pub fn full_version(&self) -> String {
        let mut lines = vec![self.version_string()];

        if let Some(date) = self.build_date {
            lines.push(format!("Built: {}", date));
        }

        lines.push(format!("Target: {}", Self::target()));

        if let Some(rustc) = self.rustc_version {
            lines.push(format!("Rustc: {}", rustc));
        }

        lines.join("\n")
    }
}

/// Command information for styled help display.
#[derive(Debug, Clone)]
pub struct CommandInfo {
    /// Command name
    pub name: &'static str,
    /// Short description
    pub description: &'static str,
    /// Arguments/options hint
    pub args_hint: Option<&'static str>,
}

impl CommandInfo {
    /// Create a new command info.
    pub const fn new(
        name: &'static str,
        description: &'static str,
        args_hint: Option<&'static str>,
    ) -> Self {
        Self {
            name,
            description,
            args_hint,
        }
    }
}

/// Available Ralph commands.
pub const COMMANDS: &[CommandInfo] = &[
    CommandInfo::new(
        "run",
        "Run all stories until complete (default if prd.json exists)",
        Some("[-p <FILE>] [-d <DIR>]"),
    ),
    CommandInfo::new(
        "mcp-server",
        "Start MCP server mode for integration with AI assistants",
        Some("[--prd <FILE>]"),
    ),
    CommandInfo::new(
        "quality",
        "Run quality checks (typecheck, lint, test)",
        None,
    ),
];

/// Global options for Ralph CLI.
pub const GLOBAL_OPTIONS: &[CommandInfo] = &[
    CommandInfo::new(
        "--ui <MODE>",
        "UI mode: auto (default), enabled, or disabled",
        None,
    ),
    CommandInfo::new(
        "--no-color",
        "Disable colors (also respects NO_COLOR env)",
        None,
    ),
    CommandInfo::new("--no-animation", "Disable startup animations", None),
    CommandInfo::new("--quiet, -q", "Suppress all output except errors", None),
    CommandInfo::new(
        "-v, -vv, -vvv",
        "Increase verbosity (expand details, debug info)",
        None,
    ),
    CommandInfo::new("--help, -h", "Print help information", None),
    CommandInfo::new(
        "--version, -V",
        "Print version information with mascot",
        None,
    ),
];

/// Styled help renderer using Ralph's theme.
pub struct HelpRenderer {
    theme: Theme,
    use_color: bool,
    /// Whether to play startup animations
    animate: bool,
    /// Mascot renderer for peek animations
    mascot_renderer: MascotRenderer,
}

impl Default for HelpRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl HelpRenderer {
    /// Create a new help renderer with default theme.
    pub fn new() -> Self {
        Self {
            theme: Theme::default(),
            use_color: true,
            animate: true,
            mascot_renderer: MascotRenderer::new(),
        }
    }

    /// Create a help renderer with color disabled.
    pub fn without_color() -> Self {
        Self {
            theme: Theme::default(),
            use_color: false,
            animate: true,
            mascot_renderer: MascotRenderer::new(),
        }
    }

    /// Set whether to use colors.
    pub fn with_color(mut self, use_color: bool) -> Self {
        self.use_color = use_color;
        self
    }

    /// Set whether to play animations.
    pub fn with_animation(mut self, animate: bool) -> Self {
        self.animate = animate;
        if !animate {
            self.mascot_renderer = MascotRenderer::without_animation();
        }
        self
    }

    /// Check if animations are enabled.
    pub fn animation_enabled(&self) -> bool {
        self.animate
    }

    /// Render the ASCII art banner with theme colors.
    pub fn render_banner(&self) -> String {
        if self.use_color {
            RALPH_BANNER.color(self.theme.story_id).to_string()
        } else {
            RALPH_BANNER.to_string()
        }
    }

    /// Render the compact banner.
    pub fn render_compact_banner(&self) -> String {
        if self.use_color {
            RALPH_BANNER_COMPACT.color(self.theme.story_id).to_string()
        } else {
            RALPH_BANNER_COMPACT.to_string()
        }
    }

    /// Render the tagline.
    pub fn render_tagline(&self) -> String {
        let tagline = "Enterprise-ready autonomous AI agent framework";
        if self.use_color {
            tagline.color(self.theme.muted).to_string()
        } else {
            tagline.to_string()
        }
    }

    /// Render a section header.
    pub fn render_section_header(&self, title: &str) -> String {
        if self.use_color {
            title.color(self.theme.warning).bold().to_string()
        } else {
            title.to_string()
        }
    }

    /// Render a command entry.
    pub fn render_command(&self, cmd: &CommandInfo) -> String {
        let name_part = if self.use_color {
            cmd.name.color(self.theme.success).bold().to_string()
        } else {
            cmd.name.to_string()
        };

        let args = cmd
            .args_hint
            .map(|a| {
                if self.use_color {
                    format!(" {}", a.color(self.theme.muted))
                } else {
                    format!(" {}", a)
                }
            })
            .unwrap_or_default();

        format!(
            "  {:<20}{}",
            format!("{}{}", name_part, args),
            cmd.description
        )
    }

    /// Render an option entry.
    pub fn render_option(&self, opt: &CommandInfo) -> String {
        let name_part = if self.use_color {
            opt.name.color(self.theme.in_progress).to_string()
        } else {
            opt.name.to_string()
        };

        format!("  {:<24}{}", name_part, opt.description)
    }

    /// Render the full styled help output.
    pub fn render_help(&self) -> String {
        let mut output = String::new();

        // Banner
        output.push_str(&self.render_banner());
        output.push('\n');

        // Tagline
        output.push_str(&format!("  {}\n\n", self.render_tagline()));

        // Usage
        output.push_str(&self.render_section_header("USAGE:"));
        output.push('\n');
        if self.use_color {
            output.push_str(&format!(
                "  {} {} {}\n\n",
                "ralph".color(self.theme.success).bold(),
                "[OPTIONS]".color(self.theme.muted),
                "[COMMAND]".color(self.theme.muted)
            ));
        } else {
            output.push_str("  ralph [OPTIONS] [COMMAND]\n\n");
        }

        // Commands
        output.push_str(&self.render_section_header("COMMANDS:"));
        output.push('\n');
        for cmd in COMMANDS {
            output.push_str(&self.render_command(cmd));
            output.push('\n');
        }
        output.push('\n');

        // Options
        output.push_str(&self.render_section_header("OPTIONS:"));
        output.push('\n');
        for opt in GLOBAL_OPTIONS {
            output.push_str(&self.render_option(opt));
            output.push('\n');
        }

        // Footer
        output.push('\n');
        if self.use_color {
            output.push_str(&format!(
                "  {} {}\n",
                "Learn more:".color(self.theme.muted),
                "https://github.com/kcirtapfromspace/ralph"
                    .color(self.theme.story_id)
                    .underline()
            ));
        } else {
            output.push_str("  Learn more: https://github.com/kcirtapfromspace/ralph\n");
        }

        // Add mascot with quote if animation is enabled
        if self.animate {
            let mascot = Mascot::random();
            output.push('\n');
            output.push_str(&self.mascot_renderer.render_with_quote(mascot));
        }

        output
    }

    /// Render styled version output.
    pub fn render_version(&self) -> String {
        let build_info = BuildInfo::new();
        let mut version_box = String::new();

        // Compact banner
        version_box.push_str(&self.render_compact_banner());
        version_box.push_str("\n│\n");

        // Version info
        let version_label = if self.use_color {
            "Version:".color(self.theme.muted).to_string()
        } else {
            "Version:".to_string()
        };

        let version_value = if self.use_color {
            build_info
                .version
                .color(self.theme.success)
                .bold()
                .to_string()
        } else {
            build_info.version.to_string()
        };

        version_box.push_str(&format!("│  {} {}\n", version_label, version_value));

        // Git hash if available
        if let Some(hash) = build_info.git_hash {
            let hash_label = if self.use_color {
                "Commit:".color(self.theme.muted).to_string()
            } else {
                "Commit:".to_string()
            };
            let hash_value = if self.use_color {
                hash.color(self.theme.story_id).to_string()
            } else {
                hash.to_string()
            };
            version_box.push_str(&format!("│  {} {}\n", hash_label, hash_value));
        }

        // Build date if available
        if let Some(date) = build_info.build_date {
            let date_label = if self.use_color {
                "Built:".color(self.theme.muted).to_string()
            } else {
                "Built:".to_string()
            };
            version_box.push_str(&format!("│  {} {}\n", date_label, date));
        }

        // Target
        let target_label = if self.use_color {
            "Target:".color(self.theme.muted).to_string()
        } else {
            "Target:".to_string()
        };
        version_box.push_str(&format!("│  {} {}\n", target_label, BuildInfo::target()));

        // Rustc version if available
        if let Some(rustc) = build_info.rustc_version {
            let rustc_label = if self.use_color {
                "Rustc:".color(self.theme.muted).to_string()
            } else {
                "Rustc:".to_string()
            };
            version_box.push_str(&format!("│  {} {}\n", rustc_label, rustc));
        }

        version_box.push_str("│\n");
        version_box.push_str("╰──────────────────────────────────────────────╯\n");

        // Add mascot beside version box if animation is enabled
        if self.animate {
            let mascot = Mascot::random();
            self.mascot_renderer
                .render_beside_content(&version_box, mascot)
        } else {
            version_box
        }
    }
}

/// Generate styled help template for clap.
///
/// This returns a help template string that clap can use,
/// but for full styled output, use `HelpRenderer::render_help()` instead.
pub fn clap_help_template() -> String {
    // We use a simple template and do the styling ourselves
    "{about}\n\n{usage-heading}\n  {usage}\n\n{all-args}".to_string()
}

/// Generate styled version string for clap.
pub fn clap_version_string() -> String {
    let build_info = BuildInfo::new();
    build_info.version_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_info_default() {
        let info = BuildInfo::default();
        assert!(!info.version.is_empty());
        assert!(!BuildInfo::target_arch().is_empty());
        assert!(!BuildInfo::target_os().is_empty());
        assert!(!BuildInfo::target().is_empty());
    }

    #[test]
    fn test_build_info_version_string() {
        let info = BuildInfo::default();
        let version = info.version_string();
        assert!(version.starts_with("ralph "));
        assert!(version.contains(info.version));
    }

    #[test]
    fn test_build_info_full_version() {
        let info = BuildInfo::default();
        let full = info.full_version();
        assert!(full.contains("ralph"));
        assert!(full.contains("Target:"));
    }

    #[test]
    fn test_command_info_new() {
        let cmd = CommandInfo::new("test", "Test command", Some("--arg"));
        assert_eq!(cmd.name, "test");
        assert_eq!(cmd.description, "Test command");
        assert_eq!(cmd.args_hint, Some("--arg"));
    }

    #[test]
    fn test_commands_defined() {
        // Verify COMMANDS has expected entries (at least 3: run, mcp-server, quality)
        assert!(COMMANDS.len() >= 3);
        assert!(COMMANDS.iter().any(|c| c.name == "run"));
        assert!(COMMANDS.iter().any(|c| c.name == "mcp-server"));
        assert!(COMMANDS.iter().any(|c| c.name == "quality"));
    }

    #[test]
    fn test_global_options_defined() {
        // Verify GLOBAL_OPTIONS has expected entries (at least 2: help, version)
        assert!(GLOBAL_OPTIONS.len() >= 2);
        assert!(GLOBAL_OPTIONS.iter().any(|o| o.name.contains("--help")));
        assert!(GLOBAL_OPTIONS.iter().any(|o| o.name.contains("--version")));
    }

    #[test]
    fn test_help_renderer_new() {
        let renderer = HelpRenderer::new();
        assert!(renderer.use_color);
    }

    #[test]
    fn test_help_renderer_without_color() {
        let renderer = HelpRenderer::without_color();
        assert!(!renderer.use_color);
    }

    #[test]
    fn test_help_renderer_with_color() {
        let renderer = HelpRenderer::new().with_color(false);
        assert!(!renderer.use_color);
    }

    #[test]
    fn test_render_banner() {
        let renderer = HelpRenderer::without_color();
        let banner = renderer.render_banner();
        assert!(banner.contains("RALPH") || banner.contains("██"));
    }

    #[test]
    fn test_render_compact_banner() {
        let renderer = HelpRenderer::without_color();
        let banner = renderer.render_compact_banner();
        assert!(banner.contains("RALPH"));
    }

    #[test]
    fn test_render_tagline() {
        let renderer = HelpRenderer::without_color();
        let tagline = renderer.render_tagline();
        assert!(tagline.contains("autonomous"));
    }

    #[test]
    fn test_render_section_header() {
        let renderer = HelpRenderer::without_color();
        let header = renderer.render_section_header("TEST:");
        assert_eq!(header, "TEST:");
    }

    #[test]
    fn test_render_command() {
        let renderer = HelpRenderer::without_color();
        let cmd = CommandInfo::new("test-cmd", "A test command", Some("[OPTIONS]"));
        let rendered = renderer.render_command(&cmd);
        assert!(rendered.contains("test-cmd"));
        assert!(rendered.contains("A test command"));
        assert!(rendered.contains("[OPTIONS]"));
    }

    #[test]
    fn test_render_option() {
        let renderer = HelpRenderer::without_color();
        let opt = CommandInfo::new("--test", "A test option", None);
        let rendered = renderer.render_option(&opt);
        assert!(rendered.contains("--test"));
        assert!(rendered.contains("A test option"));
    }

    #[test]
    fn test_render_help() {
        let renderer = HelpRenderer::without_color();
        let help = renderer.render_help();

        // Check all sections are present
        // The banner uses box drawing and block characters
        assert!(help.contains("╭─")); // Box drawing from banner
        assert!(help.contains("ralph")); // lowercase ralph in usage
        assert!(help.contains("USAGE:"));
        assert!(help.contains("COMMANDS:"));
        assert!(help.contains("OPTIONS:"));
        assert!(help.contains("run"));
        assert!(help.contains("mcp-server"));
        assert!(help.contains("quality"));
        assert!(help.contains("--help"));
        assert!(help.contains("--version"));
    }

    #[test]
    fn test_render_help_with_color() {
        let renderer = HelpRenderer::new();
        let help = renderer.render_help();

        // Should contain ANSI escape codes when color is enabled
        assert!(help.contains("\x1b["));
    }

    #[test]
    fn test_render_version() {
        let renderer = HelpRenderer::without_color();
        let version = renderer.render_version();

        assert!(version.contains("RALPH"));
        assert!(version.contains("Version:"));
        assert!(version.contains("Target:"));
    }

    #[test]
    fn test_render_version_with_color() {
        let renderer = HelpRenderer::new();
        let version = renderer.render_version();

        // Should contain ANSI escape codes when color is enabled
        assert!(version.contains("\x1b["));
    }

    #[test]
    fn test_clap_help_template() {
        let template = clap_help_template();
        assert!(template.contains("{about}"));
        assert!(template.contains("{usage}"));
    }

    #[test]
    fn test_clap_version_string() {
        let version = clap_version_string();
        assert!(version.starts_with("ralph "));
    }
}
