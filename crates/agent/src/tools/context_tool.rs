use agent_client_protocol as acp;
use anyhow::{Context, Result};
use gpui::{App, AsyncApp, Entity, SharedString, Task};
use project::{Project, Symbol};
use language_model::{LanguageModelToolResultContent, LanguageModelToolSchemaFormat};
use schemars::{JsonSchema, Schema};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct ContextTool {
    project: Entity<Project>,
}

impl ContextTool {
    pub fn new(project: Entity<Project>) -> Self {
        Self { project }
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ContextToolInput {
    /// The search query for symbols (function names, class names, etc.)
    pub query: String,
}

impl crate::tools::AgentTool for ContextTool {
    type Input = ContextToolInput;
    type Output = LanguageModelToolResultContent;

    fn name() -> &'static str {
        "query_context"
    }

    fn description() -> SharedString {
        "Search the codebase for relevant symbols, functions, classes, or other code definitions using the project's language servers. Use this to find code definitions.".into()
    }

    fn kind() -> acp::ToolKind {
        acp::ToolKind::Search
    }

    fn initial_title(
        &self,
        input: Result<Self::Input, serde_json::Value>,
        _cx: &mut App,
    ) -> SharedString {
        match input {
            Ok(input) => format!("Searching for '{}'", input.query).into(),
            Err(_) => "Searching...".into(),
        }
    }

    fn input_schema(format: LanguageModelToolSchemaFormat) -> Schema {
        language_model::tool_schema::root_schema_for::<Self::Input>(format)
    }

    fn run(
        self: Arc<Self>,
        input: Self::Input,
        _event_stream: crate::thread::ToolCallEventStream,
        cx: &mut App,
    ) -> Task<Result<Self::Output, anyhow::Error>> {
        let project = self.project.clone();
        let query = input.query.clone();

        cx.spawn(|cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
            let task: Task<Result<Vec<Symbol>>> = cx.update(|cx| {
                project.update(cx, |project, cx| {
                    project.symbols(&query, cx)
                })
            });
            
            let symbols = task.await.context("Failed to search symbols")?;

            let content = if symbols.is_empty() {
                format!("No symbols found matching '{}'", query)
            } else {
                format_search_results(&symbols)
            };

            Ok(LanguageModelToolResultContent::Text(content.into()))
            }
        })
    }
}

fn format_search_results(results: &[Symbol]) -> String {
    let mut output = String::from("Found the following symbols:\n\n");

    // Take top 20 results to avoid overflowing context
    for (i, symbol) in results.iter().take(20).enumerate() {
        output.push_str(&format!(
            "{}. **{}** ({:?})\n",
            i + 1,
            symbol.label.text,
            symbol.kind
        ));
        
        match &symbol.path {
            project::lsp_store::SymbolLocation::InProject(project_path) => {
                 output.push_str(&format!("   File: `{:?}`\n", project_path.path));
            }
            project::lsp_store::SymbolLocation::OutsideProject { abs_path, .. } => {
                 output.push_str(&format!("   File: `{:?}`\n", abs_path));
            }
        }
        
        output.push_str(&format!(
            "   Lines: {}-{}\n",
            symbol.range.start.0.row + 1,
            symbol.range.end.0.row + 1
        ));
        output.push('\n');
    }

    output
}
