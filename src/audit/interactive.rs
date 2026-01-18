//! Interactive Q&A flow for codebase audit.
//!
//! This module provides an interactive question-and-answer interface to help
//! the audit better understand project goals and priorities. Based on user
//! responses, findings can be refined and prioritized accordingly.

use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};

use crate::audit::{AuditFinding, Severity};

/// A single question in the Q&A flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    /// Question number (1-indexed for display)
    pub number: u8,
    /// The question text
    pub text: String,
    /// Available options (A, B, C, D)
    pub options: Vec<QuestionOption>,
}

/// An option for a question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionOption {
    /// Option letter (A, B, C, D)
    pub letter: char,
    /// Option text
    pub text: String,
    /// Tags associated with this option for finding refinement
    pub tags: Vec<String>,
}

/// The type of project being audited
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProjectPurpose {
    /// Internal tool for the team/organization
    InternalTool,
    /// Customer-facing product or service
    CustomerFacing,
    /// Open source library or framework
    OpenSource,
    /// Prototype or proof of concept
    #[default]
    Prototype,
}

/// The main priority for the project
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProjectPriority {
    /// Speed of development is most important
    #[default]
    Speed,
    /// Code quality and maintainability is most important
    Quality,
    /// Security is most important
    Security,
    /// Performance is most important
    Performance,
}

/// The target users for the project
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TargetUsers {
    /// Developers/technical users
    #[default]
    Developers,
    /// Non-technical end users
    EndUsers,
    /// Enterprise customers
    Enterprise,
    /// All of the above
    Mixed,
}

/// The project's current stage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStage {
    /// New project, just starting
    #[default]
    New,
    /// Active development
    Active,
    /// Maintenance mode
    Maintenance,
    /// Legacy system
    Legacy,
}

/// User's answers to the Q&A flow
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserAnswers {
    /// Project purpose (Q1)
    pub purpose: ProjectPurpose,
    /// Main priority (Q2)
    pub priority: ProjectPriority,
    /// Target users (Q3)
    pub target_users: TargetUsers,
    /// Project stage (Q4)
    pub stage: ProjectStage,
    /// Raw answers as submitted (e.g., "1A", "2C")
    pub raw_answers: Vec<String>,
}

impl UserAnswers {
    /// Create default answers (used when skipping interactive mode)
    pub fn default_answers() -> Self {
        Self::default()
    }
}

/// Configuration for the interactive Q&A session
#[derive(Debug, Clone, Default)]
pub struct InteractiveConfig {
    /// Skip all interactive prompts
    pub no_interactive: bool,
    /// Smart mode: only ask when confidence is low
    pub smart_mode: bool,
    /// Confidence threshold for smart mode (0.0-1.0)
    pub confidence_threshold: f64,
}

impl InteractiveConfig {
    /// Create a new interactive configuration
    pub fn new() -> Self {
        Self {
            no_interactive: false,
            smart_mode: false,
            confidence_threshold: 0.7,
        }
    }

    /// Set no-interactive mode
    pub fn with_no_interactive(mut self, no_interactive: bool) -> Self {
        self.no_interactive = no_interactive;
        self
    }

    /// Set smart mode
    pub fn with_smart_mode(mut self, smart_mode: bool) -> Self {
        self.smart_mode = smart_mode;
        self
    }

    /// Set confidence threshold
    pub fn with_confidence_threshold(mut self, threshold: f64) -> Self {
        self.confidence_threshold = threshold.clamp(0.0, 1.0);
        self
    }
}

/// The interactive Q&A session handler
pub struct InteractiveSession {
    config: InteractiveConfig,
    questions: Vec<Question>,
}

impl Default for InteractiveSession {
    fn default() -> Self {
        Self::new()
    }
}

impl InteractiveSession {
    /// Create a new interactive session with default questions
    pub fn new() -> Self {
        Self {
            config: InteractiveConfig::new(),
            questions: Self::default_questions(),
        }
    }

    /// Create a session with custom configuration
    pub fn with_config(config: InteractiveConfig) -> Self {
        Self {
            config,
            questions: Self::default_questions(),
        }
    }

    /// Get the configuration
    pub fn config(&self) -> &InteractiveConfig {
        &self.config
    }

    /// Get the questions
    pub fn questions(&self) -> &[Question] {
        &self.questions
    }

    /// Generate the default set of questions
    fn default_questions() -> Vec<Question> {
        vec![
            Question {
                number: 1,
                text: "What is the primary purpose of this project?".to_string(),
                options: vec![
                    QuestionOption {
                        letter: 'A',
                        text: "Internal tool for our team/organization".to_string(),
                        tags: vec!["internal".to_string(), "tooling".to_string()],
                    },
                    QuestionOption {
                        letter: 'B',
                        text: "Customer-facing product or service".to_string(),
                        tags: vec!["customer".to_string(), "production".to_string()],
                    },
                    QuestionOption {
                        letter: 'C',
                        text: "Open source library or framework".to_string(),
                        tags: vec!["opensource".to_string(), "library".to_string()],
                    },
                    QuestionOption {
                        letter: 'D',
                        text: "Prototype or proof of concept".to_string(),
                        tags: vec!["prototype".to_string(), "experimental".to_string()],
                    },
                ],
            },
            Question {
                number: 2,
                text: "What is your main priority for this codebase?".to_string(),
                options: vec![
                    QuestionOption {
                        letter: 'A',
                        text: "Speed of development (move fast)".to_string(),
                        tags: vec!["speed".to_string(), "velocity".to_string()],
                    },
                    QuestionOption {
                        letter: 'B',
                        text: "Code quality and maintainability".to_string(),
                        tags: vec!["quality".to_string(), "maintainability".to_string()],
                    },
                    QuestionOption {
                        letter: 'C',
                        text: "Security and compliance".to_string(),
                        tags: vec!["security".to_string(), "compliance".to_string()],
                    },
                    QuestionOption {
                        letter: 'D',
                        text: "Performance and scalability".to_string(),
                        tags: vec!["performance".to_string(), "scalability".to_string()],
                    },
                ],
            },
            Question {
                number: 3,
                text: "Who are the primary users of this software?".to_string(),
                options: vec![
                    QuestionOption {
                        letter: 'A',
                        text: "Developers or technical users".to_string(),
                        tags: vec!["developers".to_string(), "technical".to_string()],
                    },
                    QuestionOption {
                        letter: 'B',
                        text: "Non-technical end users".to_string(),
                        tags: vec!["endusers".to_string(), "ux".to_string()],
                    },
                    QuestionOption {
                        letter: 'C',
                        text: "Enterprise customers".to_string(),
                        tags: vec!["enterprise".to_string(), "business".to_string()],
                    },
                    QuestionOption {
                        letter: 'D',
                        text: "Mixed audience (all of the above)".to_string(),
                        tags: vec!["mixed".to_string(), "general".to_string()],
                    },
                ],
            },
            Question {
                number: 4,
                text: "What is the current stage of this project?".to_string(),
                options: vec![
                    QuestionOption {
                        letter: 'A',
                        text: "New project, just getting started".to_string(),
                        tags: vec!["new".to_string(), "greenfield".to_string()],
                    },
                    QuestionOption {
                        letter: 'B',
                        text: "Active development with regular releases".to_string(),
                        tags: vec!["active".to_string(), "developing".to_string()],
                    },
                    QuestionOption {
                        letter: 'C',
                        text: "Maintenance mode (bug fixes only)".to_string(),
                        tags: vec!["maintenance".to_string(), "stable".to_string()],
                    },
                    QuestionOption {
                        letter: 'D',
                        text: "Legacy system (needs modernization)".to_string(),
                        tags: vec!["legacy".to_string(), "modernization".to_string()],
                    },
                ],
            },
        ]
    }

    /// Check if questions should be asked based on configuration and confidence
    pub fn should_ask_questions(&self, confidence: f64) -> bool {
        if self.config.no_interactive {
            return false;
        }

        if self.config.smart_mode {
            return confidence < self.config.confidence_threshold;
        }

        true
    }

    /// Run the interactive Q&A session and return user answers
    ///
    /// If `no_interactive` is set, returns default answers.
    /// If `smart_mode` is set and confidence >= threshold, returns default answers.
    pub fn run(&self, confidence: f64) -> io::Result<UserAnswers> {
        if !self.should_ask_questions(confidence) {
            return Ok(UserAnswers::default_answers());
        }

        self.run_with_reader_writer(&mut io::stdin().lock(), &mut io::stdout())
    }

    /// Run the interactive session with custom reader/writer (for testing)
    pub fn run_with_reader_writer<R: BufRead, W: Write>(
        &self,
        reader: &mut R,
        writer: &mut W,
    ) -> io::Result<UserAnswers> {
        writeln!(writer)?;
        writeln!(
            writer,
            "╭─────────────────────────────────────────────────────────────╮"
        )?;
        writeln!(
            writer,
            "│  📋 Codebase Audit Q&A                                      │"
        )?;
        writeln!(
            writer,
            "│  Answer a few questions to help prioritize findings.        │"
        )?;
        writeln!(
            writer,
            "╰─────────────────────────────────────────────────────────────╯"
        )?;
        writeln!(writer)?;
        writeln!(
            writer,
            "You can answer all at once (e.g., \"1A 2B 3C 4A\") or one by one."
        )?;
        writeln!(writer)?;

        // Display all questions first
        for question in &self.questions {
            writeln!(writer, "{}. {}", question.number, question.text)?;
            for option in &question.options {
                writeln!(writer, "   {}) {}", option.letter, option.text)?;
            }
            writeln!(writer)?;
        }

        // Prompt for input
        write!(writer, "Your answers: ")?;
        writer.flush()?;

        let mut input = String::new();
        reader.read_line(&mut input)?;

        let answers = self.parse_response(&input);

        writeln!(writer)?;
        writeln!(
            writer,
            "Thank you! Refining analysis based on your answers..."
        )?;
        writeln!(writer)?;

        Ok(answers)
    }

    /// Parse a response string into UserAnswers
    ///
    /// Accepts formats like:
    /// - "1A 2B 3C 4A"
    /// - "1A, 2B, 3C, 4A"
    /// - "A B C A" (assumes sequential questions)
    /// - "ABCA" (assumes sequential, all letters)
    pub fn parse_response(&self, input: &str) -> UserAnswers {
        let mut answers = UserAnswers::default();
        let input = input.trim().to_uppercase();

        // Try to parse as "1A 2B 3C 4A" format
        let parts: Vec<&str> = input
            .split(|c: char| c.is_whitespace() || c == ',')
            .collect();
        let parts: Vec<&str> = parts.into_iter().filter(|s| !s.is_empty()).collect();

        // Track which questions we've answered
        let mut answered = [false; 4];

        for part in &parts {
            if let Some((q_num, letter)) = Self::parse_answer_part(part) {
                if (1..=4).contains(&q_num) {
                    let idx = (q_num - 1) as usize;
                    if !answered[idx] {
                        answered[idx] = true;
                        answers.raw_answers.push(format!("{}{}", q_num, letter));
                        Self::apply_answer(&mut answers, q_num, letter);
                    }
                }
            }
        }

        // If no structured answers found, try parsing as just letters "ABCD"
        if answers.raw_answers.is_empty() {
            let letters: Vec<char> = input.chars().filter(|c| c.is_ascii_alphabetic()).collect();

            for (i, letter) in letters.iter().enumerate() {
                let q_num = (i + 1) as u8;
                if q_num <= 4 {
                    answers.raw_answers.push(format!("{}{}", q_num, letter));
                    Self::apply_answer(&mut answers, q_num, *letter);
                }
            }
        }

        answers
    }

    /// Parse a single answer part like "1A" or "A"
    fn parse_answer_part(part: &str) -> Option<(u8, char)> {
        let chars: Vec<char> = part.chars().collect();

        match chars.len() {
            1 => {
                // Just a letter, can't determine question number
                None
            }
            2 => {
                // Could be "1A" or "A1"
                if chars[0].is_ascii_digit() && chars[1].is_ascii_alphabetic() {
                    let q_num = chars[0].to_digit(10)? as u8;
                    Some((q_num, chars[1]))
                } else if chars[0].is_ascii_alphabetic() && chars[1].is_ascii_digit() {
                    let q_num = chars[1].to_digit(10)? as u8;
                    Some((q_num, chars[0]))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Apply an answer to the UserAnswers struct
    fn apply_answer(answers: &mut UserAnswers, question: u8, letter: char) {
        match question {
            1 => {
                answers.purpose = match letter {
                    'A' => ProjectPurpose::InternalTool,
                    'B' => ProjectPurpose::CustomerFacing,
                    'C' => ProjectPurpose::OpenSource,
                    'D' => ProjectPurpose::Prototype,
                    _ => ProjectPurpose::Prototype,
                };
            }
            2 => {
                answers.priority = match letter {
                    'A' => ProjectPriority::Speed,
                    'B' => ProjectPriority::Quality,
                    'C' => ProjectPriority::Security,
                    'D' => ProjectPriority::Performance,
                    _ => ProjectPriority::Speed,
                };
            }
            3 => {
                answers.target_users = match letter {
                    'A' => TargetUsers::Developers,
                    'B' => TargetUsers::EndUsers,
                    'C' => TargetUsers::Enterprise,
                    'D' => TargetUsers::Mixed,
                    _ => TargetUsers::Developers,
                };
            }
            4 => {
                answers.stage = match letter {
                    'A' => ProjectStage::New,
                    'B' => ProjectStage::Active,
                    'C' => ProjectStage::Maintenance,
                    'D' => ProjectStage::Legacy,
                    _ => ProjectStage::Active,
                };
            }
            _ => {}
        }
    }

    /// Refine findings based on user answers
    ///
    /// This adjusts severity and adds context based on the user's priorities.
    pub fn refine_findings(
        &self,
        findings: Vec<AuditFinding>,
        answers: &UserAnswers,
    ) -> Vec<AuditFinding> {
        findings
            .into_iter()
            .map(|mut finding| {
                // Adjust severity based on priority
                finding = self.adjust_severity_for_priority(finding, answers);

                // Add context to recommendations based on project type
                finding = self.add_context_to_recommendation(finding, answers);

                finding
            })
            .collect()
    }

    /// Adjust finding severity based on user's stated priorities
    fn adjust_severity_for_priority(
        &self,
        mut finding: AuditFinding,
        answers: &UserAnswers,
    ) -> AuditFinding {
        let category = finding.category.to_lowercase();

        match answers.priority {
            ProjectPriority::Security => {
                // Elevate security-related findings
                if category.contains("security")
                    || category.contains("vulnerability")
                    || finding.title.to_lowercase().contains("security")
                {
                    finding.severity = Self::elevate_severity(finding.severity);
                }
            }
            ProjectPriority::Quality => {
                // Elevate code quality findings
                if category.contains("debt")
                    || category.contains("quality")
                    || category.contains("maintainability")
                {
                    finding.severity = Self::elevate_severity(finding.severity);
                }
            }
            ProjectPriority::Performance => {
                // Elevate performance-related findings
                if category.contains("performance")
                    || category.contains("optimization")
                    || finding.title.to_lowercase().contains("performance")
                {
                    finding.severity = Self::elevate_severity(finding.severity);
                }
            }
            ProjectPriority::Speed => {
                // When speed is priority, lower severity of minor findings
                if finding.severity == Severity::Low {
                    // Keep low severity for low priority
                }
            }
        }

        // Adjust based on project purpose
        match answers.purpose {
            ProjectPurpose::CustomerFacing | ProjectPurpose::OpenSource => {
                // Customer-facing and open source should care more about quality
                if category.contains("documentation") {
                    finding.severity = Self::elevate_severity(finding.severity);
                }
            }
            ProjectPurpose::Prototype => {
                // Prototypes can have lower severity for non-critical issues
                if finding.severity == Severity::Low {
                    // Keep as-is, might even demote if we had a lower level
                }
            }
            ProjectPurpose::InternalTool => {
                // Internal tools might care less about documentation
            }
        }

        // Adjust based on project stage
        match answers.stage {
            ProjectStage::Legacy => {
                // Legacy systems should prioritize modernization issues
                if category.contains("deprecated") || category.contains("outdated") {
                    finding.severity = Self::elevate_severity(finding.severity);
                }
            }
            ProjectStage::Maintenance => {
                // Maintenance mode: stability is key
                if category.contains("breaking") || category.contains("compatibility") {
                    finding.severity = Self::elevate_severity(finding.severity);
                }
            }
            ProjectStage::New | ProjectStage::Active => {
                // Active development can address issues as they go
            }
        }

        finding
    }

    /// Add context to recommendations based on user answers
    fn add_context_to_recommendation(
        &self,
        mut finding: AuditFinding,
        answers: &UserAnswers,
    ) -> AuditFinding {
        // Add context based on target users
        if finding.category.contains("documentation") {
            match answers.target_users {
                TargetUsers::EndUsers => {
                    finding.recommendation = format!(
                        "{} Consider adding user-focused documentation and guides.",
                        finding.recommendation
                    );
                }
                TargetUsers::Developers => {
                    finding.recommendation = format!(
                        "{} Include API documentation and code examples.",
                        finding.recommendation
                    );
                }
                TargetUsers::Enterprise => {
                    finding.recommendation = format!(
                        "{} Add enterprise deployment and integration guides.",
                        finding.recommendation
                    );
                }
                TargetUsers::Mixed => {
                    finding.recommendation = format!(
                        "{} Consider documentation for different audience levels.",
                        finding.recommendation
                    );
                }
            }
        }

        // Add priority context
        match answers.priority {
            ProjectPriority::Security => {
                if finding.category.contains("security") {
                    finding.recommendation = format!(
                        "[HIGH PRIORITY based on your security focus] {}",
                        finding.recommendation
                    );
                }
            }
            ProjectPriority::Quality => {
                if finding.category.contains("debt") || finding.category.contains("quality") {
                    finding.recommendation = format!(
                        "[HIGH PRIORITY based on your quality focus] {}",
                        finding.recommendation
                    );
                }
            }
            ProjectPriority::Performance => {
                if finding.category.contains("performance") {
                    finding.recommendation = format!(
                        "[HIGH PRIORITY based on your performance focus] {}",
                        finding.recommendation
                    );
                }
            }
            ProjectPriority::Speed => {
                if finding.severity == Severity::Low {
                    finding.recommendation = format!(
                        "[Lower priority given speed focus] {}",
                        finding.recommendation
                    );
                }
            }
        }

        finding
    }

    /// Elevate a severity level by one step
    fn elevate_severity(severity: Severity) -> Severity {
        match severity {
            Severity::Low => Severity::Medium,
            Severity::Medium => Severity::High,
            Severity::High => Severity::Critical,
            Severity::Critical => Severity::Critical,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::path::PathBuf;

    #[test]
    fn test_default_questions() {
        let session = InteractiveSession::new();
        assert_eq!(session.questions().len(), 4);

        // Check first question
        let q1 = &session.questions()[0];
        assert_eq!(q1.number, 1);
        assert!(q1.text.contains("purpose"));
        assert_eq!(q1.options.len(), 4);
    }

    #[test]
    fn test_parse_response_structured() {
        let session = InteractiveSession::new();

        // Test "1A 2B 3C 4D" format
        let answers = session.parse_response("1A 2B 3C 4D");
        assert_eq!(answers.purpose, ProjectPurpose::InternalTool);
        assert_eq!(answers.priority, ProjectPriority::Quality);
        assert_eq!(answers.target_users, TargetUsers::Enterprise);
        assert_eq!(answers.stage, ProjectStage::Legacy);
        assert_eq!(answers.raw_answers.len(), 4);
    }

    #[test]
    fn test_parse_response_with_commas() {
        let session = InteractiveSession::new();

        // Test "1A, 2B, 3C, 4D" format
        let answers = session.parse_response("1A, 2B, 3C, 4D");
        assert_eq!(answers.purpose, ProjectPurpose::InternalTool);
        assert_eq!(answers.priority, ProjectPriority::Quality);
        assert_eq!(answers.raw_answers.len(), 4);
    }

    #[test]
    fn test_parse_response_letters_only() {
        let session = InteractiveSession::new();

        // Test "ABCD" format
        let answers = session.parse_response("ABCD");
        assert_eq!(answers.purpose, ProjectPurpose::InternalTool);
        assert_eq!(answers.priority, ProjectPriority::Quality);
        assert_eq!(answers.target_users, TargetUsers::Enterprise);
        assert_eq!(answers.stage, ProjectStage::Legacy);
    }

    #[test]
    fn test_parse_response_lowercase() {
        let session = InteractiveSession::new();

        // Test lowercase input
        let answers = session.parse_response("1a 2b 3c 4d");
        assert_eq!(answers.purpose, ProjectPurpose::InternalTool);
        assert_eq!(answers.priority, ProjectPriority::Quality);
    }

    #[test]
    fn test_parse_response_reversed_format() {
        let session = InteractiveSession::new();

        // Test "A1 B2 C3 D4" format
        let answers = session.parse_response("A1 B2 C3 D4");
        assert_eq!(answers.purpose, ProjectPurpose::InternalTool);
        assert_eq!(answers.priority, ProjectPriority::Quality);
    }

    #[test]
    fn test_parse_response_empty() {
        let session = InteractiveSession::new();

        let answers = session.parse_response("");
        assert_eq!(answers.purpose, ProjectPurpose::Prototype); // Default
        assert_eq!(answers.raw_answers.len(), 0);
    }

    #[test]
    fn test_should_ask_questions_no_interactive() {
        let config = InteractiveConfig::new().with_no_interactive(true);
        let session = InteractiveSession::with_config(config);

        assert!(!session.should_ask_questions(0.5));
        assert!(!session.should_ask_questions(0.0));
        assert!(!session.should_ask_questions(1.0));
    }

    #[test]
    fn test_should_ask_questions_smart_mode() {
        let config = InteractiveConfig::new()
            .with_smart_mode(true)
            .with_confidence_threshold(0.7);
        let session = InteractiveSession::with_config(config);

        assert!(session.should_ask_questions(0.5)); // Below threshold
        assert!(!session.should_ask_questions(0.7)); // At threshold
        assert!(!session.should_ask_questions(0.9)); // Above threshold
    }

    #[test]
    fn test_should_ask_questions_default() {
        let session = InteractiveSession::new();

        assert!(session.should_ask_questions(0.5));
        assert!(session.should_ask_questions(0.9));
    }

    #[test]
    fn test_run_with_reader_writer() {
        let session = InteractiveSession::new();

        let input = "1B 2C 3A 4B\n";
        let mut reader = Cursor::new(input);
        let mut writer = Vec::new();

        let answers = session
            .run_with_reader_writer(&mut reader, &mut writer)
            .unwrap();

        assert_eq!(answers.purpose, ProjectPurpose::CustomerFacing);
        assert_eq!(answers.priority, ProjectPriority::Security);
        assert_eq!(answers.target_users, TargetUsers::Developers);
        assert_eq!(answers.stage, ProjectStage::Active);

        // Check that output contains expected prompts
        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("Codebase Audit Q&A"));
        assert!(output.contains("primary purpose"));
    }

    #[test]
    fn test_elevate_severity() {
        assert_eq!(
            InteractiveSession::elevate_severity(Severity::Low),
            Severity::Medium
        );
        assert_eq!(
            InteractiveSession::elevate_severity(Severity::Medium),
            Severity::High
        );
        assert_eq!(
            InteractiveSession::elevate_severity(Severity::High),
            Severity::Critical
        );
        assert_eq!(
            InteractiveSession::elevate_severity(Severity::Critical),
            Severity::Critical
        );
    }

    #[test]
    fn test_refine_findings_security_priority() {
        let session = InteractiveSession::new();

        let findings = vec![AuditFinding {
            id: "SEC-001".to_string(),
            severity: Severity::Medium,
            category: "security".to_string(),
            title: "Security vulnerability".to_string(),
            description: "Test finding".to_string(),
            affected_files: vec![PathBuf::from("test.rs")],
            recommendation: "Fix it".to_string(),
        }];

        let answers = UserAnswers {
            priority: ProjectPriority::Security,
            ..Default::default()
        };

        let refined = session.refine_findings(findings, &answers);

        assert_eq!(refined.len(), 1);
        assert_eq!(refined[0].severity, Severity::High); // Elevated
    }

    #[test]
    fn test_refine_findings_quality_priority() {
        let session = InteractiveSession::new();

        let findings = vec![AuditFinding {
            id: "DEBT-001".to_string(),
            severity: Severity::Low,
            category: "tech_debt".to_string(),
            title: "Technical debt".to_string(),
            description: "Test finding".to_string(),
            affected_files: vec![PathBuf::from("test.rs")],
            recommendation: "Address it".to_string(),
        }];

        let answers = UserAnswers {
            priority: ProjectPriority::Quality,
            ..Default::default()
        };

        let refined = session.refine_findings(findings, &answers);

        assert_eq!(refined.len(), 1);
        assert_eq!(refined[0].severity, Severity::Medium); // Elevated
    }

    #[test]
    fn test_refine_findings_adds_context() {
        let session = InteractiveSession::new();

        let findings = vec![AuditFinding {
            id: "DOC-001".to_string(),
            severity: Severity::Low,
            category: "documentation".to_string(),
            title: "Missing docs".to_string(),
            description: "Missing documentation".to_string(),
            affected_files: vec![PathBuf::from("lib.rs")],
            recommendation: "Add documentation.".to_string(),
        }];

        let answers = UserAnswers {
            target_users: TargetUsers::Developers,
            ..Default::default()
        };

        let refined = session.refine_findings(findings, &answers);

        assert!(refined[0].recommendation.contains("API documentation"));
    }

    #[test]
    fn test_refine_findings_legacy_stage() {
        let session = InteractiveSession::new();

        let findings = vec![AuditFinding {
            id: "DEP-001".to_string(),
            severity: Severity::Low,
            category: "deprecated".to_string(),
            title: "Deprecated API".to_string(),
            description: "Using deprecated API".to_string(),
            affected_files: vec![PathBuf::from("old.rs")],
            recommendation: "Update to new API.".to_string(),
        }];

        let answers = UserAnswers {
            stage: ProjectStage::Legacy,
            ..Default::default()
        };

        let refined = session.refine_findings(findings, &answers);

        assert_eq!(refined[0].severity, Severity::Medium); // Elevated for legacy
    }

    #[test]
    fn test_config_builder() {
        let config = InteractiveConfig::new()
            .with_no_interactive(true)
            .with_smart_mode(true)
            .with_confidence_threshold(0.8);

        assert!(config.no_interactive);
        assert!(config.smart_mode);
        assert!((config.confidence_threshold - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_config_threshold_clamping() {
        let config = InteractiveConfig::new().with_confidence_threshold(1.5);
        assert!((config.confidence_threshold - 1.0).abs() < 0.001);

        let config = InteractiveConfig::new().with_confidence_threshold(-0.5);
        assert!(config.confidence_threshold.abs() < 0.001);
    }

    #[test]
    fn test_user_answers_default() {
        let answers = UserAnswers::default_answers();
        assert_eq!(answers.purpose, ProjectPurpose::Prototype);
        assert_eq!(answers.priority, ProjectPriority::Speed);
        assert_eq!(answers.target_users, TargetUsers::Developers);
        assert_eq!(answers.stage, ProjectStage::New);
    }

    #[test]
    fn test_question_option_tags() {
        let session = InteractiveSession::new();
        let q1 = &session.questions()[0];

        // First option (Internal tool) should have internal tag
        let internal_opt = q1.options.iter().find(|o| o.letter == 'A').unwrap();
        assert!(internal_opt.tags.contains(&"internal".to_string()));

        // Customer-facing should have production tag
        let customer_opt = q1.options.iter().find(|o| o.letter == 'B').unwrap();
        assert!(customer_opt.tags.contains(&"production".to_string()));
    }

    #[test]
    fn test_partial_answers() {
        let session = InteractiveSession::new();

        // Only answer some questions
        let answers = session.parse_response("1A 3C");
        assert_eq!(answers.purpose, ProjectPurpose::InternalTool);
        assert_eq!(answers.priority, ProjectPriority::Speed); // Default
        assert_eq!(answers.target_users, TargetUsers::Enterprise);
        assert_eq!(answers.stage, ProjectStage::New); // Default
        assert_eq!(answers.raw_answers.len(), 2);
    }

    #[test]
    fn test_duplicate_question_answers() {
        let session = InteractiveSession::new();

        // Answer same question twice - should use first answer
        let answers = session.parse_response("1A 1B 2C");
        assert_eq!(answers.purpose, ProjectPurpose::InternalTool); // First answer wins
        assert_eq!(answers.raw_answers.len(), 2); // Only 2 unique questions
    }

    #[test]
    fn test_all_answer_combinations() {
        let session = InteractiveSession::new();

        // Test all project purposes
        assert_eq!(
            session.parse_response("1A").purpose,
            ProjectPurpose::InternalTool
        );
        assert_eq!(
            session.parse_response("1B").purpose,
            ProjectPurpose::CustomerFacing
        );
        assert_eq!(
            session.parse_response("1C").purpose,
            ProjectPurpose::OpenSource
        );
        assert_eq!(
            session.parse_response("1D").purpose,
            ProjectPurpose::Prototype
        );

        // Test all priorities
        assert_eq!(
            session.parse_response("2A").priority,
            ProjectPriority::Speed
        );
        assert_eq!(
            session.parse_response("2B").priority,
            ProjectPriority::Quality
        );
        assert_eq!(
            session.parse_response("2C").priority,
            ProjectPriority::Security
        );
        assert_eq!(
            session.parse_response("2D").priority,
            ProjectPriority::Performance
        );

        // Test all target users
        assert_eq!(
            session.parse_response("3A").target_users,
            TargetUsers::Developers
        );
        assert_eq!(
            session.parse_response("3B").target_users,
            TargetUsers::EndUsers
        );
        assert_eq!(
            session.parse_response("3C").target_users,
            TargetUsers::Enterprise
        );
        assert_eq!(
            session.parse_response("3D").target_users,
            TargetUsers::Mixed
        );

        // Test all stages
        assert_eq!(session.parse_response("4A").stage, ProjectStage::New);
        assert_eq!(session.parse_response("4B").stage, ProjectStage::Active);
        assert_eq!(
            session.parse_response("4C").stage,
            ProjectStage::Maintenance
        );
        assert_eq!(session.parse_response("4D").stage, ProjectStage::Legacy);
    }

    #[test]
    fn test_serialization() {
        let answers = UserAnswers {
            purpose: ProjectPurpose::CustomerFacing,
            priority: ProjectPriority::Security,
            target_users: TargetUsers::Enterprise,
            stage: ProjectStage::Active,
            raw_answers: vec!["1B".to_string(), "2C".to_string()],
        };

        let json = serde_json::to_string(&answers).unwrap();
        let deserialized: UserAnswers = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.purpose, ProjectPurpose::CustomerFacing);
        assert_eq!(deserialized.priority, ProjectPriority::Security);
    }
}
