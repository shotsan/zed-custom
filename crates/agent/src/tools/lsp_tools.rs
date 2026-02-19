use crate::{AgentTool, AnyAgentTool, Thread, ThreadsDatabase, ToolCallEventStream};
use agent_client_protocol as acp;
use anyhow::{Result, anyhow};
use futures::FutureExt;
use futures::future::Shared;
use gpui::{App, Entity, Task, WeakEntity};
use language::{Bias, PointUtf16};
use text::ToPointUtf16;
use project::Project;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use ui::SharedString;
use util::markdown::MarkdownInlineCode;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct LspSymbolInput {
    /// The path of the file containing the symbol.
    pub path: String,
    /// The 1-based line number of the symbol.
    pub line: u32,
    /// The 1-based column number of the symbol.
    pub column: u32,
}

pub struct LspGetDefinitionTool {
    project: Entity<Project>,
}

impl LspGetDefinitionTool {
    pub fn new(project: Entity<Project>) -> Self {
        Self { project }
    }
}

impl AgentTool for LspGetDefinitionTool {
    type Input = LspSymbolInput;
    type Output = String;

    fn name() -> &'static str {
        "lsp_get_definition"
    }

    fn description() -> SharedString {
        "Get the definition(s) of a symbol at a specific location using LSP.".into()
    }

    fn kind() -> acp::ToolKind {
        acp::ToolKind::Read
    }

    fn initial_title(
        &self,
        input: Result<Self::Input, serde_json::Value>,
        _cx: &mut App,
    ) -> SharedString {
        if let Ok(input) = input {
            format!("Go to definition in {}", MarkdownInlineCode(&input.path)).into()
        } else {
            "Go to definition".into()
        }
    }

    fn run(
        self: Arc<Self>,
        input: Self::Input,
        event_stream: ToolCallEventStream,
        cx: &mut App,
    ) -> Task<Result<Self::Output, anyhow::Error>> {
        let project = self.project.clone();
        cx.spawn(async move |cx| -> Result<Self::Output> {
            let (buffer_task, _) = project.update(cx, |project, cx| {
                let project_path = project.find_project_path(&input.path, cx).ok_or_else(|| anyhow!("Could not find path {} in project", input.path))?;
                let buffer_task = project.open_buffer(project_path, cx);
                anyhow::Ok((buffer_task, ()))
            })?;
            let buffer: Entity<language::Buffer> = buffer_task.await?;

            let position = buffer.read_with(cx, |buffer, _| {
                buffer.snapshot().clip_point_utf16(text::Unclipped(PointUtf16::new(input.line.saturating_sub(1), input.column.saturating_sub(1))), Bias::Left)
            });

            let definitions_task: Task<Result<Option<Vec<project::LocationLink>>>> = project.update(cx, |project, cx| {
                project.definitions(&buffer, position, cx)
            });

            let result = futures::select! {
                result = definitions_task.fuse() => result?,
                _ = event_stream.cancelled_by_user().fuse() => {
                    anyhow::bail!("LSP request cancelled by user");
                }
            };

            let Some(locations) = result else {
                return Ok("No definitions found.".to_string());
            };

            let mut output = String::new();
            for link in locations {
                let loc = link.target;
                let path = project.read_with(cx, |project, cx| {
                    let buffer = loc.buffer.read(cx);
                    let file = buffer.file()?;
                    let project_path = project::ProjectPath {
                        worktree_id: file.worktree_id(cx),
                        path: file.path().clone(),
                    };
                    let worktree = project.worktree_for_id(project_path.worktree_id, cx)?;
                    Some(worktree.read(cx).absolutize(&project_path.path).display().to_string())
                }).unwrap_or_else(|| "unknown".to_string());
                
                let range = loc.buffer.read_with(cx, |buffer, _| {
                    let snapshot = buffer.snapshot();
                    let start = loc.range.start.to_point_utf16(&snapshot);
                    (start.row + 1, start.column + 1)
                });

                output.push_str(&format!("- {}: line {}, column {}\n", path, range.0, range.1));
            }

            Ok(output)
        })
    }
}

pub struct LspFindReferencesTool {
    project: Entity<Project>,
}

impl LspFindReferencesTool {
    pub fn new(project: Entity<Project>) -> Self {
        Self { project }
    }
}

impl AgentTool for LspFindReferencesTool {
    type Input = LspSymbolInput;
    type Output = String;

    fn name() -> &'static str {
        "lsp_find_references"
    }

    fn description() -> SharedString {
        "Find all references to a symbol at a specific location using LSP.".into()
    }

    fn kind() -> acp::ToolKind {
        acp::ToolKind::Read
    }

    fn initial_title(
        &self,
        input: Result<Self::Input, serde_json::Value>,
        _cx: &mut App,
    ) -> SharedString {
        if let Ok(input) = input {
            format!("Find references in {}", MarkdownInlineCode(&input.path)).into()
        } else {
            "Find references".into()
        }
    }

    fn run(
        self: Arc<Self>,
        input: Self::Input,
        event_stream: ToolCallEventStream,
        cx: &mut App,
    ) -> Task<Result<Self::Output, anyhow::Error>> {
        let project = self.project.clone();
        cx.spawn(async move |cx| -> Result<Self::Output> {
            let (buffer_task, _) = project.update(cx, |project, cx| {
                let project_path = project.find_project_path(&input.path, cx).ok_or_else(|| anyhow!("Could not find path {} in project", input.path))?;
                let buffer_task = project.open_buffer(project_path, cx);
                anyhow::Ok((buffer_task, ()))
            })?;
            let buffer: Entity<language::Buffer> = buffer_task.await?;

            let position = buffer.read_with(cx, |buffer, _| {
                buffer.snapshot().clip_point_utf16(text::Unclipped(PointUtf16::new(input.line.saturating_sub(1), input.column.saturating_sub(1))), Bias::Left)
            });

            let references_task: Task<Result<Option<Vec<project::Location>>>> = project.update(cx, |project, cx| {
                project.references(&buffer, position, cx)
            });

            let result = futures::select! {
                result = references_task.fuse() => result?,
                _ = event_stream.cancelled_by_user().fuse() => {
                    anyhow::bail!("LSP request cancelled by user");
                }
            };

            let Some(locations) = result else {
                return Ok("No references found.".to_string());
            };

            let mut output = String::new();
            for loc in locations {
                let path = project.read_with(cx, |project, cx| {
                    let buffer = loc.buffer.read(cx);
                    let file = buffer.file()?;
                    let project_path = project::ProjectPath {
                        worktree_id: file.worktree_id(cx),
                        path: file.path().clone(),
                    };
                    let worktree = project.worktree_for_id(project_path.worktree_id, cx)?;
                    Some(worktree.read(cx).absolutize(&project_path.path).display().to_string())
                }).unwrap_or_else(|| "unknown".to_string());

                 let range = loc.buffer.read_with(cx, |buffer, _| {
                    let snapshot = buffer.snapshot();
                    let start = loc.range.start.to_point_utf16(&snapshot);
                    (start.row + 1, start.column + 1)
                });

                 output.push_str(&format!("- {}: line {}, column {}\n", path, range.0, range.1));
            }

            Ok(output)
        })
    }
}

pub struct LspGetImplementationsTool {
    project: Entity<Project>,
}

impl LspGetImplementationsTool {
    pub fn new(project: Entity<Project>) -> Self {
        Self { project }
    }
}

impl AgentTool for LspGetImplementationsTool {
    type Input = LspSymbolInput;
    type Output = String;

    fn name() -> &'static str {
        "lsp_get_implementations"
    }

    fn description() -> SharedString {
        "Get implementations of a symbol (e.g., trait or interface) at a specific location using LSP.".into()
    }

    fn kind() -> acp::ToolKind {
        acp::ToolKind::Read
    }

    fn initial_title(
        &self,
        input: Result<Self::Input, serde_json::Value>,
        _cx: &mut App,
    ) -> SharedString {
        if let Ok(input) = input {
            format!("Get implementations in {}", MarkdownInlineCode(&input.path)).into()
        } else {
            "Get implementations".into()
        }
    }

    fn run(
        self: Arc<Self>,
        input: Self::Input,
        event_stream: ToolCallEventStream,
        cx: &mut App,
    ) -> Task<Result<Self::Output, anyhow::Error>> {
        let project = self.project.clone();
        cx.spawn(async move |cx| -> Result<Self::Output> {
            let (buffer_task, _) = project.update(cx, |project, cx| {
                let project_path = project.find_project_path(&input.path, cx).ok_or_else(|| anyhow!("Could not find path {} in project", input.path))?;
                let buffer_task = project.open_buffer(project_path, cx);
                anyhow::Ok((buffer_task, ()))
            })?;
            let buffer: Entity<language::Buffer> = buffer_task.await?;

            let position = buffer.read_with(cx, |buffer, _| {
                buffer.snapshot().clip_point_utf16(text::Unclipped(PointUtf16::new(input.line.saturating_sub(1), input.column.saturating_sub(1))), Bias::Left)
            });

            let implementations_task: Task<Result<Option<Vec<project::LocationLink>>>> = project.update(cx, |project, cx| {
                project.implementations(&buffer, position, cx)
            });

            let result = futures::select! {
                result = implementations_task.fuse() => result?,
                _ = event_stream.cancelled_by_user().fuse() => {
                    anyhow::bail!("LSP request cancelled by user");
                }
            };

            let Some(locations) = result else {
                return Ok("No implementations found.".to_string());
            };

            let mut output = String::new();
            for link in locations {
                let loc = link.target;
                let path = project.read_with(cx, |project, cx| {
                    let buffer = loc.buffer.read(cx);
                    let file = buffer.file()?;
                    let project_path = project::ProjectPath {
                        worktree_id: file.worktree_id(cx),
                        path: file.path().clone(),
                    };
                    let worktree = project.worktree_for_id(project_path.worktree_id, cx)?;
                    Some(worktree.read(cx).absolutize(&project_path.path).display().to_string())
                }).unwrap_or_else(|| "unknown".to_string());

                let range = loc.buffer.read_with(cx, |buffer, _| {
                    let snapshot = buffer.snapshot();
                    let start = loc.range.start.to_point_utf16(&snapshot);
                    (start.row + 1, start.column + 1)
                });

                output.push_str(&format!("- {}: line {}, column {}\n", path, range.0, range.1));
            }

            Ok(output)
        })
    }
}

pub struct SaveReflectionTool {
    pub(crate) thread: WeakEntity<Thread>,
    pub(crate) database: Shared<Task<Result<Arc<ThreadsDatabase>, Arc<anyhow::Error>>>>,
}

impl SaveReflectionTool {
    pub fn new(
        thread: WeakEntity<Thread>,
        database: Shared<Task<Result<Arc<ThreadsDatabase>, Arc<anyhow::Error>>>>,
    ) -> Self {
        Self { thread, database }
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct SaveReflectionInput {
    /// The high-level insight or reflection to store.
    pub content: String,
}

impl AgentTool for SaveReflectionTool {
    type Input = SaveReflectionInput;
    type Output = String;

    fn name() -> &'static str {
        "lsp_save_reflection"
    }

    fn description() -> SharedString {
        "Store a high-level technical insight or reflection in the persistent memory store. \
        Use this after completing a complex task, discovering a subtle bug, or successfully navigating a deep code graph."
            .into()
    }

    fn kind() -> acp::ToolKind {
        acp::ToolKind::Read
    }

    fn initial_title(
        &self,
        _input: Result<Self::Input, serde_json::Value>,
        _cx: &mut App,
    ) -> SharedString {
        "Saving reflection".into()
    }

    fn run(
        self: Arc<Self>,
        input: Self::Input,
        _event_stream: ToolCallEventStream,
        cx: &mut App,
    ) -> Task<Result<Self::Output>> {
        let database = self.database.clone();
        let thread = self.thread.clone();
        cx.spawn(async move |cx| {
            let db = database.clone().await.map_err(|e| anyhow!("{e:?}"))?;
            let session_id = thread.update(cx, |thread, _| thread.id().clone())?;
            db.save_reflection(session_id, input.content).await?;
            Ok("Reflection saved successfully.".to_string())
        })
    }

    fn rebind_thread(&self, new_thread: WeakEntity<Thread>) -> Option<Arc<dyn AnyAgentTool>> {
        Some(Self::new(new_thread, self.database.clone()).erase())
    }
}
