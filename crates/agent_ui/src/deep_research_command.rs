use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use anyhow::Result;
use assistant_slash_command::{
    ArgumentCompletion, SlashCommand, SlashCommandOutputSection, SlashCommandResult,
};
use gpui::{App, Task, WeakEntity, Window};
use language::{BufferSnapshot, LspAdapterDelegate};
use ui::IconName;
use workspace::Workspace;

pub struct DeepResearchSlashCommand;

impl SlashCommand for DeepResearchSlashCommand {
    fn name(&self) -> String {
        "deep-research".into()
    }

    fn description(&self) -> String {
        "Perform a deep, multi-tab graph-based research dive.".into()
    }

    fn icon(&self) -> IconName {
        IconName::MagnifyingGlass
    }

    fn menu_text(&self) -> String {
        self.description()
    }

    fn requires_argument(&self) -> bool {
        true
    }

    fn complete_argument(
        self: Arc<Self>,
        _arguments: &[String],
        _cancel: Arc<AtomicBool>,
        _workspace: Option<WeakEntity<Workspace>>,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Task<Result<Vec<ArgumentCompletion>>> {
        Task::ready(Ok(Vec::new()))
    }

    fn run(
        self: Arc<Self>,
        _arguments: &[String],
        _context_slash_command_output_sections: &[SlashCommandOutputSection<language::Anchor>],
        _context_buffer: BufferSnapshot,
        _workspace: WeakEntity<Workspace>,
        _delegate: Option<Arc<dyn LspAdapterDelegate>>,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Task<SlashCommandResult> {
        // Run logic is intercepted upstream by the ACP message editor natively for Agent UI.
        Task::ready(Err(anyhow::anyhow!("handled natively")))
    }
}
