//! API inventory and endpoint discovery.

use ignore::WalkBuilder;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use super::AuditResult;

/// HTTP method
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    #[default]
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
    Trace,
    Connect,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Put => write!(f, "PUT"),
            HttpMethod::Patch => write!(f, "PATCH"),
            HttpMethod::Delete => write!(f, "DELETE"),
            HttpMethod::Head => write!(f, "HEAD"),
            HttpMethod::Options => write!(f, "OPTIONS"),
            HttpMethod::Trace => write!(f, "TRACE"),
            HttpMethod::Connect => write!(f, "CONNECT"),
        }
    }
}

/// Framework that provides the API
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiFramework {
    /// Axum web framework (Rust)
    Axum,
    /// Actix-web framework (Rust)
    ActixWeb,
    /// Rocket framework (Rust)
    Rocket,
    /// Express.js (Node.js)
    Express,
    /// Fastify (Node.js)
    Fastify,
    /// Hono (Deno/Bun/Node.js)
    Hono,
    /// Flask (Python)
    Flask,
    /// FastAPI (Python)
    FastApi,
    /// Django (Python)
    Django,
    /// Gin (Go)
    Gin,
    /// Echo (Go)
    Echo,
    /// Unknown framework
    #[default]
    Unknown,
}

impl std::fmt::Display for ApiFramework {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiFramework::Axum => write!(f, "axum"),
            ApiFramework::ActixWeb => write!(f, "actix-web"),
            ApiFramework::Rocket => write!(f, "rocket"),
            ApiFramework::Express => write!(f, "express"),
            ApiFramework::Fastify => write!(f, "fastify"),
            ApiFramework::Hono => write!(f, "hono"),
            ApiFramework::Flask => write!(f, "flask"),
            ApiFramework::FastApi => write!(f, "fastapi"),
            ApiFramework::Django => write!(f, "django"),
            ApiFramework::Gin => write!(f, "gin"),
            ApiFramework::Echo => write!(f, "echo"),
            ApiFramework::Unknown => write!(f, "unknown"),
        }
    }
}

/// An HTTP endpoint discovered in the codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpEndpoint {
    /// HTTP method
    pub method: HttpMethod,
    /// Route path (e.g., "/users/:id")
    pub path: String,
    /// Handler function name (if detectable)
    pub handler: Option<String>,
    /// File where this endpoint is defined
    pub file: PathBuf,
    /// Line number (if available)
    pub line: Option<usize>,
    /// Framework used
    pub framework: ApiFramework,
}

/// A CLI command discovered in the codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliCommand {
    /// Command name
    pub name: String,
    /// Description (if available)
    pub description: Option<String>,
    /// Subcommands (if any)
    pub subcommands: Vec<String>,
    /// File where this command is defined
    pub file: PathBuf,
    /// Line number (if available)
    pub line: Option<usize>,
    /// Framework used (clap, structopt, etc.)
    pub framework: String,
}

/// An MCP tool discovered in the codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    /// Tool name
    pub name: String,
    /// Tool description (if available)
    pub description: Option<String>,
    /// Input schema fields (if detectable)
    pub inputs: Vec<String>,
    /// File where this tool is defined
    pub file: PathBuf,
    /// Line number (if available)
    pub line: Option<usize>,
}

/// Complete API analysis results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApiAnalysis {
    /// Detected HTTP endpoints
    pub endpoints: Vec<HttpEndpoint>,
    /// Detected CLI commands
    pub commands: Vec<CliCommand>,
    /// Detected MCP tools
    pub mcp_tools: Vec<McpTool>,
    /// Detected frameworks
    pub frameworks: Vec<ApiFramework>,
    /// Total number of API surfaces
    pub total_api_surfaces: usize,
    /// Observations about the API structure
    pub observations: Vec<String>,
}

/// Analyzer for discovering API endpoints and interfaces
pub struct ApiInventory {
    root: PathBuf,
}

impl ApiInventory {
    /// Create a new API inventory analyzer
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Get the root path
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// Analyze the codebase for API endpoints and interfaces
    pub fn analyze(&self) -> AuditResult<ApiAnalysis> {
        let mut analysis = ApiAnalysis::default();

        // Walk the codebase
        let walker = WalkBuilder::new(&self.root)
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .build();

        for entry in walker.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            // Only analyze source code files
            if !matches!(ext.as_str(), "rs" | "js" | "ts" | "tsx" | "py" | "go") {
                continue;
            }

            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let relative_path = path.strip_prefix(&self.root).unwrap_or(path).to_path_buf();

            // Detect HTTP endpoints
            let endpoints = match ext.as_str() {
                "rs" => self.detect_rust_endpoints(&content, &relative_path),
                "js" | "ts" | "tsx" => self.detect_js_endpoints(&content, &relative_path),
                "py" => self.detect_python_endpoints(&content, &relative_path),
                "go" => self.detect_go_endpoints(&content, &relative_path),
                _ => Vec::new(),
            };
            analysis.endpoints.extend(endpoints);

            // Detect CLI commands
            let commands = match ext.as_str() {
                "rs" => self.detect_rust_cli_commands(&content, &relative_path),
                "js" | "ts" => self.detect_js_cli_commands(&content, &relative_path),
                "py" => self.detect_python_cli_commands(&content, &relative_path),
                "go" => self.detect_go_cli_commands(&content, &relative_path),
                _ => Vec::new(),
            };
            analysis.commands.extend(commands);

            // Detect MCP tools
            let mcp_tools = self.detect_mcp_tools(&content, &relative_path, &ext);
            analysis.mcp_tools.extend(mcp_tools);
        }

        // Collect unique frameworks
        let mut frameworks: Vec<ApiFramework> = analysis
            .endpoints
            .iter()
            .map(|e| e.framework.clone())
            .filter(|f| *f != ApiFramework::Unknown)
            .collect();
        frameworks.sort_by(|a, b| format!("{a}").cmp(&format!("{b}")));
        frameworks.dedup();
        analysis.frameworks = frameworks;

        // Calculate total API surfaces
        analysis.total_api_surfaces =
            analysis.endpoints.len() + analysis.commands.len() + analysis.mcp_tools.len();

        // Generate observations
        analysis.observations = self.generate_observations(&analysis);

        Ok(analysis)
    }

    /// Detect HTTP endpoints in Rust code (axum, actix-web, rocket)
    fn detect_rust_endpoints(&self, content: &str, file: &Path) -> Vec<HttpEndpoint> {
        let mut endpoints = Vec::new();

        // Detect framework
        let framework = if content.contains("axum::") || content.contains("use axum") {
            ApiFramework::Axum
        } else if content.contains("actix_web::") || content.contains("use actix_web") {
            ApiFramework::ActixWeb
        } else if content.contains("rocket::") || content.contains("use rocket") {
            ApiFramework::Rocket
        } else {
            ApiFramework::Unknown
        };

        // Axum route patterns: .route("/path", get(handler))
        let axum_re = Regex::new(
            r#"\.route\s*\(\s*["']([^"']+)["']\s*,\s*(get|post|put|patch|delete|head|options)\s*\(\s*(\w+)\s*\)"#,
        )
        .unwrap();
        for cap in axum_re.captures_iter(content) {
            let path = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let method_str = cap.get(2).map(|m| m.as_str()).unwrap_or("get");
            let handler = cap.get(3).map(|m| m.as_str().to_string());

            let method = Self::parse_http_method(method_str);
            let line = Self::find_line_number(content, cap.get(0).unwrap().start());

            endpoints.push(HttpEndpoint {
                method,
                path: path.to_string(),
                handler,
                file: file.to_path_buf(),
                line: Some(line),
                framework: ApiFramework::Axum,
            });
        }

        // Axum method_router patterns: get("/path").handler(...)
        let axum_method_re =
            Regex::new(r#"(get|post|put|patch|delete)\s*\(\s*["']([^"']+)["']\s*\)"#).unwrap();
        if framework == ApiFramework::Axum {
            for cap in axum_method_re.captures_iter(content) {
                let method_str = cap.get(1).map(|m| m.as_str()).unwrap_or("get");
                let path = cap.get(2).map(|m| m.as_str()).unwrap_or("");

                // Skip if already captured by route pattern
                if endpoints.iter().any(|e| e.path == path) {
                    continue;
                }

                let method = Self::parse_http_method(method_str);
                let line = Self::find_line_number(content, cap.get(0).unwrap().start());

                endpoints.push(HttpEndpoint {
                    method,
                    path: path.to_string(),
                    handler: None,
                    file: file.to_path_buf(),
                    line: Some(line),
                    framework: ApiFramework::Axum,
                });
            }
        }

        // Actix-web attribute patterns: #[get("/path")] or #[route("/path", method = "GET")]
        let actix_attr_re =
            Regex::new(r#"#\[(get|post|put|patch|delete|head|options)\s*\(\s*["']([^"']+)["']"#)
                .unwrap();
        for cap in actix_attr_re.captures_iter(content) {
            let method_str = cap.get(1).map(|m| m.as_str()).unwrap_or("get");
            let path = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            let method = Self::parse_http_method(method_str);
            let line = Self::find_line_number(content, cap.get(0).unwrap().start());

            // Try to find the handler name on the next line
            let handler = Self::find_next_function_name(content, cap.get(0).unwrap().end());

            endpoints.push(HttpEndpoint {
                method,
                path: path.to_string(),
                handler,
                file: file.to_path_buf(),
                line: Some(line),
                framework: ApiFramework::ActixWeb,
            });
        }

        // Actix-web resource patterns: .resource("/path").route(web::get().to(handler))
        let actix_resource_re = Regex::new(
            r#"\.resource\s*\(\s*["']([^"']+)["']\s*\)[^;]*\.route\s*\(\s*web::(get|post|put|patch|delete)\s*\(\s*\)\s*\.to\s*\(\s*(\w+)\s*\)"#,
        )
        .unwrap();
        for cap in actix_resource_re.captures_iter(content) {
            let path = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let method_str = cap.get(2).map(|m| m.as_str()).unwrap_or("get");
            let handler = cap.get(3).map(|m| m.as_str().to_string());

            let method = Self::parse_http_method(method_str);
            let line = Self::find_line_number(content, cap.get(0).unwrap().start());

            endpoints.push(HttpEndpoint {
                method,
                path: path.to_string(),
                handler,
                file: file.to_path_buf(),
                line: Some(line),
                framework: ApiFramework::ActixWeb,
            });
        }

        // Rocket attribute patterns: #[get("/path")]
        let rocket_re =
            Regex::new(r#"#\[(get|post|put|patch|delete|head|options)\s*\(\s*["']([^"']+)["']"#)
                .unwrap();
        if framework == ApiFramework::Rocket {
            for cap in rocket_re.captures_iter(content) {
                let method_str = cap.get(1).map(|m| m.as_str()).unwrap_or("get");
                let path = cap.get(2).map(|m| m.as_str()).unwrap_or("");

                // Skip if already captured by actix patterns
                if endpoints.iter().any(|e| e.path == path) {
                    continue;
                }

                let method = Self::parse_http_method(method_str);
                let line = Self::find_line_number(content, cap.get(0).unwrap().start());
                let handler = Self::find_next_function_name(content, cap.get(0).unwrap().end());

                endpoints.push(HttpEndpoint {
                    method,
                    path: path.to_string(),
                    handler,
                    file: file.to_path_buf(),
                    line: Some(line),
                    framework: ApiFramework::Rocket,
                });
            }
        }

        endpoints
    }

    /// Detect HTTP endpoints in JavaScript/TypeScript code (express, fastify, hono)
    fn detect_js_endpoints(&self, content: &str, file: &Path) -> Vec<HttpEndpoint> {
        let mut endpoints = Vec::new();

        // Detect framework
        let framework = if content.contains("express()")
            || content.contains("require('express')")
            || content.contains("from 'express'")
            || content.contains("from \"express\"")
        {
            ApiFramework::Express
        } else if content.contains("fastify()")
            || content.contains("require('fastify')")
            || content.contains("from 'fastify'")
            || content.contains("from \"fastify\"")
        {
            ApiFramework::Fastify
        } else if content.contains("Hono()")
            || content.contains("from 'hono'")
            || content.contains("from \"hono\"")
        {
            ApiFramework::Hono
        } else {
            ApiFramework::Unknown
        };

        // Express/Fastify/Hono patterns: app.get('/path', handler) or router.get('/path', ...)
        let express_re = Regex::new(
            r#"(?:app|router|server)\.(get|post|put|patch|delete|head|options)\s*\(\s*['"`]([^'"`]+)['"`]"#,
        )
        .unwrap();
        for cap in express_re.captures_iter(content) {
            let method_str = cap.get(1).map(|m| m.as_str()).unwrap_or("get");
            let path = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            let method = Self::parse_http_method(method_str);
            let line = Self::find_line_number(content, cap.get(0).unwrap().start());

            endpoints.push(HttpEndpoint {
                method,
                path: path.to_string(),
                handler: None,
                file: file.to_path_buf(),
                line: Some(line),
                framework: framework.clone(),
            });
        }

        // Hono-specific chained patterns: app.get('/path').post('/path')
        if framework == ApiFramework::Hono {
            let hono_chain_re =
                Regex::new(r#"\.(get|post|put|patch|delete)\s*\(\s*['"`]([^'"`]+)['"`]"#).unwrap();
            for cap in hono_chain_re.captures_iter(content) {
                let method_str = cap.get(1).map(|m| m.as_str()).unwrap_or("get");
                let path = cap.get(2).map(|m| m.as_str()).unwrap_or("");

                // Skip if already captured
                if endpoints.iter().any(|e| e.path == path) {
                    continue;
                }

                let method = Self::parse_http_method(method_str);
                let line = Self::find_line_number(content, cap.get(0).unwrap().start());

                endpoints.push(HttpEndpoint {
                    method,
                    path: path.to_string(),
                    handler: None,
                    file: file.to_path_buf(),
                    line: Some(line),
                    framework: ApiFramework::Hono,
                });
            }
        }

        endpoints
    }

    /// Detect HTTP endpoints in Python code (flask, fastapi, django)
    fn detect_python_endpoints(&self, content: &str, file: &Path) -> Vec<HttpEndpoint> {
        let mut endpoints = Vec::new();

        // Detect framework
        let framework = if content.contains("from flask") || content.contains("import flask") {
            ApiFramework::Flask
        } else if content.contains("from fastapi") || content.contains("import fastapi") {
            ApiFramework::FastApi
        } else if content.contains("from django") || content.contains("import django") {
            ApiFramework::Django
        } else {
            ApiFramework::Unknown
        };

        // Flask/FastAPI decorator patterns: @app.get("/path") or @router.get("/path")
        let decorator_re = Regex::new(
            r#"@(?:app|router|api|blueprint)\.(get|post|put|patch|delete|head|options)\s*\(\s*['"]([^'"]+)['"]"#,
        )
        .unwrap();
        for cap in decorator_re.captures_iter(content) {
            let method_str = cap.get(1).map(|m| m.as_str()).unwrap_or("get");
            let path = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            let method = Self::parse_http_method(method_str);
            let line = Self::find_line_number(content, cap.get(0).unwrap().start());
            let handler = Self::find_next_python_function(content, cap.get(0).unwrap().end());

            endpoints.push(HttpEndpoint {
                method,
                path: path.to_string(),
                handler,
                file: file.to_path_buf(),
                line: Some(line),
                framework: framework.clone(),
            });
        }

        // Flask route decorator: @app.route("/path", methods=["GET"])
        let flask_route_re =
            Regex::new(r#"@(?:app|blueprint)\.route\s*\(\s*['"]([^'"]+)['"]"#).unwrap();
        for cap in flask_route_re.captures_iter(content) {
            let path = cap.get(1).map(|m| m.as_str()).unwrap_or("");

            // Skip if already captured
            if endpoints.iter().any(|e| e.path == path) {
                continue;
            }

            let line = Self::find_line_number(content, cap.get(0).unwrap().start());
            let handler = Self::find_next_python_function(content, cap.get(0).unwrap().end());

            // Try to detect methods from the decorator
            let full_match = &content[cap.get(0).unwrap().start()..];
            let method = if full_match.contains("methods=") {
                if full_match.contains("POST") {
                    HttpMethod::Post
                } else if full_match.contains("PUT") {
                    HttpMethod::Put
                } else if full_match.contains("DELETE") {
                    HttpMethod::Delete
                } else if full_match.contains("PATCH") {
                    HttpMethod::Patch
                } else {
                    HttpMethod::Get
                }
            } else {
                HttpMethod::Get
            };

            endpoints.push(HttpEndpoint {
                method,
                path: path.to_string(),
                handler,
                file: file.to_path_buf(),
                line: Some(line),
                framework: ApiFramework::Flask,
            });
        }

        // Django URL patterns: path('route/', view, name='name')
        let django_re = Regex::new(r#"path\s*\(\s*['"]([^'"]+)['"]"#).unwrap();
        if framework == ApiFramework::Django {
            for cap in django_re.captures_iter(content) {
                let path = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                let line = Self::find_line_number(content, cap.get(0).unwrap().start());

                endpoints.push(HttpEndpoint {
                    method: HttpMethod::Get, // Django paths handle all methods by default
                    path: path.to_string(),
                    handler: None,
                    file: file.to_path_buf(),
                    line: Some(line),
                    framework: ApiFramework::Django,
                });
            }
        }

        endpoints
    }

    /// Detect HTTP endpoints in Go code (gin, echo)
    fn detect_go_endpoints(&self, content: &str, file: &Path) -> Vec<HttpEndpoint> {
        let mut endpoints = Vec::new();

        // Detect framework
        let framework = if content.contains("\"github.com/gin-gonic/gin\"")
            || content.contains("gin.")
        {
            ApiFramework::Gin
        } else if content.contains("\"github.com/labstack/echo\"") || content.contains("echo.") {
            ApiFramework::Echo
        } else {
            ApiFramework::Unknown
        };

        // Gin/Echo patterns: router.GET("/path", handler)
        let gin_re = Regex::new(
            r#"(?:router|r|e|g|app)\.(GET|POST|PUT|PATCH|DELETE|HEAD|OPTIONS)\s*\(\s*["']([^"']+)["']"#,
        )
        .unwrap();
        for cap in gin_re.captures_iter(content) {
            let method_str = cap.get(1).map(|m| m.as_str()).unwrap_or("GET");
            let path = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            let method = Self::parse_http_method(method_str);
            let line = Self::find_line_number(content, cap.get(0).unwrap().start());

            endpoints.push(HttpEndpoint {
                method,
                path: path.to_string(),
                handler: None,
                file: file.to_path_buf(),
                line: Some(line),
                framework: framework.clone(),
            });
        }

        endpoints
    }

    /// Detect CLI commands in Rust code (clap, structopt)
    fn detect_rust_cli_commands(&self, content: &str, file: &Path) -> Vec<CliCommand> {
        let mut commands = Vec::new();

        // Detect clap Command patterns
        let has_clap = content.contains("clap::") || content.contains("use clap");
        if !has_clap {
            return commands;
        }

        // Clap derive pattern: #[command(name = "...")]
        let derive_re = Regex::new(r#"#\[command\s*\([^)]*name\s*=\s*["']([^"']+)["']"#).unwrap();
        for cap in derive_re.captures_iter(content) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let line = Self::find_line_number(content, cap.get(0).unwrap().start());

            // Try to find description
            let description = Self::find_clap_about(content, cap.get(0).unwrap().start());

            commands.push(CliCommand {
                name: name.to_string(),
                description,
                subcommands: Vec::new(),
                file: file.to_path_buf(),
                line: Some(line),
                framework: "clap".to_string(),
            });
        }

        // Clap builder pattern: Command::new("name")
        let builder_re = Regex::new(r#"Command::new\s*\(\s*["']([^"']+)["']\s*\)"#).unwrap();
        for cap in builder_re.captures_iter(content) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");

            // Skip if already captured
            if commands.iter().any(|c| c.name == name) {
                continue;
            }

            let line = Self::find_line_number(content, cap.get(0).unwrap().start());

            commands.push(CliCommand {
                name: name.to_string(),
                description: None,
                subcommands: Vec::new(),
                file: file.to_path_buf(),
                line: Some(line),
                framework: "clap".to_string(),
            });
        }

        // Detect subcommands: #[command(subcommand)]
        let subcommand_re = Regex::new(r#"#\[derive\([^)]*Subcommand[^)]*\)\]"#).unwrap();
        if subcommand_re.is_match(content) {
            // Find enum variants
            let variant_re = Regex::new(r#"^\s*(\w+)\s*(?:\{|,|\()"#).unwrap();
            for line in content.lines() {
                if let Some(cap) = variant_re.captures(line) {
                    if let Some(name) = cap.get(1) {
                        let variant_name = name.as_str();
                        // Skip common Rust keywords and derive macro related items
                        if !["pub", "struct", "enum", "fn", "impl", "use", "mod", "type"]
                            .contains(&variant_name)
                        {
                            // Add as subcommand to the main command if exists
                            if let Some(cmd) = commands.first_mut() {
                                cmd.subcommands.push(variant_name.to_string());
                            }
                        }
                    }
                }
            }
        }

        commands
    }

    /// Detect CLI commands in JavaScript/TypeScript code (commander, yargs)
    fn detect_js_cli_commands(&self, content: &str, file: &Path) -> Vec<CliCommand> {
        let mut commands = Vec::new();

        // Commander.js patterns: .command('name')
        let commander_re = Regex::new(r#"\.command\s*\(\s*['"]([^'"]+)['"]"#).unwrap();
        for cap in commander_re.captures_iter(content) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let line = Self::find_line_number(content, cap.get(0).unwrap().start());

            commands.push(CliCommand {
                name: name.to_string(),
                description: None,
                subcommands: Vec::new(),
                file: file.to_path_buf(),
                line: Some(line),
                framework: "commander".to_string(),
            });
        }

        // Yargs patterns: .command('name', 'description')
        if content.contains("yargs") {
            let yargs_re = Regex::new(r#"\.command\s*\(\s*['"]([^'"]+)['"]"#).unwrap();
            for cap in yargs_re.captures_iter(content) {
                let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");

                // Skip if already captured
                if commands.iter().any(|c| c.name == name) {
                    continue;
                }

                let line = Self::find_line_number(content, cap.get(0).unwrap().start());

                commands.push(CliCommand {
                    name: name.to_string(),
                    description: None,
                    subcommands: Vec::new(),
                    file: file.to_path_buf(),
                    line: Some(line),
                    framework: "yargs".to_string(),
                });
            }
        }

        commands
    }

    /// Detect CLI commands in Python code (click, argparse, typer)
    fn detect_python_cli_commands(&self, content: &str, file: &Path) -> Vec<CliCommand> {
        let mut commands = Vec::new();

        // Click patterns: @click.command() or @click.group()
        let click_re = Regex::new(r#"@(?:click|app)\.(command|group)\s*\([^)]*\)"#).unwrap();
        for cap in click_re.captures_iter(content) {
            let line = Self::find_line_number(content, cap.get(0).unwrap().start());
            let handler = Self::find_next_python_function(content, cap.get(0).unwrap().end());

            if let Some(name) = handler {
                commands.push(CliCommand {
                    name,
                    description: None,
                    subcommands: Vec::new(),
                    file: file.to_path_buf(),
                    line: Some(line),
                    framework: "click".to_string(),
                });
            }
        }

        // Typer patterns: @app.command()
        if content.contains("typer") {
            let typer_re = Regex::new(r#"@app\.command\s*\([^)]*\)"#).unwrap();
            for cap in typer_re.captures_iter(content) {
                let line = Self::find_line_number(content, cap.get(0).unwrap().start());
                let handler = Self::find_next_python_function(content, cap.get(0).unwrap().end());

                if let Some(name) = handler {
                    if !commands.iter().any(|c| c.name == name) {
                        commands.push(CliCommand {
                            name,
                            description: None,
                            subcommands: Vec::new(),
                            file: file.to_path_buf(),
                            line: Some(line),
                            framework: "typer".to_string(),
                        });
                    }
                }
            }
        }

        // Argparse patterns: subparsers.add_parser('name')
        let argparse_re = Regex::new(r#"add_parser\s*\(\s*['"]([^'"]+)['"]"#).unwrap();
        for cap in argparse_re.captures_iter(content) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let line = Self::find_line_number(content, cap.get(0).unwrap().start());

            commands.push(CliCommand {
                name: name.to_string(),
                description: None,
                subcommands: Vec::new(),
                file: file.to_path_buf(),
                line: Some(line),
                framework: "argparse".to_string(),
            });
        }

        commands
    }

    /// Detect CLI commands in Go code (cobra, urfave/cli)
    fn detect_go_cli_commands(&self, content: &str, file: &Path) -> Vec<CliCommand> {
        let mut commands = Vec::new();

        // Cobra patterns: &cobra.Command{Use: "name"}
        let cobra_re = Regex::new(r#"&cobra\.Command\s*\{[^}]*Use:\s*["']([^"']+)["']"#).unwrap();
        for cap in cobra_re.captures_iter(content) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let line = Self::find_line_number(content, cap.get(0).unwrap().start());

            commands.push(CliCommand {
                name: name.to_string(),
                description: None,
                subcommands: Vec::new(),
                file: file.to_path_buf(),
                line: Some(line),
                framework: "cobra".to_string(),
            });
        }

        // urfave/cli patterns: &cli.Command{Name: "name"}
        let cli_re = Regex::new(r#"&cli\.Command\s*\{[^}]*Name:\s*["']([^"']+)["']"#).unwrap();
        for cap in cli_re.captures_iter(content) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let line = Self::find_line_number(content, cap.get(0).unwrap().start());

            commands.push(CliCommand {
                name: name.to_string(),
                description: None,
                subcommands: Vec::new(),
                file: file.to_path_buf(),
                line: Some(line),
                framework: "urfave/cli".to_string(),
            });
        }

        commands
    }

    /// Detect MCP tools in the codebase
    fn detect_mcp_tools(&self, content: &str, file: &Path, ext: &str) -> Vec<McpTool> {
        let mut tools = Vec::new();

        // Check for MCP-related imports/usage
        let has_mcp = content.contains("mcp")
            || content.contains("MCP")
            || content.contains("ModelContextProtocol")
            || content.contains("model_context_protocol")
            || content.contains("@modelcontextprotocol");

        if !has_mcp {
            return tools;
        }

        match ext {
            "rs" => {
                // Rust MCP tool patterns
                // Look for tool definitions like: Tool::new("name")
                let tool_re = Regex::new(r#"Tool::new\s*\(\s*["']([^"']+)["']\s*\)"#).unwrap();
                for cap in tool_re.captures_iter(content) {
                    let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                    let line = Self::find_line_number(content, cap.get(0).unwrap().start());

                    tools.push(McpTool {
                        name: name.to_string(),
                        description: None,
                        inputs: Vec::new(),
                        file: file.to_path_buf(),
                        line: Some(line),
                    });
                }

                // Look for #[tool] or #[mcp_tool] attributes
                let attr_re =
                    Regex::new(r#"#\[(?:tool|mcp_tool)\s*\([^)]*name\s*=\s*["']([^"']+)["']"#)
                        .unwrap();
                for cap in attr_re.captures_iter(content) {
                    let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                    let line = Self::find_line_number(content, cap.get(0).unwrap().start());

                    if !tools.iter().any(|t| t.name == name) {
                        tools.push(McpTool {
                            name: name.to_string(),
                            description: None,
                            inputs: Vec::new(),
                            file: file.to_path_buf(),
                            line: Some(line),
                        });
                    }
                }
            }
            "js" | "ts" | "tsx" => {
                // TypeScript/JavaScript MCP patterns
                // Look for tool definitions: { name: "tool_name", ... }
                let tool_re =
                    Regex::new(r#"\{\s*name:\s*['"]([^'"]+)['"][^}]*(?:description|inputSchema)"#)
                        .unwrap();
                for cap in tool_re.captures_iter(content) {
                    let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                    let line = Self::find_line_number(content, cap.get(0).unwrap().start());

                    tools.push(McpTool {
                        name: name.to_string(),
                        description: None,
                        inputs: Vec::new(),
                        file: file.to_path_buf(),
                        line: Some(line),
                    });
                }

                // Look for server.tool("name", ...) or addTool("name", ...)
                let method_re =
                    Regex::new(r#"(?:server\.tool|addTool)\s*\(\s*['"]([^'"]+)['"]"#).unwrap();
                for cap in method_re.captures_iter(content) {
                    let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");

                    if !tools.iter().any(|t| t.name == name) {
                        let line = Self::find_line_number(content, cap.get(0).unwrap().start());
                        tools.push(McpTool {
                            name: name.to_string(),
                            description: None,
                            inputs: Vec::new(),
                            file: file.to_path_buf(),
                            line: Some(line),
                        });
                    }
                }
            }
            "py" => {
                // Python MCP patterns
                // Look for @tool decorator or Tool(name="...")
                let decorator_re =
                    Regex::new(r#"@tool\s*\(\s*name\s*=\s*['"]([^'"]+)['"]"#).unwrap();
                for cap in decorator_re.captures_iter(content) {
                    let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                    let line = Self::find_line_number(content, cap.get(0).unwrap().start());

                    tools.push(McpTool {
                        name: name.to_string(),
                        description: None,
                        inputs: Vec::new(),
                        file: file.to_path_buf(),
                        line: Some(line),
                    });
                }

                // Look for Tool(name="...")
                let tool_re = Regex::new(r#"Tool\s*\([^)]*name\s*=\s*['"]([^'"]+)['"]"#).unwrap();
                for cap in tool_re.captures_iter(content) {
                    let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");

                    if !tools.iter().any(|t| t.name == name) {
                        let line = Self::find_line_number(content, cap.get(0).unwrap().start());
                        tools.push(McpTool {
                            name: name.to_string(),
                            description: None,
                            inputs: Vec::new(),
                            file: file.to_path_buf(),
                            line: Some(line),
                        });
                    }
                }
            }
            _ => {}
        }

        tools
    }

    /// Parse HTTP method from string
    fn parse_http_method(method: &str) -> HttpMethod {
        match method.to_lowercase().as_str() {
            "get" => HttpMethod::Get,
            "post" => HttpMethod::Post,
            "put" => HttpMethod::Put,
            "patch" => HttpMethod::Patch,
            "delete" => HttpMethod::Delete,
            "head" => HttpMethod::Head,
            "options" => HttpMethod::Options,
            "trace" => HttpMethod::Trace,
            "connect" => HttpMethod::Connect,
            _ => HttpMethod::Get,
        }
    }

    /// Find line number for a character position
    fn find_line_number(content: &str, pos: usize) -> usize {
        content[..pos].chars().filter(|c| *c == '\n').count() + 1
    }

    /// Find the next function name after a position (for Rust)
    fn find_next_function_name(content: &str, pos: usize) -> Option<String> {
        let remaining = &content[pos..];
        let fn_re = Regex::new(r#"(?:pub\s+)?(?:async\s+)?fn\s+(\w+)"#).unwrap();
        fn_re
            .captures(remaining)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Find the next function name after a position (for Python)
    fn find_next_python_function(content: &str, pos: usize) -> Option<String> {
        let remaining = &content[pos..];
        let fn_re = Regex::new(r#"(?:async\s+)?def\s+(\w+)"#).unwrap();
        fn_re
            .captures(remaining)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Find clap about/description near a position
    fn find_clap_about(content: &str, pos: usize) -> Option<String> {
        let start = pos.saturating_sub(200);
        let end = (pos + 200).min(content.len());
        let context = &content[start..end];

        let about_re = Regex::new(r#"about\s*=\s*["']([^"']+)["']"#).unwrap();
        about_re
            .captures(context)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Generate observations about the API structure
    fn generate_observations(&self, analysis: &ApiAnalysis) -> Vec<String> {
        let mut observations = Vec::new();

        // Endpoint observations
        if analysis.endpoints.is_empty() {
            observations.push("No HTTP endpoints detected.".to_string());
        } else {
            observations.push(format!(
                "Found {} HTTP endpoint(s) across {} framework(s).",
                analysis.endpoints.len(),
                analysis.frameworks.len().max(1)
            ));

            // Check for REST patterns
            let has_crud = analysis
                .endpoints
                .iter()
                .any(|e| e.method == HttpMethod::Get)
                && analysis
                    .endpoints
                    .iter()
                    .any(|e| e.method == HttpMethod::Post)
                && (analysis
                    .endpoints
                    .iter()
                    .any(|e| e.method == HttpMethod::Put)
                    || analysis
                        .endpoints
                        .iter()
                        .any(|e| e.method == HttpMethod::Patch))
                && analysis
                    .endpoints
                    .iter()
                    .any(|e| e.method == HttpMethod::Delete);

            if has_crud {
                observations.push("RESTful CRUD pattern detected.".to_string());
            }
        }

        // CLI observations
        if !analysis.commands.is_empty() {
            observations.push(format!("Found {} CLI command(s).", analysis.commands.len()));
        }

        // MCP observations
        if !analysis.mcp_tools.is_empty() {
            observations.push(format!("Found {} MCP tool(s).", analysis.mcp_tools.len()));
        }

        // Total API surface
        if analysis.total_api_surfaces == 0 {
            observations.push(
                "No public API surfaces detected. This may be a library or internal module."
                    .to_string(),
            );
        }

        observations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_http_method_default() {
        assert_eq!(HttpMethod::default(), HttpMethod::Get);
    }

    #[test]
    fn test_http_method_display() {
        assert_eq!(format!("{}", HttpMethod::Get), "GET");
        assert_eq!(format!("{}", HttpMethod::Post), "POST");
        assert_eq!(format!("{}", HttpMethod::Put), "PUT");
        assert_eq!(format!("{}", HttpMethod::Patch), "PATCH");
        assert_eq!(format!("{}", HttpMethod::Delete), "DELETE");
    }

    #[test]
    fn test_api_framework_default() {
        assert_eq!(ApiFramework::default(), ApiFramework::Unknown);
    }

    #[test]
    fn test_api_framework_display() {
        assert_eq!(format!("{}", ApiFramework::Axum), "axum");
        assert_eq!(format!("{}", ApiFramework::ActixWeb), "actix-web");
        assert_eq!(format!("{}", ApiFramework::Express), "express");
        assert_eq!(format!("{}", ApiFramework::FastApi), "fastapi");
    }

    #[test]
    fn test_api_inventory_new() {
        let inventory = ApiInventory::new(PathBuf::from("/test"));
        assert_eq!(inventory.root(), &PathBuf::from("/test"));
    }

    #[test]
    fn test_parse_http_method() {
        assert_eq!(ApiInventory::parse_http_method("get"), HttpMethod::Get);
        assert_eq!(ApiInventory::parse_http_method("GET"), HttpMethod::Get);
        assert_eq!(ApiInventory::parse_http_method("post"), HttpMethod::Post);
        assert_eq!(ApiInventory::parse_http_method("PUT"), HttpMethod::Put);
        assert_eq!(ApiInventory::parse_http_method("patch"), HttpMethod::Patch);
        assert_eq!(
            ApiInventory::parse_http_method("delete"),
            HttpMethod::Delete
        );
    }

    #[test]
    fn test_find_line_number() {
        let content = "line1\nline2\nline3";
        assert_eq!(ApiInventory::find_line_number(content, 0), 1);
        assert_eq!(ApiInventory::find_line_number(content, 6), 2);
        assert_eq!(ApiInventory::find_line_number(content, 12), 3);
    }

    #[test]
    fn test_analyze_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let inventory = ApiInventory::new(temp_dir.path().to_path_buf());
        let analysis = inventory.analyze().unwrap();

        assert!(analysis.endpoints.is_empty());
        assert!(analysis.commands.is_empty());
        assert!(analysis.mcp_tools.is_empty());
        assert_eq!(analysis.total_api_surfaces, 0);
    }

    #[test]
    fn test_detect_axum_endpoints() {
        let temp_dir = TempDir::new().unwrap();
        let src = temp_dir.path().join("src");
        fs::create_dir(&src).unwrap();

        fs::write(
            src.join("main.rs"),
            r#"
use axum::{routing::get, Router};

async fn get_users() -> String {
    "users".to_string()
}

async fn create_user() -> String {
    "created".to_string()
}

fn main() {
    let app = Router::new()
        .route("/users", get(get_users))
        .route("/users", post(create_user))
        .route("/users/:id", get(get_user_by_id))
        .route("/users/:id", delete(delete_user));
}
"#,
        )
        .unwrap();

        let inventory = ApiInventory::new(temp_dir.path().to_path_buf());
        let analysis = inventory.analyze().unwrap();

        assert!(!analysis.endpoints.is_empty());
        assert!(analysis.endpoints.iter().any(|e| e.path == "/users"));
        assert!(analysis.endpoints.iter().any(|e| e.path == "/users/:id"));
        assert!(analysis.frameworks.contains(&ApiFramework::Axum));
    }

    #[test]
    fn test_detect_actix_endpoints() {
        let temp_dir = TempDir::new().unwrap();
        let src = temp_dir.path().join("src");
        fs::create_dir(&src).unwrap();

        fs::write(
            src.join("main.rs"),
            r#"
use actix_web::{web, App, HttpServer, get, post};

#[get("/api/users")]
async fn get_users() -> impl Responder {
    "users"
}

#[post("/api/users")]
async fn create_user() -> impl Responder {
    "created"
}

fn main() {
    HttpServer::new(|| App::new().service(get_users).service(create_user))
}
"#,
        )
        .unwrap();

        let inventory = ApiInventory::new(temp_dir.path().to_path_buf());
        let analysis = inventory.analyze().unwrap();

        assert!(!analysis.endpoints.is_empty());
        assert!(analysis.endpoints.iter().any(|e| e.path == "/api/users"));
        assert!(analysis
            .endpoints
            .iter()
            .any(|e| e.framework == ApiFramework::ActixWeb));
    }

    #[test]
    fn test_detect_express_endpoints() {
        let temp_dir = TempDir::new().unwrap();
        let src = temp_dir.path().join("src");
        fs::create_dir(&src).unwrap();

        fs::write(
            src.join("app.ts"),
            r#"
import express from 'express';

const app = express();

app.get('/api/users', (req, res) => {
    res.json([]);
});

app.post('/api/users', (req, res) => {
    res.json({ created: true });
});

app.delete('/api/users/:id', (req, res) => {
    res.json({ deleted: true });
});
"#,
        )
        .unwrap();

        let inventory = ApiInventory::new(temp_dir.path().to_path_buf());
        let analysis = inventory.analyze().unwrap();

        assert!(!analysis.endpoints.is_empty());
        assert!(analysis.endpoints.iter().any(|e| e.path == "/api/users"));
        assert!(analysis
            .endpoints
            .iter()
            .any(|e| e.path == "/api/users/:id"));
        assert!(analysis
            .endpoints
            .iter()
            .any(|e| e.framework == ApiFramework::Express));
    }

    #[test]
    fn test_detect_fastapi_endpoints() {
        let temp_dir = TempDir::new().unwrap();
        let src = temp_dir.path().join("src");
        fs::create_dir(&src).unwrap();

        fs::write(
            src.join("main.py"),
            r#"
from fastapi import FastAPI

app = FastAPI()

@app.get("/items")
async def get_items():
    return []

@app.post("/items")
async def create_item():
    return {"created": True}

@router.delete("/items/{item_id}")
async def delete_item(item_id: int):
    return {"deleted": True}
"#,
        )
        .unwrap();

        let inventory = ApiInventory::new(temp_dir.path().to_path_buf());
        let analysis = inventory.analyze().unwrap();

        assert!(!analysis.endpoints.is_empty());
        assert!(analysis.endpoints.iter().any(|e| e.path == "/items"));
        assert!(analysis
            .endpoints
            .iter()
            .any(|e| e.path == "/items/{item_id}"));
    }

    #[test]
    fn test_detect_flask_endpoints() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(
            temp_dir.path().join("app.py"),
            r#"
from flask import Flask

app = Flask(__name__)

@app.route('/users', methods=['GET'])
def get_users():
    return []

@app.route('/users', methods=['POST'])
def create_user():
    return {'created': True}
"#,
        )
        .unwrap();

        let inventory = ApiInventory::new(temp_dir.path().to_path_buf());
        let analysis = inventory.analyze().unwrap();

        assert!(!analysis.endpoints.is_empty());
        assert!(analysis.endpoints.iter().any(|e| e.path == "/users"));
        assert!(analysis
            .endpoints
            .iter()
            .any(|e| e.framework == ApiFramework::Flask));
    }

    #[test]
    fn test_detect_gin_endpoints() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(
            temp_dir.path().join("main.go"),
            r#"
package main

import "github.com/gin-gonic/gin"

func main() {
    r := gin.Default()
    r.GET("/users", getUsers)
    r.POST("/users", createUser)
    r.DELETE("/users/:id", deleteUser)
}
"#,
        )
        .unwrap();

        let inventory = ApiInventory::new(temp_dir.path().to_path_buf());
        let analysis = inventory.analyze().unwrap();

        assert!(!analysis.endpoints.is_empty());
        assert!(analysis.endpoints.iter().any(|e| e.path == "/users"));
        assert!(analysis.endpoints.iter().any(|e| e.path == "/users/:id"));
        assert!(analysis
            .endpoints
            .iter()
            .any(|e| e.framework == ApiFramework::Gin));
    }

    #[test]
    fn test_detect_clap_commands() {
        let temp_dir = TempDir::new().unwrap();
        let src = temp_dir.path().join("src");
        fs::create_dir(&src).unwrap();

        fs::write(
            src.join("main.rs"),
            r#"
use clap::{Command, Parser};

#[derive(Parser)]
#[command(name = "myapp", about = "My application")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        name: String,
    },
    Build,
    Run,
}

fn main() {
    let cmd = Command::new("myapp")
        .subcommand(Command::new("init"))
        .subcommand(Command::new("build"));
}
"#,
        )
        .unwrap();

        let inventory = ApiInventory::new(temp_dir.path().to_path_buf());
        let analysis = inventory.analyze().unwrap();

        assert!(!analysis.commands.is_empty());
        assert!(analysis.commands.iter().any(|c| c.name == "myapp"));
        assert!(analysis.commands.iter().any(|c| c.framework == "clap"));
    }

    #[test]
    fn test_detect_click_commands() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(
            temp_dir.path().join("cli.py"),
            r#"
import click

@click.group()
def cli():
    pass

@click.command()
def init():
    pass

@click.command()
def build():
    pass
"#,
        )
        .unwrap();

        let inventory = ApiInventory::new(temp_dir.path().to_path_buf());
        let analysis = inventory.analyze().unwrap();

        assert!(!analysis.commands.is_empty());
        assert!(analysis.commands.iter().any(|c| c.framework == "click"));
    }

    #[test]
    fn test_detect_cobra_commands() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(
            temp_dir.path().join("main.go"),
            r#"
package main

import "github.com/spf13/cobra"

var rootCmd = &cobra.Command{
    Use:   "myapp",
    Short: "My application",
}

var initCmd = &cobra.Command{
    Use:   "init",
    Short: "Initialize the project",
}

func main() {
    rootCmd.AddCommand(initCmd)
    rootCmd.Execute()
}
"#,
        )
        .unwrap();

        let inventory = ApiInventory::new(temp_dir.path().to_path_buf());
        let analysis = inventory.analyze().unwrap();

        assert!(!analysis.commands.is_empty());
        assert!(analysis.commands.iter().any(|c| c.name == "myapp"));
        assert!(analysis.commands.iter().any(|c| c.name == "init"));
        assert!(analysis.commands.iter().any(|c| c.framework == "cobra"));
    }

    #[test]
    fn test_detect_mcp_tools_typescript() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(
            temp_dir.path().join("tools.ts"),
            r#"
import { McpServer } from '@modelcontextprotocol/sdk';

const server = new McpServer();

server.tool("read_file", async (params) => {
    return { content: "file contents" };
});

server.tool("write_file", async (params) => {
    return { success: true };
});

const tools = [
    {
        name: "search_code",
        description: "Search for code patterns",
        inputSchema: { type: "object" }
    }
];
"#,
        )
        .unwrap();

        let inventory = ApiInventory::new(temp_dir.path().to_path_buf());
        let analysis = inventory.analyze().unwrap();

        assert!(!analysis.mcp_tools.is_empty());
        assert!(analysis.mcp_tools.iter().any(|t| t.name == "read_file"));
        assert!(analysis.mcp_tools.iter().any(|t| t.name == "write_file"));
        assert!(analysis.mcp_tools.iter().any(|t| t.name == "search_code"));
    }

    #[test]
    fn test_detect_mcp_tools_python() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(
            temp_dir.path().join("tools.py"),
            r#"
from mcp import Tool

@tool(name="read_file")
async def read_file(path: str) -> str:
    return "contents"

file_tool = Tool(name="write_file", description="Write to a file")
"#,
        )
        .unwrap();

        let inventory = ApiInventory::new(temp_dir.path().to_path_buf());
        let analysis = inventory.analyze().unwrap();

        assert!(!analysis.mcp_tools.is_empty());
        assert!(analysis.mcp_tools.iter().any(|t| t.name == "read_file"));
        assert!(analysis.mcp_tools.iter().any(|t| t.name == "write_file"));
    }

    #[test]
    fn test_detect_mcp_tools_rust() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(
            temp_dir.path().join("tools.rs"),
            r#"
use mcp::Tool;

let read_tool = Tool::new("read_file");
let write_tool = Tool::new("write_file");

#[mcp_tool(name = "search")]
async fn search_code() {}
"#,
        )
        .unwrap();

        let inventory = ApiInventory::new(temp_dir.path().to_path_buf());
        let analysis = inventory.analyze().unwrap();

        assert!(!analysis.mcp_tools.is_empty());
        assert!(analysis.mcp_tools.iter().any(|t| t.name == "read_file"));
        assert!(analysis.mcp_tools.iter().any(|t| t.name == "write_file"));
        assert!(analysis.mcp_tools.iter().any(|t| t.name == "search"));
    }

    #[test]
    fn test_api_analysis_total_surfaces() {
        let temp_dir = TempDir::new().unwrap();
        let src = temp_dir.path().join("src");
        fs::create_dir(&src).unwrap();

        fs::write(
            src.join("main.rs"),
            r#"
use axum::{routing::get, Router};
use clap::Command;
use mcp::Tool;

fn main() {
    let app = Router::new()
        .route("/users", get(get_users));

    let cmd = Command::new("myapp");

    let tool = Tool::new("my_tool");
}
"#,
        )
        .unwrap();

        let inventory = ApiInventory::new(temp_dir.path().to_path_buf());
        let analysis = inventory.analyze().unwrap();

        assert_eq!(
            analysis.total_api_surfaces,
            analysis.endpoints.len() + analysis.commands.len() + analysis.mcp_tools.len()
        );
    }

    #[test]
    fn test_generate_observations_restful() {
        let temp_dir = TempDir::new().unwrap();
        let src = temp_dir.path().join("src");
        fs::create_dir(&src).unwrap();

        fs::write(
            src.join("main.rs"),
            r#"
use axum::{routing::{get, post, put, delete}, Router};

fn main() {
    let app = Router::new()
        .route("/users", get(list))
        .route("/users", post(create))
        .route("/users/:id", put(update))
        .route("/users/:id", delete(remove));
}
"#,
        )
        .unwrap();

        let inventory = ApiInventory::new(temp_dir.path().to_path_buf());
        let analysis = inventory.analyze().unwrap();

        assert!(analysis
            .observations
            .iter()
            .any(|o| o.contains("RESTful CRUD")));
    }

    #[test]
    fn test_api_analysis_serialization() {
        let analysis = ApiAnalysis {
            endpoints: vec![HttpEndpoint {
                method: HttpMethod::Get,
                path: "/users".to_string(),
                handler: Some("get_users".to_string()),
                file: PathBuf::from("src/main.rs"),
                line: Some(10),
                framework: ApiFramework::Axum,
            }],
            commands: vec![CliCommand {
                name: "myapp".to_string(),
                description: Some("My application".to_string()),
                subcommands: vec!["init".to_string()],
                file: PathBuf::from("src/main.rs"),
                line: Some(5),
                framework: "clap".to_string(),
            }],
            mcp_tools: vec![McpTool {
                name: "read_file".to_string(),
                description: Some("Read a file".to_string()),
                inputs: vec!["path".to_string()],
                file: PathBuf::from("src/tools.rs"),
                line: Some(15),
            }],
            frameworks: vec![ApiFramework::Axum],
            total_api_surfaces: 3,
            observations: vec!["Found 1 endpoint".to_string()],
        };

        let json = serde_json::to_string(&analysis).unwrap();
        let deserialized: ApiAnalysis = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.endpoints.len(), 1);
        assert_eq!(deserialized.commands.len(), 1);
        assert_eq!(deserialized.mcp_tools.len(), 1);
        assert_eq!(deserialized.total_api_surfaces, 3);
    }

    #[test]
    fn test_find_next_function_name() {
        let content = r#"
#[get("/users")]
pub async fn get_users() -> String {
    "users".to_string()
}
"#;

        let pos = content.find("#[get").unwrap();
        let handler = ApiInventory::find_next_function_name(content, pos + 15);
        assert_eq!(handler, Some("get_users".to_string()));
    }

    #[test]
    fn test_find_next_python_function() {
        let content = r#"
@app.get("/users")
async def get_users():
    return []
"#;

        let pos = content.find("@app.get").unwrap();
        let handler = ApiInventory::find_next_python_function(content, pos + 20);
        assert_eq!(handler, Some("get_users".to_string()));
    }

    #[test]
    fn test_hono_endpoints() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(
            temp_dir.path().join("app.ts"),
            r#"
import { Hono } from 'hono';

const app = new Hono();

app.get('/api/items', (c) => c.json([]));
app.post('/api/items', (c) => c.json({ created: true }));
"#,
        )
        .unwrap();

        let inventory = ApiInventory::new(temp_dir.path().to_path_buf());
        let analysis = inventory.analyze().unwrap();

        assert!(!analysis.endpoints.is_empty());
        assert!(analysis
            .endpoints
            .iter()
            .any(|e| e.framework == ApiFramework::Hono));
    }

    #[test]
    fn test_commander_js_commands() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(
            temp_dir.path().join("cli.js"),
            r#"
const { program } = require('commander');

program
    .command('init')
    .description('Initialize project')
    .action(() => {});

program
    .command('build')
    .description('Build project')
    .action(() => {});
"#,
        )
        .unwrap();

        let inventory = ApiInventory::new(temp_dir.path().to_path_buf());
        let analysis = inventory.analyze().unwrap();

        assert!(!analysis.commands.is_empty());
        assert!(analysis.commands.iter().any(|c| c.name == "init"));
        assert!(analysis.commands.iter().any(|c| c.name == "build"));
    }

    #[test]
    fn test_no_api_surfaces_observation() {
        let temp_dir = TempDir::new().unwrap();

        // Create a Rust file with no API surfaces
        fs::write(
            temp_dir.path().join("lib.rs"),
            r#"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#,
        )
        .unwrap();

        let inventory = ApiInventory::new(temp_dir.path().to_path_buf());
        let analysis = inventory.analyze().unwrap();

        assert!(analysis
            .observations
            .iter()
            .any(|o| o.contains("No public API surfaces")));
    }
}
