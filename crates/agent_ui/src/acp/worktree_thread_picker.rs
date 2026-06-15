use anyhow::Context as _;
use fuzzy::StringMatchCandidate;
use git::repository::Worktree as GitWorktree;
use gpui::{
    App, DismissEvent, Entity, EventEmitter, FocusHandle, Focusable, PathPromptOptions, Render,
    SharedString, Task, WeakEntity, Window, rems,
};
use picker::{Picker, PickerDelegate, PickerEditorPosition};
use project::{DirectoryLister, git_store::Repository};
use std::{path::PathBuf, rc::Rc, sync::Arc};
use ui::{HighlightedLabel, ListItem, ListItemSpacing, prelude::*};
use util::ResultExt as _;
use workspace::{ModalView, Workspace, notifications::DetachAndPromptErr};

/// Callback invoked with the absolute path of the worktree the user picked (or
/// created) for a new agent thread.
pub type OnConfirmWorktree = Rc<dyn Fn(PathBuf, &mut Window, &mut App)>;

/// A modal that lets the user pick (or create) a git worktree to start an agent
/// thread in, so parallel threads can be isolated onto separate working
/// directories. Modeled on `git_ui::worktree_picker`, but instead of opening a
/// new workspace window it hands the chosen path back via `on_confirm`.
pub struct WorktreeThreadPicker {
    picker: Entity<Picker<WorktreeThreadPickerDelegate>>,
    picker_focus_handle: FocusHandle,
    _subscription: gpui::Subscription,
}

impl WorktreeThreadPicker {
    pub fn new(
        repository: Option<Entity<Repository>>,
        workspace: WeakEntity<Workspace>,
        on_confirm: OnConfirmWorktree,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let all_worktrees_request = repository
            .clone()
            .map(|repository| repository.update(cx, |repository, _| repository.worktrees()));
        let default_branch_request = repository.clone().map(|repository| {
            repository.update(cx, |repository, _| repository.default_branch(false))
        });

        cx.spawn_in(window, async move |this, cx| {
            let all_worktrees = match all_worktrees_request {
                Some(request) => request.await??,
                None => Vec::new(),
            };
            let default_branch = match default_branch_request {
                Some(request) => request.await.ok().and_then(Result::ok).flatten(),
                None => None,
            };

            this.update_in(cx, |this, window, cx| {
                this.picker.update(cx, |picker, cx| {
                    picker.delegate.all_worktrees = Some(all_worktrees);
                    picker.delegate.default_branch = default_branch;
                    picker.refresh(window, cx);
                })
            })?;
            anyhow::Ok(())
        })
        .detach_and_log_err(cx);

        let delegate = WorktreeThreadPickerDelegate {
            matches: Vec::new(),
            all_worktrees: None,
            workspace,
            repo: repository,
            selected_index: 0,
            default_branch: None,
            on_confirm,
        };
        let picker = cx.new(|cx| Picker::uniform_list(delegate, window, cx).show_scrollbar(true));
        let picker_focus_handle = picker.focus_handle(cx);
        let subscription = cx.subscribe(&picker, |_, _, _, cx| cx.emit(DismissEvent));

        Self {
            picker,
            picker_focus_handle,
            _subscription: subscription,
        }
    }
}

impl ModalView for WorktreeThreadPicker {}
impl EventEmitter<DismissEvent> for WorktreeThreadPicker {}

impl Focusable for WorktreeThreadPicker {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.picker_focus_handle.clone()
    }
}

impl Render for WorktreeThreadPicker {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .key_context("WorktreeThreadPicker")
            .w(rems(34.))
            .child(self.picker.clone())
            .on_mouse_down_out(cx.listener(|this, _, window, cx| {
                this.picker.update(cx, |picker, cx| {
                    picker.cancel(&Default::default(), window, cx);
                })
            }))
    }
}

#[derive(Clone)]
struct WorktreeEntry {
    worktree: GitWorktree,
    positions: Vec<usize>,
    is_new: bool,
}

struct WorktreeThreadPickerDelegate {
    matches: Vec<WorktreeEntry>,
    all_worktrees: Option<Vec<GitWorktree>>,
    workspace: WeakEntity<Workspace>,
    repo: Option<Entity<Repository>>,
    selected_index: usize,
    default_branch: Option<SharedString>,
    on_confirm: OnConfirmWorktree,
}

impl WorktreeThreadPickerDelegate {
    fn create_worktree(
        &self,
        worktree_branch: &str,
        commit: Option<String>,
        window: &mut Window,
        cx: &mut Context<Picker<Self>>,
    ) {
        let Some(repo) = self.repo.clone() else {
            return;
        };
        let prompt = self
            .workspace
            .update(cx, |workspace, cx| {
                workspace.prompt_for_open_path(
                    PathPromptOptions {
                        files: false,
                        directories: true,
                        multiple: false,
                        prompt: Some("Select directory for new worktree".into()),
                    },
                    DirectoryLister::Project(workspace.project().clone()),
                    window,
                    cx,
                )
            })
            .log_err();
        let Some(prompt) = prompt else {
            return;
        };

        let branch = worktree_branch.to_string();
        let on_confirm = self.on_confirm.clone();
        cx.spawn_in(window, async move |_, cx| {
            let Some(paths) = prompt.await? else {
                return anyhow::Ok(());
            };
            let directory = paths.first().cloned().context("No path selected")?;
            repo.update(cx, |repo, _| {
                repo.create_worktree(branch.clone(), directory.clone(), commit)
            })
            .await??;
            let new_worktree_path = directory.join(&branch);
            cx.update(|window, cx| on_confirm(new_worktree_path, window, cx))?;
            anyhow::Ok(())
        })
        .detach_and_prompt_err("Failed to create worktree", window, cx, |e, _, _| {
            Some(e.to_string())
        });
    }

    fn base_branch<'a>(&'a self, cx: &'a mut Context<Picker<Self>>) -> Option<&'a str> {
        self.repo
            .as_ref()
            .and_then(|repo| repo.read(cx).branch.as_ref().map(|branch| branch.name()))
    }
}

impl PickerDelegate for WorktreeThreadPickerDelegate {
    type ListItem = ListItem;

    fn placeholder_text(&self, _window: &mut Window, _cx: &mut App) -> Arc<str> {
        "Select or create a worktree for this thread…".into()
    }

    fn editor_position(&self) -> PickerEditorPosition {
        PickerEditorPosition::Start
    }

    fn match_count(&self) -> usize {
        self.matches.len()
    }

    fn selected_index(&self) -> usize {
        self.selected_index
    }

    fn set_selected_index(
        &mut self,
        ix: usize,
        _window: &mut Window,
        _: &mut Context<Picker<Self>>,
    ) {
        self.selected_index = ix;
    }

    fn update_matches(
        &mut self,
        query: String,
        window: &mut Window,
        cx: &mut Context<Picker<Self>>,
    ) -> Task<()> {
        let Some(all_worktrees) = self.all_worktrees.clone() else {
            return Task::ready(());
        };

        cx.spawn_in(window, async move |picker, cx| {
            let mut matches: Vec<WorktreeEntry> = if query.is_empty() {
                all_worktrees
                    .into_iter()
                    .map(|worktree| WorktreeEntry {
                        worktree,
                        positions: Vec::new(),
                        is_new: false,
                    })
                    .collect()
            } else {
                let candidates = all_worktrees
                    .iter()
                    .enumerate()
                    .map(|(ix, worktree)| StringMatchCandidate::new(ix, worktree.branch()))
                    .collect::<Vec<_>>();
                fuzzy::match_strings(
                    &candidates,
                    &query,
                    true,
                    true,
                    10000,
                    &Default::default(),
                    cx.background_executor().clone(),
                )
                .await
                .into_iter()
                .map(|candidate| WorktreeEntry {
                    worktree: all_worktrees[candidate.candidate_id].clone(),
                    positions: candidate.positions,
                    is_new: false,
                })
                .collect()
            };

            picker
                .update(cx, |picker, _| {
                    if !query.is_empty()
                        && !matches
                            .first()
                            .is_some_and(|entry| entry.worktree.branch() == query)
                    {
                        let branch = query.replace(' ', "-");
                        matches.push(WorktreeEntry {
                            worktree: GitWorktree {
                                path: PathBuf::new(),
                                ref_name: format!("refs/heads/{branch}").into(),
                                sha: Default::default(),
                            },
                            positions: Vec::new(),
                            is_new: true,
                        });
                    }
                    let delegate = &mut picker.delegate;
                    delegate.matches = matches;
                    delegate.selected_index = delegate
                        .selected_index
                        .min(delegate.matches.len().saturating_sub(1));
                })
                .log_err();
        })
    }

    fn confirm(&mut self, _secondary: bool, window: &mut Window, cx: &mut Context<Picker<Self>>) {
        let Some(entry) = self.matches.get(self.selected_index()).cloned() else {
            return;
        };
        if entry.is_new {
            let commit = self.default_branch.clone().map(Into::into);
            self.create_worktree(&entry.worktree.branch(), commit, window, cx);
            return;
        }

        (self.on_confirm)(entry.worktree.path.clone(), window, cx);
        cx.emit(DismissEvent);
    }

    fn dismissed(&mut self, _: &mut Window, cx: &mut Context<Picker<Self>>) {
        cx.emit(DismissEvent);
    }

    fn no_matches_text(&self, _window: &mut Window, _cx: &mut App) -> Option<SharedString> {
        Some("No worktrees found".into())
    }

    fn render_match(
        &self,
        ix: usize,
        selected: bool,
        _window: &mut Window,
        cx: &mut Context<Picker<Self>>,
    ) -> Option<Self::ListItem> {
        let entry = self.matches.get(ix)?;
        let (label, sublabel) = if entry.is_new {
            (
                Label::new(format!("Create Worktree: \"{}\"…", entry.worktree.branch()))
                    .truncate()
                    .into_any_element(),
                format!(
                    "based off {}",
                    self.base_branch(cx).unwrap_or("the current branch")
                ),
            )
        } else {
            let branch = entry.worktree.branch();
            let branch_first_line = branch.lines().next().unwrap_or(branch);
            let positions = entry
                .positions
                .iter()
                .copied()
                .filter(|&pos| pos < branch_first_line.len())
                .collect::<Vec<_>>();
            (
                HighlightedLabel::new(branch_first_line.to_owned(), positions)
                    .truncate()
                    .into_any_element(),
                entry.worktree.path.to_string_lossy().to_string(),
            )
        };

        Some(
            ListItem::new(format!("worktree-thread-{ix}"))
                .inset(true)
                .spacing(ListItemSpacing::Sparse)
                .toggle_state(selected)
                .child(
                    v_flex().w_full().child(label).child(
                        Label::new(sublabel)
                            .size(LabelSize::Small)
                            .color(Color::Muted)
                            .truncate(),
                    ),
                ),
        )
    }
}
