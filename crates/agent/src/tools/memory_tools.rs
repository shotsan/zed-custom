use crate::memory_store::{Memory, MemoryCategory, MemoryStore};
use agent_client_protocol as acp;
use anyhow::Result;
use chrono::Utc;
use gpui::{App, SharedString, Task};
use language_model::{
    LanguageModelToolResultContent, LanguageModelToolSchemaFormat,
};
use schemars::{JsonSchema, Schema};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

pub struct RememberTool {
    memory_store: Arc<parking_lot::RwLock<Option<Arc<MemoryStore>>>>,
}

impl RememberTool {
    pub fn new() -> Self {
        Self {
            memory_store: Arc::new(parking_lot::RwLock::new(None)),
        }
    }

    pub fn set_store(&self, store: Arc<MemoryStore>) {
        *self.memory_store.write() = Some(store);
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RememberToolInput {
    /// The information to remember
    pub content: String,
    /// Category of the memory
    pub category: RememberCategory,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RememberCategory {
    Architecture,
    Patterns,
    Issues,
    Procedures,
    Notes,
}

impl From<RememberCategory> for MemoryCategory {
    fn from(cat: RememberCategory) -> Self {
        match cat {
            RememberCategory::Architecture => MemoryCategory::Architecture,
            RememberCategory::Patterns => MemoryCategory::Patterns,
            RememberCategory::Issues => MemoryCategory::Issues,
            RememberCategory::Procedures => MemoryCategory::Procedures,
            RememberCategory::Notes => MemoryCategory::Notes,
        }
    }
}

impl crate::tools::AgentTool for RememberTool {
    type Input = RememberToolInput;
    type Output = LanguageModelToolResultContent;

    fn name() -> &'static str {
        "remember"
    }

    fn description() -> SharedString {
        "Store important information for future sessions. Use this to remember project architecture, patterns, known issues, procedures, or general notes.".into()
    }

    fn kind() -> acp::ToolKind {
        acp::ToolKind::Other
    }

    fn initial_title(
        &self,
        input: Result<Self::Input, serde_json::Value>,
        _cx: &mut App,
    ) -> SharedString {
        match input {
            Ok(_) => "Remembering...".into(),
            Err(_) => "Remembering".into(),
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
        let memory_store = self.memory_store.clone();

        Task::ready(Ok({
            log::info!("=== RememberTool::run called ===");
            log::info!("Input category: {:?}", input.category);
            log::info!("Input content: {}", input.content);
            
            let store_guard = memory_store.read();
            if let Some(store) = store_guard.as_ref() {
                log::info!("Memory store is available");
                
                let memory = Memory {
                    id: Uuid::new_v4(),
                    category: input.category.into(),
                    content: input.content,
                    metadata: serde_json::Value::Null,
                    created_at: Utc::now(),
                    last_accessed: Utc::now(),
                };

                log::info!("Created memory object with id: {}", memory.id);
                log::info!("Calling store.remember() and detaching task...");
                
                // Detach the task so it completes even after the tool returns.
                // In GPUI, dropping a Task handle cancels the operation.
                store.remember(memory).detach_and_log_err(cx);
                
                log::info!("Task detached successfully");

                LanguageModelToolResultContent::Text(
                    "Memory stored successfully. This information will be available in future sessions.".into()
                )
            } else {
                log::error!("Memory store not initialized!");
                LanguageModelToolResultContent::Text(
                    "Error: Memory store not initialized".into()
                )
            }
        }))
    }
}

pub struct RecallTool {
    memory_store: Arc<parking_lot::RwLock<Option<Arc<MemoryStore>>>>,
}

impl RecallTool {
    pub fn new() -> Self {
        Self {
            memory_store: Arc::new(parking_lot::RwLock::new(None)),
        }
    }

    pub fn set_store(&self, store: Arc<MemoryStore>) {
        *self.memory_store.write() = Some(store);
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RecallToolInput {
    /// Optional search query to filter memories
    pub query: Option<String>,
    /// Optional category filter
    pub category: Option<RememberCategory>,
    /// Maximum number of memories to recall (default: 5)
    #[serde(default = "default_recall_limit")]
    pub limit: usize,
}

fn default_recall_limit() -> usize {
    5
}

impl crate::tools::AgentTool for RecallTool {
    type Input = RecallToolInput;
    type Output = LanguageModelToolResultContent;

    fn name() -> &'static str {
        "recall"
    }

    fn description() -> SharedString {
        "Retrieve previously stored information from past sessions. Use this to recall project architecture, patterns, known issues, procedures, or notes.".into()
    }

    fn kind() -> acp::ToolKind {
        acp::ToolKind::Other
    }

    fn initial_title(
        &self,
        input: Result<Self::Input, serde_json::Value>,
        _cx: &mut App,
    ) -> SharedString {
        match input {
            Ok(input) => {
                if let Some(query) = &input.query {
                    format!("Recalling '{}'", query).into()
                } else {
                    "Recalling memories".into()
                }
            }
            Err(_) => "Recalling".into(),
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
        let memory_store = self.memory_store.clone();

        cx.spawn(async move |_cx| {
            let store = memory_store.read().clone();
            let Some(store) = store else {
                return Ok(LanguageModelToolResultContent::Text(
                    "Error: Memory store not initialized".into(),
                ));
            };

            let memories = store.recall(input.query, input.category.map(Into::into), input.limit).await?;
            
            if memories.is_empty() {
                return Ok(LanguageModelToolResultContent::Text(
                    "No matching memories found.".into(),
                ));
            }

            let mut response = String::from("Recalled memories:\n\n");
            for memory in memories {
                let category = format!("{:?}", memory.category);
                let date = memory.created_at.format("%Y-%m-%d");
                response.push_str(&format!("### {} ({})\n{}\n\n", category, date, memory.content));
            }

            Ok(LanguageModelToolResultContent::Text(response.into()))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_conversion() {
        let cat = RememberCategory::Architecture;
        let mem_cat: MemoryCategory = cat.into();
        assert_eq!(mem_cat, MemoryCategory::Architecture);
    }
}
