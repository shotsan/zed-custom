use gpui::*;
use ui::*;
use workspace::ModalView;

pub struct CustomPromptModal {
    thread: Entity<agent::Thread>,
    input: Entity<editor::Editor>,
    focus_handle: FocusHandle,
}

impl CustomPromptModal {
    pub fn new(
        thread: Entity<agent::Thread>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let existing = thread.read(cx).custom_instructions().map(|s| s.to_string()).unwrap_or_default();

        let input = cx.new(|cx| {
            let mut editor = editor::Editor::multi_line(window, cx);
            editor.set_placeholder_text(
                "Enter additional instructions for the AI (e.g., 'Always use TypeScript', 'Respond in Spanish')...",
                window,
                cx,
            );
            if !existing.is_empty() {
                editor.set_text(existing, window, cx);
            }
            editor
        });

        Self {
            thread,
            input,
            focus_handle: cx.focus_handle(),
        }
    }

    fn save(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let text = self.input.read(cx).text(cx);
        let text = text.trim().to_string();
        let instructions = if text.is_empty() { None } else { Some(text) };
        self.thread.update(cx, |thread, cx| {
            thread.set_custom_instructions(instructions, cx);
        });
        cx.emit(DismissEvent);
    }

    fn clear_and_save(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.thread.update(cx, |thread, cx| {
            thread.set_custom_instructions(None, cx);
        });
        cx.emit(DismissEvent);
    }

    fn cancel(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        cx.emit(DismissEvent);
    }
}

impl Focusable for CustomPromptModal {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<DismissEvent> for CustomPromptModal {}

impl ModalView for CustomPromptModal {}

impl Render for CustomPromptModal {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let has_existing = self.thread.read(cx).custom_instructions().is_some();

        v_flex()
            .w(rems(48.))
            .p_4()
            .gap_3()
            .elevation_3(cx)
            .key_context("CustomPromptModal")
            .on_action(cx.listener(|_this, _: &menu::Cancel, _, cx| cx.emit(DismissEvent)))
            .on_action(cx.listener(|this, _: &menu::Confirm, _window, cx| {
                let text = this.input.read(cx).text(cx);
                let text = text.trim().to_string();
                let instructions = if text.is_empty() { None } else { Some(text) };
                this.thread.update(cx, |thread, cx| {
                    thread.set_custom_instructions(instructions, cx);
                });
                cx.emit(DismissEvent);
            }))
            .child(
                h_flex()
                    .justify_between()
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Icon::new(IconName::Sparkle)
                                    .size(IconSize::Medium)
                                    .color(Color::Accent),
                            )
                            .child(Headline::new("Custom System Instructions").size(HeadlineSize::Small)),
                    ),
            )
            .child(
                Label::new(
                    "These instructions are injected into every system prompt for this session. \
                     Use them to set the AI's personality, constraints, or project-specific rules.",
                )
                .size(LabelSize::Small)
                .color(Color::Muted),
            )
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        div()
                            .h(rems(10.))
                            .w_full()
                            .rounded_md()
                            .border_1()
                            .border_color(cx.theme().colors().border)
                            .bg(cx.theme().colors().editor_background)
                            .p_2()
                            .child(self.input.clone()),
                    ),
            )
            .child(
                h_flex()
                    .justify_between()
                    .gap_2()
                    .child(
                        h_flex().when(has_existing, |this| {
                            this.child(
                                Button::new("clear", "Clear Instructions")
                                    .style(ButtonStyle::Subtle)
                                    .color(Color::Error)
                                    .on_click(cx.listener(Self::clear_and_save)),
                            )
                        }),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(Button::new("cancel", "Cancel").on_click(cx.listener(Self::cancel)))
                            .child(
                                Button::new("save", "Apply Instructions")
                                    .style(ButtonStyle::Filled)
                                    .on_click(cx.listener(Self::save)),
                            ),
                    ),
            )
    }
}
