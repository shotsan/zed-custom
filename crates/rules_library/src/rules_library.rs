use anyhow::Result;
use collections::{HashMap, HashSet};
use fs::Fs;
use fuzzy;
use editor::{CompletionProvider, SelectionEffects};
use editor::{CurrentLineHighlight, Editor, EditorElement, EditorEvent, EditorStyle, actions::Tab};
use gpui::{
    App, Bounds, DEFAULT_ADDITIONAL_WINDOW_SIZE, Entity, EventEmitter, Focusable, PromptLevel,
    Subscription, Task, TextStyle, TitlebarOptions, WindowBounds, WindowHandle, WindowOptions,
    actions, point, size, transparent_black,
};
use language::{Buffer, LanguageRegistry, language_settings::SoftWrap};
use language_model::{
    ConfiguredModel, LanguageModelRegistry, LanguageModelRequest, LanguageModelRequestMessage, Role,
};
use picker::{Picker, PickerDelegate};
use platform_title_bar::PlatformTitleBar;
use release_channel::ReleaseChannel;
use rope::Rope;
use settings::Settings;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;
use theme::ThemeSettings;
use ui::{Divider, ListItem, ListItemSpacing, ListSubHeader, Tooltip, prelude::*};
use ui_input::ErasedEditor;
use util::{ResultExt, TryFutureExt};
use workspace::{Workspace, WorkspaceSettings, client_side_decorations};
use zed_custom_actions::assistant::InlineAssist;

use prompt_store::{self, *};
pub use prompt_store::user_slash_command::UserSlashCommandRegistry;

pub fn init(fs: Arc<dyn Fs>, cx: &mut App) {
    prompt_store::init(fs, cx);
}

actions!(
    skill_library,
    [
        /// Creates a new skill in the skill library.
        NewSkill,
        /// Deletes the selected skill.
        DeleteSkill,
        /// Duplicates the selected skill.
        DuplicateSkill,
        /// Toggles whether the selected skill is a default skill.
        ToggleDefaultSkill,
        /// Restores a built-in skill to its default content.
        RestoreDefaultContent
    ]
);

pub trait InlineAssistDelegate {
    fn assist(
        &self,
        prompt_editor: &Entity<Editor>,
        initial_prompt: Option<String>,
        window: &mut Window,
        cx: &mut Context<SkillLibrary>,
    );

    /// Returns whether the Agent panel was focused.
    fn focus_agent_panel(
        &self,
        workspace: &mut Workspace,
        window: &mut Window,
        cx: &mut Context<Workspace>,
    ) -> bool;
}

/// This function opens a new Skill library window if one doesn't exist already.
/// If one exists, it brings it to the foreground.
///
/// Note that, when opening a new window, this waits for the PromptStore to be
/// initialized. If it was initialized successfully, it returns a window handle
/// to a Skill library.
pub fn open_skill_library(
    language_registry: Arc<LanguageRegistry>,
    inline_assist_delegate: Box<dyn InlineAssistDelegate>,
    make_completion_provider: Rc<dyn Fn() -> Rc<dyn CompletionProvider>>,
    prompt_to_select: Option<PromptId>,
    user_slash_commands: Option<Entity<prompt_store::user_slash_command::UserSlashCommandRegistry>>,
    cx: &mut App,
) -> Task<Result<WindowHandle<SkillLibrary>>> {
    let store = PromptStore::global(cx);
    cx.spawn(async move |cx| {
        // We query windows in spawn so that all windows have been returned to GPUI
        let existing_window = cx.update({
            let prompt_to_select = prompt_to_select.clone();
            |cx| {
                let existing_window = cx
                    .windows()
                    .into_iter()
                    .find_map(|window| window.downcast::<SkillLibrary>());
                if let Some(existing_window) = existing_window {
                    existing_window
                        .update(cx, |skill_library, window, cx| {
                            if let Some(prompt_to_select) = prompt_to_select {
                                skill_library.load_skill(prompt_to_select, true, window, cx);
                            }
                            window.focus(&skill_library.picker.focus_handle(cx), cx);
                        })
                        .ok();

                    Some(existing_window)
                } else {
                    None
                }
            }
        });

        if let Some(existing_window) = existing_window {
            return Ok(existing_window);
        }

        let store = store.await?;
        cx.update(|cx| {
            let app_id = ReleaseChannel::global(cx).app_id();
            let bounds = Bounds::centered(None, size(px(1024.0), px(768.0)), cx);
            let window_decorations = match std::env::var("ZED_WINDOW_DECORATIONS") {
                Ok(val) if val == "server" => gpui::WindowDecorations::Server,
                Ok(val) if val == "client" => gpui::WindowDecorations::Client,
                _ => match WorkspaceSettings::get_global(cx).window_decorations {
                    settings::WindowDecorations::Server => gpui::WindowDecorations::Server,
                    settings::WindowDecorations::Client => gpui::WindowDecorations::Client,
                },
            };
            cx.open_window(
                WindowOptions {
                    titlebar: Some(TitlebarOptions {
                        title: Some("Skill Library".into()),
                        appears_transparent: true,
                        traffic_light_position: Some(point(px(12.0), px(12.0))),
                    }),
                    app_id: Some(app_id.to_owned()),
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    window_background: cx.theme().window_background_appearance(),
                    window_decorations: Some(window_decorations),
                    window_min_size: Some(DEFAULT_ADDITIONAL_WINDOW_SIZE),
                    kind: gpui::WindowKind::Floating,
                    ..Default::default()
                },
                |window, cx| {
                    cx.new(|cx| {
                        SkillLibrary::new(
                            store,
                            language_registry,
                            inline_assist_delegate,
                            make_completion_provider,
                            prompt_to_select,
                            user_slash_commands,
                            window,
                            cx,
                        )
                    })
                },
            )
        })
    })
}

pub struct SkillLibrary {
    title_bar: Option<Entity<PlatformTitleBar>>,
    store: Entity<PromptStore>,
    language_registry: Arc<LanguageRegistry>,
    skill_editors: HashMap<PromptId, SkillEditor>,
    active_skill_id: Option<PromptId>,
    picker: Entity<Picker<SkillPickerDelegate>>,
    pending_load: Task<()>,
    _user_slash_commands: Option<Entity<prompt_store::user_slash_command::UserSlashCommandRegistry>>,
    inline_assist_delegate: Box<dyn InlineAssistDelegate>,
    make_completion_provider: Rc<dyn Fn() -> Rc<dyn CompletionProvider>>,
    _subscriptions: Vec<Subscription>,
}

struct SkillEditor {
    title_editor: Entity<Editor>,
    body_editor: Entity<Editor>,
    token_count: Option<u64>,
    pending_token_count: Task<Option<()>>,
    next_title_and_body_to_save: Option<(String, Rope)>,
    pending_save: Option<Task<Option<()>>>,
    _subscriptions: Vec<Subscription>,
}

enum SkillPickerEntry {
    Header(SharedString),
    Skill(PromptMetadata),
    Separator,
}

struct SkillPickerDelegate {
    store: Entity<PromptStore>,
    user_slash_commands: Option<Entity<prompt_store::user_slash_command::UserSlashCommandRegistry>>,
    selected_index: usize,
    filtered_entries: Vec<SkillPickerEntry>,
}

enum SkillPickerEvent {
    Selected { prompt_id: PromptId },
    Confirmed { prompt_id: PromptId },
    Deleted { prompt_id: PromptId },
    ToggledDefault { prompt_id: PromptId },
}

impl EventEmitter<SkillPickerEvent> for Picker<SkillPickerDelegate> {}

impl PickerDelegate for SkillPickerDelegate {
    type ListItem = AnyElement;

    fn match_count(&self) -> usize {
        self.filtered_entries.len()
    }

    fn no_matches_text(&self, _window: &mut Window, _cx: &mut App) -> Option<SharedString> {
        Some("No skills found matching your search.".into())
    }

    fn selected_index(&self) -> usize {
        self.selected_index
    }

    fn set_selected_index(&mut self, ix: usize, _: &mut Window, cx: &mut Context<Picker<Self>>) {
        self.selected_index = ix.min(self.filtered_entries.len().saturating_sub(1));

        if let Some(SkillPickerEntry::Skill(rule)) = self.filtered_entries.get(self.selected_index) {
            cx.emit(SkillPickerEvent::Selected {
                prompt_id: rule.id.clone(),
            });
        }

        cx.notify();
    }

    fn can_select(&mut self, ix: usize, _: &mut Window, _: &mut Context<Picker<Self>>) -> bool {
        match self.filtered_entries.get(ix) {
            Some(SkillPickerEntry::Skill(_)) => true,
            Some(SkillPickerEntry::Header(_)) | Some(SkillPickerEntry::Separator) | None => false,
        }
    }

    fn placeholder_text(&self, _window: &mut Window, _cx: &mut App) -> Arc<str> {
        "Search…".into()
    }

    fn update_matches(
        &mut self,
        query: String,
        window: &mut Window,
        cx: &mut Context<Picker<Self>>,
    ) -> Task<()> {
        let cancellation_flag = Arc::new(AtomicBool::default());
        let search = self.store.read(cx).search(query.clone(), cancellation_flag.clone(), cx);
        let user_slash_commands = self.user_slash_commands.clone();
        let query_for_capture = query.clone();

        let prev_prompt_id = self
            .filtered_entries
            .get(self.selected_index)
            .and_then(|entry| {
                if let SkillPickerEntry::Skill(rule) = entry {
                    Some(rule.id.clone())
                } else {
                    None
                }
            });

        cx.spawn_in(window, async move |this, cx| {
            let mut matches = search.await;

            if let Some(user_slash_commands) = user_slash_commands {
                let commands = this.update(cx, |_, cx| {
                    user_slash_commands
                        .read(cx)
                        .commands()
                        .values()
                        .filter(|cmd| !cmd.path.to_string_lossy().starts_with("db://"))
                        .cloned()
                        .collect::<Vec<_>>()
                });
                let Ok(commands) = commands else { return };

                if !commands.is_empty() {
                    let candidates = commands
                        .iter()
                        .enumerate()
                        .map(|(i, cmd)| fuzzy::StringMatchCandidate::new(i, &cmd.name))
                        .collect::<Vec<_>>();
                    let query = query_for_capture.clone();
                    let executor = cx.background_executor().clone();

                    let match_indices = cx
                        .background_spawn(async move {
                            fuzzy::match_strings(
                                &candidates,
                                &query,
                                false,
                                false,
                                100,
                                &std::sync::atomic::AtomicBool::new(false),
                                executor,
                            )
                            .await
                        })
                        .await;

                    for mat in match_indices {
                        matches.push(PromptMetadata::from_user_slash_command(
                            &commands[mat.candidate_id],
                        ));
                    }
                }
            }

            let (filtered_entries, selected_index) = cx
                .background_spawn(async move {
                    // Remove duplicates if any (e.g. if we somehow indexed the same thing twice)
                    let mut seen = HashSet::default();
                    matches.retain(|m| seen.insert(m.id.clone()));

                    let (file_skills, db_skills): (Vec<_>, Vec<_>) = matches
                        .into_iter()
                        .partition(|rule| matches!(rule.id, PromptId::File { .. }));

                    let (project_skills, user_file_skills): (Vec<_>, Vec<_>) =
                        file_skills.into_iter().partition(|rule| {
                            if let PromptId::File { scope, .. } = rule.id {
                                scope == prompt_store::user_slash_command::CommandScope::Project
                            } else {
                                false
                            }
                        });

                    let (built_in_skills, db_user_skills): (Vec<_>, Vec<_>) =
                        db_skills.into_iter().partition(|rule| rule.id.is_built_in());

                    let mut user_skills = db_user_skills;
                    user_skills.extend(user_file_skills);

                    let (default_user_skills, other_user_skills): (Vec<_>, Vec<_>) =
                        user_skills.into_iter().partition(|rule| rule.default);

                    let mut filtered_entries = Vec::new();

                    if !project_skills.is_empty() {
                        filtered_entries.push(SkillPickerEntry::Header("Project Skills".into()));
                        for rule in project_skills {
                            filtered_entries.push(SkillPickerEntry::Skill(rule));
                        }
                        filtered_entries.push(SkillPickerEntry::Separator);
                    }

                    if !default_user_skills.is_empty() {
                        filtered_entries.push(SkillPickerEntry::Header("Default Skills".into()));
                        for rule in default_user_skills {
                            filtered_entries.push(SkillPickerEntry::Skill(rule));
                        }
                        filtered_entries.push(SkillPickerEntry::Separator);
                    }

                    if !other_user_skills.is_empty() {
                        filtered_entries.push(SkillPickerEntry::Header("My Skills".into()));
                        for rule in other_user_skills {
                            filtered_entries.push(SkillPickerEntry::Skill(rule));
                        }
                        filtered_entries.push(SkillPickerEntry::Separator);
                    }

                    if !built_in_skills.is_empty() {
                        filtered_entries.push(SkillPickerEntry::Header("Built-in Skills".into()));
                        for rule in built_in_skills {
                            filtered_entries.push(SkillPickerEntry::Skill(rule));
                        }
                    }

                    let selected_index = prev_prompt_id
                        .and_then(|prev_prompt_id| {
                            filtered_entries.iter().position(|entry| {
                                if let SkillPickerEntry::Skill(rule) = entry {
                                    rule.id == prev_prompt_id
                                } else {
                                    false
                                }
                            })
                        })
                        .unwrap_or_else(|| {
                            filtered_entries
                                .iter()
                                .position(|entry| matches!(entry, SkillPickerEntry::Skill(_)))
                                .unwrap_or(0)
                        });

                    (filtered_entries, selected_index)
                })
                .await;

            this.update_in(cx, |this, window, cx| {
                this.delegate.filtered_entries = filtered_entries;
                this.set_selected_index(
                    selected_index,
                    Some(picker::Direction::Down),
                    true,
                    window,
                    cx,
                );
                cx.notify();
            })
            .ok();
        })
    }

    fn confirm(&mut self, _secondary: bool, _: &mut Window, cx: &mut Context<Picker<Self>>) {
        if let Some(SkillPickerEntry::Skill(rule)) = self.filtered_entries.get(self.selected_index) {
            cx.emit(SkillPickerEvent::Confirmed {
                prompt_id: rule.id.clone(),
            });
        }
    }

    fn dismissed(&mut self, _window: &mut Window, _cx: &mut Context<Picker<Self>>) {}

    fn render_match(
        &self,
        ix: usize,
        selected: bool,
        _: &mut Window,
        cx: &mut Context<Picker<Self>>,
    ) -> Option<Self::ListItem> {
        match self.filtered_entries.get(ix)? {
            SkillPickerEntry::Header(title) => {
                let tooltip_text = match title.as_ref() {
                    "Built-in Skills" => "Built-in skills are those included out of the box with zed-custom.",
                    "Default Skills" => "Default skills are attached by default to every new thread.",
                    "Project Skills" => "Project skills are defined in your workspace and stay with the project.",
                    _ => "These are your personal skills stored globally across all projects.",
                };

                Some(
                    ListSubHeader::new(title.clone())
                        .end_slot(
                            IconButton::new("info", IconName::Info)
                                .style(ButtonStyle::Transparent)
                                .icon_size(IconSize::Small)
                                .icon_color(Color::Muted)
                                .tooltip(Tooltip::text(tooltip_text))
                                .into_any_element(),
                        )
                        .inset(true)
                        .into_any_element(),
                )
            }
            SkillPickerEntry::Separator => Some(
                h_flex()
                    .py_1()
                    .child(Divider::horizontal())
                    .into_any_element(),
            ),
            SkillPickerEntry::Skill(rule) => {
                let default = rule.default;
                let prompt_id = rule.id.clone();

                Some(
                    ListItem::new(ix)
                        .inset(true)
                        .spacing(ListItemSpacing::Sparse)
                        .toggle_state(selected)
                        .child(
                            Label::new(rule.title.clone().unwrap_or("Untitled".into()))
                                .truncate()
                                .mr_10(),
                        )
                        .end_slot::<IconButton>((default && !prompt_id.is_built_in()).then(|| {
                            IconButton::new("toggle-default-skill", IconName::Paperclip)
                                .toggle_state(true)
                                .icon_color(Color::Accent)
                                .icon_size(IconSize::Small)
                                .tooltip(Tooltip::text("Remove from Default Skills"))
                                .on_click(cx.listener({
                                    let prompt_id = prompt_id.clone();
                                    move |_, _, _, cx| {
                                        cx.emit(SkillPickerEvent::ToggledDefault {
                                            prompt_id: prompt_id.clone(),
                                        })
                                    }
                                }))
                        }))
                        .when(!prompt_id.is_built_in(), |this| {
                            this.end_hover_slot(
                                h_flex()
                                    .child(
                                        IconButton::new("delete-skill", IconName::Trash)
                                            .icon_color(Color::Muted)
                                            .icon_size(IconSize::Small)
                                            .tooltip(Tooltip::text("Delete Skill"))
                                            .on_click(cx.listener({
                                                let prompt_id = prompt_id.clone();
                                                move |_, _, _, cx| {
                                                    cx.emit(SkillPickerEvent::Deleted {
                                                        prompt_id: prompt_id.clone(),
                                                    })
                                                }
                                            })),
                                    )
                                    .child(
                                        IconButton::new("toggle-default-skill", IconName::Plus)
                                            .selected_icon(IconName::Dash)
                                            .toggle_state(default)
                                            .icon_size(IconSize::Small)
                                            .icon_color(if default {
                                                Color::Accent
                                            } else {
                                                Color::Muted
                                            })
                                            .map(|this| {
                                                if default {
                                                    this.tooltip(Tooltip::text(
                                                        "Remove from Default Skills",
                                                    ))
                                                } else {
                                                    this.tooltip(move |_window, cx| {
                                                        Tooltip::with_meta(
                                                            "Add to Default Skills",
                                                            None,
                                                            "Always included in every thread.",
                                                            cx,
                                                        )
                                                    })
                                                }
                                            })
                                            .on_click(cx.listener({
                                                let prompt_id = prompt_id.clone();
                                                move |_, _, _, cx| {
                                                    cx.emit(SkillPickerEvent::ToggledDefault {
                                                        prompt_id: prompt_id.clone(),
                                                    })
                                                }
                                            })),
                                    ),
                            )
                        })
                        .into_any_element(),
                )
            }
        }
    }

    fn render_editor(
        &self,
        editor: &Arc<dyn ErasedEditor>,
        _: &mut Window,
        cx: &mut Context<Picker<Self>>,
    ) -> Div {
        let editor = editor.as_any().downcast_ref::<Entity<Editor>>().unwrap();

        h_flex()
            .py_1()
            .px_1p5()
            .mx_1()
            .gap_1p5()
            .rounded_sm()
            .bg(cx.theme().colors().editor_background)
            .border_1()
            .border_color(cx.theme().colors().border)
            .child(Icon::new(IconName::MagnifyingGlass).color(Color::Muted))
            .child(editor.clone())
    }
}

impl SkillLibrary {
    fn new(
        store: Entity<PromptStore>,
        language_registry: Arc<LanguageRegistry>,
        inline_assist_delegate: Box<dyn InlineAssistDelegate>,
        make_completion_provider: Rc<dyn Fn() -> Rc<dyn CompletionProvider>>,
        skill_to_select: Option<PromptId>,
        user_slash_commands: Option<Entity<prompt_store::user_slash_command::UserSlashCommandRegistry>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let (_selected_index, _matches) = if let Some(skill_to_select) = skill_to_select {
            let matches = store.read(cx).all_prompt_metadata();
            let selected_index = matches
                .iter()
                .enumerate()
                .find(|(_, metadata)| metadata.id == skill_to_select)
                .map_or(0, |(ix, _)| ix);
            (selected_index, matches)
        } else {
            (0, vec![])
        };

        let picker_delegate = SkillPickerDelegate {
            store: store.clone(),
            user_slash_commands: user_slash_commands.clone(),
            selected_index: 0,
            filtered_entries: Vec::new(),
        };

        let picker = cx.new(|cx| {
            let picker = Picker::list(picker_delegate, window, cx)
                .modal(false)
                .max_height(None);
            picker.focus(window, cx);
            picker
        });

        Self {
            title_bar: if !cfg!(target_os = "macos") {
                Some(cx.new(|cx| PlatformTitleBar::new("Skill library-title-bar", cx)))
            } else {
                None
            },
            store,
            language_registry,
            skill_editors: HashMap::default(),
            active_skill_id: None,
            pending_load: Task::ready(()),
            inline_assist_delegate,
            make_completion_provider,
            _user_slash_commands: user_slash_commands,
            _subscriptions: {
                let mut subscriptions = vec![
                    cx.subscribe_in(&picker, window, Self::handle_picker_event),
                    cx.subscribe_in(&store, window, |this, _, _, window, cx| {
                        this.picker.update(cx, |picker, cx| picker.refresh(window, cx));
                    }),
                ];
                if let Some(registry) = &user_slash_commands {
                    subscriptions.push(cx.subscribe_in(registry, window, |this, _, _, window, cx| {
                        this.picker.update(cx, |picker, cx| picker.refresh(window, cx));
                    }));
                }
                subscriptions
            },
            picker,
        }
    }

    fn handle_picker_event(
        &mut self,
        _: &Entity<Picker<SkillPickerDelegate>>,
        event: &SkillPickerEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            SkillPickerEvent::Selected { prompt_id } => {
                self.load_skill(prompt_id.clone(), false, window, cx);
            }
            SkillPickerEvent::Confirmed { prompt_id } => {
                self.load_skill(prompt_id.clone(), true, window, cx);
            }
            SkillPickerEvent::ToggledDefault { prompt_id } => {
                self.toggle_default_for_skill(prompt_id.clone(), window, cx);
            }
            SkillPickerEvent::Deleted { prompt_id } => {
                self.delete_skill(prompt_id.clone(), window, cx);
            }
        }
    }

    pub fn new_skill(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // If we already have an untitled rule, use that instead
        // of creating a new one.
        if let Some(metadata) = self.store.read(cx).first()
            && metadata.title.is_none()
        {
            self.load_skill(metadata.id.clone(), true, window, cx);
            return;
        }

        let prompt_id = PromptId::new();
        let save = self.store.update(cx, |store, cx| {
            store.save(prompt_id.clone(), None, false, "".into(), cx)
        });
        self.picker
            .update(cx, |picker, cx| picker.refresh(window, cx));
        cx.spawn_in(window, async move |this, cx| {
            save.await?;
            this.update_in(cx, |this, window, cx| {
                this.load_skill(prompt_id, true, window, cx)
            })
        })
        .detach_and_log_err(cx);
    }

    pub fn save_skill(&mut self, prompt_id: PromptId, window: &mut Window, cx: &mut Context<Self>) {
        const SAVE_THROTTLE: Duration = Duration::from_millis(500);

        if !prompt_id.can_edit() {
            return;
        }

        let skill_metadata = self.store.read(cx).metadata(prompt_id.clone()).unwrap();
        let skill_editor = self.skill_editors.get_mut(&prompt_id).unwrap();
        let title = skill_editor.title_editor.read(cx).text(cx);
        let body = skill_editor.body_editor.update(cx, |editor, cx| {
            editor
                .buffer()
                .read(cx)
                .as_singleton()
                .unwrap()
                .read(cx)
                .as_rope()
                .clone()
        });

        let store = self.store.clone();
        let executor = cx.background_executor().clone();

        skill_editor.next_title_and_body_to_save = Some((title, body));
        if skill_editor.pending_save.is_none() {
            let prompt_id = prompt_id.clone();
            skill_editor.pending_save = Some(cx.spawn_in(window, async move |this, cx| {
                async move {
                    loop {
                        let prompt_id = prompt_id.clone();
                        let title_and_body = this.update(cx, |this, _| {
                            this.skill_editors
                                .get_mut(&prompt_id)?
                                .next_title_and_body_to_save
                                .take()
                        })?;

                        if let Some((title, body)) = title_and_body {
                            let title = if title.trim().is_empty() {
                                None
                            } else {
                                Some(SharedString::from(title))
                            };
                            let prompt_id = prompt_id.clone();
                            cx.update(|_window, cx| {
                                store.update(cx, |store, cx| {
                                    store.save(prompt_id, title, skill_metadata.default, body, cx)
                                })
                            })?
                            .await
                            .log_err();
                            this.update_in(cx, |this, window, cx| {
                                this.picker
                                    .update(cx, |picker, cx| picker.refresh(window, cx));
                                cx.notify();
                            })?;

                            executor.timer(SAVE_THROTTLE).await;
                        } else {
                            break;
                        }
                    }

                    this.update(cx, |this, _cx| {
                        if let Some(skill_editor) = this.skill_editors.get_mut(&prompt_id) {
                            skill_editor.pending_save = None;
                        }
                    })
                }
                .log_err()
                .await
            }));
        }
    }

    pub fn delete_active_skill(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(active_skill_id) = self.active_skill_id.clone() {
            self.delete_skill(active_skill_id, window, cx);
        }
    }

    pub fn duplicate_active_skill(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(active_skill_id) = self.active_skill_id.clone() {
            self.duplicate_skill(active_skill_id, window, cx);
        }
    }

    pub fn toggle_default_for_active_skill(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(active_skill_id) = self.active_skill_id.clone() {
            self.toggle_default_for_skill(active_skill_id, window, cx);
        }
    }

    pub fn restore_default_content_for_active_skill(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(active_skill_id) = self.active_skill_id.clone() {
            self.restore_default_content(active_skill_id, window, cx);
        }
    }

    pub fn restore_default_content(
        &mut self,
        prompt_id: PromptId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(built_in) = prompt_id.as_built_in() else {
            return;
        };

        if let Some(skill_editor) = self.skill_editors.get(&prompt_id) {
            skill_editor.body_editor.update(cx, |editor, cx| {
                editor.set_text(built_in.default_content(), window, cx);
            });
        }
    }

    pub fn toggle_default_for_skill(
        &mut self,
        prompt_id: PromptId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.store.update(cx, move |store, cx| {
            if let Some(skill_metadata) = store.metadata(prompt_id.clone()) {
                store
                    .save_metadata(prompt_id, skill_metadata.title, !skill_metadata.default, cx)
                    .detach_and_log_err(cx);
            }
        });
        self.picker
            .update(cx, |picker, cx| picker.refresh(window, cx));
        cx.notify();
    }

    pub fn load_skill(
        &mut self,
        prompt_id: PromptId,
        focus: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(skill_editor) = self.skill_editors.get(&prompt_id) {
            if focus {
                skill_editor
                    .body_editor
                    .update(cx, |editor, cx| window.focus(&editor.focus_handle(cx), cx));
            }
            self.set_active_skill(Some(prompt_id), window, cx);
        } else if let Some(skill_metadata) = self.store.read(cx).metadata(prompt_id.clone()) {
            let language_registry = self.language_registry.clone();
            let rule = self.store.read(cx).load(prompt_id.clone(), cx);
            let make_completion_provider = self.make_completion_provider.clone();
            self.pending_load = cx.spawn_in(window, async move |this, cx| {
                let rule = rule.await;
                let markdown = language_registry.language_for_name("Markdown").await;
                this.update_in(cx, |this, window, cx| match rule {
                    Ok(rule) => {
                        let title_editor = cx.new(|cx| {
                            let mut editor = Editor::single_line(window, cx);
                            editor.set_placeholder_text("Untitled", window, cx);
                            editor.set_text(skill_metadata.title.unwrap_or_default(), window, cx);
                            if prompt_id.is_built_in() {
                                editor.set_read_only(true);
                                editor.set_show_edit_predictions(Some(false), window, cx);
                            }
                            editor
                        });
                        let body_editor = cx.new(|cx| {
                            let buffer = cx.new(|cx| {
                                let mut buffer = Buffer::local(rule, cx);
                                buffer.set_language(markdown.log_err(), cx);
                                buffer.set_language_registry(language_registry);
                                buffer
                            });

                            let mut editor = Editor::for_buffer(buffer, None, window, cx);
                            if !prompt_id.can_edit() {
                                editor.set_read_only(true);
                                editor.set_show_edit_predictions(Some(false), window, cx);
                            }
                            editor.set_soft_wrap_mode(SoftWrap::EditorWidth, cx);
                            editor.set_show_gutter(false, cx);
                            editor.set_show_wrap_guides(false, cx);
                            editor.set_show_indent_guides(false, cx);
                            editor.set_use_modal_editing(true);
                            editor.set_current_line_highlight(Some(CurrentLineHighlight::None));
                            editor.set_completion_provider(Some(make_completion_provider()));
                            if focus {
                                window.focus(&editor.focus_handle(cx), cx);
                            }
                            editor
                        });
                        let _subscriptions = [
                            cx.subscribe_in(
                                &title_editor,
                                window,
                                {
                                    let prompt_id = prompt_id.clone();
                                    move |this, editor, event, window, cx| {
                                        this.handle_rule_title_editor_event(
                                            prompt_id.clone(),
                                            editor,
                                            event,
                                            window,
                                            cx,
                                        )
                                    }
                                },
                            ),
                            cx.subscribe_in(
                                &body_editor,
                                window,
                                {
                                    let prompt_id = prompt_id.clone();
                                    move |this, editor, event, window, cx| {
                                        this.handle_rule_body_editor_event(
                                            prompt_id.clone(),
                                            editor,
                                            event,
                                            window,
                                            cx,
                                        )
                                    }
                                },
                            ),
                        ];
                        this.skill_editors.insert(
                            prompt_id.clone(),
                            SkillEditor {
                                title_editor,
                                body_editor,
                                next_title_and_body_to_save: None,
                                pending_save: None,
                                token_count: None,
                                pending_token_count: Task::ready(None),
                                _subscriptions: Vec::from(_subscriptions),
                            },
                        );
                        this.set_active_skill(Some(prompt_id.clone()), window, cx);
                        this.count_tokens(prompt_id, window, cx);
                    }
                    Err(error) => {
                        // TODO: we should show the error in the UI.
                        log::error!("error while loading rule: {:?}", error);
                    }
                })
                .ok();
            });
        }
    }

    fn set_active_skill(
        &mut self,
        prompt_id: Option<PromptId>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.active_skill_id = prompt_id.clone();
        self.picker.update(cx, |picker, cx| {
            if let Some(prompt_id) = prompt_id {
                if picker
                    .delegate
                    .filtered_entries
                    .get(picker.delegate.selected_index())
                    .is_none_or(|old_selected_prompt| {
                        if let SkillPickerEntry::Skill(rule) = old_selected_prompt {
                            rule.id != prompt_id
                        } else {
                            true
                        }
                    })
                    && let Some(ix) = picker.delegate.filtered_entries.iter().position(|mat| {
                        if let SkillPickerEntry::Skill(rule) = mat {
                            rule.id == prompt_id
                        } else {
                            false
                        }
                    })
                {
                    picker.set_selected_index(ix, None, true, window, cx);
                }
            } else {
                picker.focus(window, cx);
            }
        });
        cx.notify();
    }

    pub fn delete_skill(
        &mut self,
        prompt_id: PromptId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(metadata) = self.store.read(cx).metadata(prompt_id.clone()) {
            let confirmation = window.prompt(
                PromptLevel::Warning,
                &format!(
                    "Are you sure you want to delete {}",
                    metadata.title.unwrap_or("Untitled".into())
                ),
                None,
                &["Delete", "Cancel"],
                cx,
            );

            cx.spawn_in(window, async move |this, cx| {
                if confirmation.await.ok() == Some(0) {
                    this.update_in(cx, |this, window, cx| {
                        if this.active_skill_id.as_ref() == Some(&prompt_id) {
                            this.set_active_skill(None, window, cx);
                        }
                        this.skill_editors.remove(&prompt_id);
                        this.store
                            .update(cx, |store, cx| store.delete(prompt_id, cx))
                            .detach_and_log_err(cx);
                        this.picker
                            .update(cx, |picker, cx| picker.refresh(window, cx));
                        cx.notify();
                    })?;
                }
                anyhow::Ok(())
            })
            .detach_and_log_err(cx);
        }
    }

    pub fn duplicate_skill(
        &mut self,
        prompt_id: PromptId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(rule) = self.skill_editors.get(&prompt_id) {
            const DUPLICATE_SUFFIX: &str = " copy";
            let title_to_duplicate = rule.title_editor.read(cx).text(cx);
            let existing_titles = self
                .skill_editors
                .iter()
                .filter(|&(id, _)| id != &prompt_id)
                .map(|(_, skill_editor)| skill_editor.title_editor.read(cx).text(cx))
                .filter(|title| title.starts_with(&title_to_duplicate))
                .collect::<HashSet<_>>();

            let title = if existing_titles.is_empty() {
                title_to_duplicate + DUPLICATE_SUFFIX
            } else {
                let mut i = 1;
                loop {
                    let new_title = format!("{title_to_duplicate}{DUPLICATE_SUFFIX} {i}");
                    if !existing_titles.contains(&new_title) {
                        break new_title;
                    }
                    i += 1;
                }
            };

            let new_id = PromptId::new();
            let body = rule.body_editor.read(cx).text(cx);
            let save = self.store.update(cx, |store, cx| {
                store.save(new_id.clone(), Some(title.into()), false, body.into(), cx)
            });
            self.picker
                .update(cx, |picker, cx| picker.refresh(window, cx));
            cx.spawn_in(window, async move |this, cx| {
                save.await?;
                this.update_in(cx, |skill_library, window, cx| {
                    skill_library.load_skill(new_id, true, window, cx)
                })
            })
            .detach_and_log_err(cx);
        }
    }

    fn focus_active_skill(&mut self, _: &Tab, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(active_rule) = &self.active_skill_id {
            self.skill_editors[&active_rule]
                .body_editor
                .update(cx, |editor, cx| window.focus(&editor.focus_handle(cx), cx));
            cx.stop_propagation();
        }
    }

    fn focus_picker(&mut self, _: &menu::Cancel, window: &mut Window, cx: &mut Context<Self>) {
        self.picker
            .update(cx, |picker, cx| picker.focus(window, cx));
    }

    pub fn inline_assist(
        &mut self,
        action: &InlineAssist,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(active_skill_id) = self.active_skill_id.as_ref() else {
            cx.propagate();
            return;
        };

        let skill_editor = &self.skill_editors[&active_skill_id].body_editor;
        let Some(ConfiguredModel { provider, .. }) =
            LanguageModelRegistry::read_global(cx).inline_assistant_model()
        else {
            return;
        };

        let initial_prompt = action.prompt.clone();
        if provider.is_authenticated(cx) {
            self.inline_assist_delegate
                .assist(skill_editor, initial_prompt, window, cx);
        } else {
            for window in cx.windows() {
                if let Some(workspace) = window.downcast::<Workspace>() {
                    let panel = workspace
                        .update(cx, |workspace, window, cx| {
                            window.activate_window();
                            self.inline_assist_delegate
                                .focus_agent_panel(workspace, window, cx)
                        })
                        .ok();
                    if panel == Some(true) {
                        return;
                    }
                }
            }
        }
    }

    fn move_down_from_title(
        &mut self,
        _: &zed_custom_actions::editor::MoveDown,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(rule_id) = &self.active_skill_id
            && let Some(skill_editor) = self.skill_editors.get(&rule_id)
        {
            window.focus(&skill_editor.body_editor.focus_handle(cx), cx);
        }
    }

    fn move_up_from_body(
        &mut self,
        _: &zed_custom_actions::editor::MoveUp,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(rule_id) = &self.active_skill_id
            && let Some(skill_editor) = self.skill_editors.get(&rule_id)
        {
            window.focus(&skill_editor.title_editor.focus_handle(cx), cx);
        }
    }

    fn handle_rule_title_editor_event(
        &mut self,
        prompt_id: PromptId,
        title_editor: &Entity<Editor>,
        event: &EditorEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            EditorEvent::BufferEdited => {
                self.save_skill(prompt_id.clone(), window, cx);
                self.count_tokens(prompt_id, window, cx);
            }
            EditorEvent::Blurred => {
                title_editor.update(cx, |title_editor, cx| {
                    title_editor.change_selections(
                        SelectionEffects::no_scroll(),
                        window,
                        cx,
                        |selections| {
                            let cursor = selections.oldest_anchor().head();
                            selections.select_anchor_ranges([cursor..cursor]);
                        },
                    );
                });
            }
            _ => {}
        }
    }

    fn handle_rule_body_editor_event(
        &mut self,
        prompt_id: PromptId,
        body_editor: &Entity<Editor>,
        event: &EditorEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            EditorEvent::BufferEdited => {
                self.save_skill(prompt_id.clone(), window, cx);
                self.count_tokens(prompt_id, window, cx);
            }
            EditorEvent::Blurred => {
                body_editor.update(cx, |body_editor, cx| {
                    body_editor.change_selections(
                        SelectionEffects::no_scroll(),
                        window,
                        cx,
                        |selections| {
                            let cursor = selections.oldest_anchor().head();
                            selections.select_anchor_ranges([cursor..cursor]);
                        },
                    );
                });
            }
            _ => {}
        }
    }

    fn count_tokens(&mut self, prompt_id: PromptId, window: &mut Window, cx: &mut Context<Self>) {
        let Some(ConfiguredModel { model, .. }) =
            LanguageModelRegistry::read_global(cx).default_model()
        else {
            return;
        };
        if let Some(rule) = self.skill_editors.get_mut(&prompt_id) {
            let editor = &rule.body_editor.read(cx);
            let buffer = &editor.buffer().read(cx).as_singleton().unwrap().read(cx);
            let body = buffer.as_rope().clone();
            rule.pending_token_count = cx.spawn_in(window, async move |this, cx| {
                async move {
                    const DEBOUNCE_TIMEOUT: Duration = Duration::from_secs(1);

                    cx.background_executor().timer(DEBOUNCE_TIMEOUT).await;
                    let token_count = cx
                        .update(|_, cx| {
                            model.count_tokens(
                                LanguageModelRequest {
                                    thread_id: None,
                                    prompt_id: None,
                                    intent: None,
                                    messages: vec![LanguageModelRequestMessage {
                                        role: Role::System,
                                        content: vec![body.to_string().into()],
                                        cache: false,
                                        reasoning_details: None,
                                    }],
                                    tools: Vec::new(),
                                    tool_choice: None,
                                    stop: Vec::new(),
                                    temperature: None,
                                    thinking_allowed: true,
                                },
                                cx,
                            )
                        })?
                        .await?;

                    this.update(cx, |this, cx| {
                        let skill_editor = this.skill_editors.get_mut(&prompt_id).unwrap();
                        skill_editor.token_count = Some(token_count);
                        cx.notify();
                    })
                }
                .log_err()
                .await
            });
        }
    }

    fn render_skill_list(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .id("skill-list")
            .capture_action(cx.listener(Self::focus_active_skill))
            .px_1p5()
            .h_full()
            .w_64()
            .overflow_x_hidden()
            .bg(cx.theme().colors().panel_background)
            .map(|this| {
                if cfg!(target_os = "macos") {
                    this.child(
                        h_flex()
                            .p(DynamicSpacing::Base04.rems(cx))
                            .h_9()
                            .w_full()
                            .flex_none()
                            .justify_end()
                            .child(
                                IconButton::new("new-rule", IconName::Plus)
                                    .tooltip(move |_window, cx| {
                                        Tooltip::for_action("New Skill", &NewSkill, cx)
                                    })
                                    .on_click(|_, window, cx| {
                                        window.dispatch_action(Box::new(NewSkill), cx);
                                    }),
                            ),
                    )
                } else {
                    this.child(
                        h_flex().p_1().w_full().child(
                            Button::new("new-rule", "New Skill")
                                .full_width()
                                .style(ButtonStyle::Outlined)
                                .icon(IconName::Plus)
                                .icon_size(IconSize::Small)
                                .icon_position(IconPosition::Start)
                                .icon_color(Color::Muted)
                                .on_click(|_, window, cx| {
                                    window.dispatch_action(Box::new(NewSkill), cx);
                                }),
                        ),
                    )
                }
            })
            .child(div().flex_grow().child(self.picker.clone()))
    }

    fn render_active_skill_editor(
        &self,
        editor: &Entity<Editor>,
        read_only: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let settings = ThemeSettings::get_global(cx);
        let text_color = if read_only {
            cx.theme().colors().text_muted
        } else {
            cx.theme().colors().text
        };

        div()
            .w_full()
            .pl_1()
            .border_1()
            .border_color(transparent_black())
            .rounded_sm()
            .when(!read_only, |this| {
                this.group_hover("active-editor-header", |this| {
                    this.border_color(cx.theme().colors().border_variant)
                })
            })
            .on_action(cx.listener(Self::move_down_from_title))
            .child(EditorElement::new(
                &editor,
                EditorStyle {
                    background: cx.theme().system().transparent,
                    local_player: cx.theme().players().local(),
                    text: TextStyle {
                        color: text_color,
                        font_family: settings.ui_font.family.clone(),
                        font_features: settings.ui_font.features.clone(),
                        font_size: HeadlineSize::Medium.rems().into(),
                        font_weight: settings.ui_font.weight,
                        line_height: relative(settings.buffer_line_height.value()),
                        ..Default::default()
                    },
                    scrollbar_width: Pixels::ZERO,
                    syntax: cx.theme().syntax().clone(),
                    status: cx.theme().status().clone(),
                    inlay_hints_style: editor::make_inlay_hints_style(cx),
                    edit_prediction_styles: editor::make_suggestion_styles(cx),
                    ..EditorStyle::default()
                },
            ))
    }

    fn render_duplicate_skill_button(&self) -> impl IntoElement {
        IconButton::new("duplicate-rule", IconName::BookCopy)
            .tooltip(move |_window, cx| Tooltip::for_action("Duplicate Skill", &DuplicateSkill, cx))
            .on_click(|_, window, cx| {
                window.dispatch_action(Box::new(DuplicateSkill), cx);
            })
    }

    fn render_built_in_rule_controls(&self) -> impl IntoElement {
        h_flex()
            .gap_1()
            .child(self.render_duplicate_skill_button())
            .child(
                IconButton::new("restore-default", IconName::RotateCcw)
                    .tooltip(move |_window, cx| {
                        Tooltip::for_action(
                            "Restore to Default Content",
                            &RestoreDefaultContent,
                            cx,
                        )
                    })
                    .on_click(|_, window, cx| {
                        window.dispatch_action(Box::new(RestoreDefaultContent), cx);
                    }),
            )
    }

    fn render_regular_rule_controls(&self, default: bool) -> impl IntoElement {
        h_flex()
            .gap_1()
            .child(
                IconButton::new("toggle-default-rule", IconName::Paperclip)
                    .toggle_state(default)
                    .when(default, |this| this.icon_color(Color::Accent))
                    .map(|this| {
                        if default {
                            this.tooltip(Tooltip::text("Remove from Default Skills"))
                        } else {
                            this.tooltip(move |_window, cx| {
                                Tooltip::with_meta(
                                    "Add to Default Skills",
                                    None,
                                    "Always included in every thread.",
                                    cx,
                                )
                            })
                        }
                    })
                    .on_click(|_, window, cx| {
                        window.dispatch_action(Box::new(ToggleDefaultSkill), cx);
                    }),
            )
            .child(self.render_duplicate_skill_button())
            .child(
                IconButton::new("delete-rule", IconName::Trash)
                    .tooltip(move |_window, cx| Tooltip::for_action("Delete Skill", &DeleteSkill, cx))
                    .on_click(|_, window, cx| {
                        window.dispatch_action(Box::new(DeleteSkill), cx);
                    }),
            )
    }

    fn render_active_skill(&mut self, cx: &mut Context<SkillLibrary>) -> gpui::Stateful<Div> {
        div()
            .id("skill-editor")
            .h_full()
            .flex_grow()
            .border_l_1()
            .border_color(cx.theme().colors().border)
            .bg(cx.theme().colors().editor_background)
            .children(self.active_skill_id.clone().and_then(|prompt_id| {
                let skill_metadata = self.store.read(cx).metadata(prompt_id.clone())?;
                let skill_editor = &self.skill_editors[&prompt_id];
                let focus_handle = skill_editor.body_editor.focus_handle(cx);
                let registry = LanguageModelRegistry::read_global(cx);
                let model = registry.default_model().map(|default| default.model);
                let built_in = prompt_id.is_built_in();

                Some(
                    v_flex()
                        .id("skill-editor-inner")
                        .size_full()
                        .relative()
                        .overflow_hidden()
                        .on_click(cx.listener(move |_, _, window, cx| {
                            window.focus(&focus_handle, cx);
                        }))
                        .child(
                            h_flex()
                                .group("active-editor-header")
                                .h_12()
                                .px_2()
                                .gap_2()
                                .justify_between()
                                .child(self.render_active_skill_editor(
                                    &skill_editor.title_editor,
                                    built_in,
                                    cx,
                                ))
                                .child(
                                    h_flex()
                                        .h_full()
                                        .flex_shrink_0()
                                        .children(skill_editor.token_count.map(|token_count| {
                                            let token_count: SharedString =
                                                token_count.to_string().into();
                                            let label_token_count: SharedString =
                                                token_count.to_string().into();

                                            div()
                                                .id("token_count")
                                                .mr_1()
                                                .flex_shrink_0()
                                                .tooltip(move |_window, cx| {
                                                    Tooltip::with_meta(
                                                        "Token Estimation",
                                                        None,
                                                        format!(
                                                            "Model: {}",
                                                            model
                                                                .as_ref()
                                                                .map(|model| model.name().0)
                                                                .unwrap_or_default()
                                                        ),
                                                        cx,
                                                    )
                                                })
                                                .child(
                                                    Label::new(format!(
                                                        "{} tokens",
                                                        label_token_count
                                                    ))
                                                    .color(Color::Muted),
                                                )
                                        }))
                                        .map(|this| {
                                            if built_in {
                                                this.child(self.render_built_in_rule_controls())
                                            } else {
                                                this.child(self.render_regular_rule_controls(
                                                    skill_metadata.default,
                                                ))
                                            }
                                        }),
                                ),
                        )
                        .child(
                            div()
                                .on_action(cx.listener(Self::focus_picker))
                                .on_action(cx.listener(Self::inline_assist))
                                .on_action(cx.listener(Self::move_up_from_body))
                                .h_full()
                                .flex_grow()
                                .child(
                                    h_flex()
                                        .py_2()
                                        .pl_2p5()
                                        .h_full()
                                        .flex_1()
                                        .child(skill_editor.body_editor.clone()),
                                ),
                        ),
                )
            }))
    }
}

impl Render for SkillLibrary {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let ui_font = theme::setup_ui_font(window, cx);
        let theme = cx.theme().clone();

        client_side_decorations(
            v_flex()
                .id("Skill library")
                .key_context("SkillLibrary")
                .on_action(cx.listener(|this, &NewSkill, window, cx| this.new_skill(window, cx)))
                .on_action(
                    cx.listener(|this, &DeleteSkill, window, cx| {
                        this.delete_active_skill(window, cx)
                    }),
                )
                .on_action(cx.listener(|this, &DuplicateSkill, window, cx| {
                    this.duplicate_active_skill(window, cx)
                }))
                .on_action(cx.listener(|this, &ToggleDefaultSkill, window, cx| {
                    this.toggle_default_for_active_skill(window, cx)
                }))
                .on_action(cx.listener(|this, &RestoreDefaultContent, window, cx| {
                    this.restore_default_content_for_active_skill(window, cx)
                }))
                .size_full()
                .overflow_hidden()
                .font(ui_font)
                .text_color(theme.colors().text)
                .children(self.title_bar.clone())
                .bg(theme.colors().background)
                .child(
                    h_flex()
                        .flex_1()
                        .when(!cfg!(target_os = "macos"), |this| {
                            this.border_t_1().border_color(cx.theme().colors().border)
                        })
                        .child(self.render_skill_list(cx))
                        .child(self.render_active_skill(cx)),
                ),
            window,
            cx,
        )
    }
}
