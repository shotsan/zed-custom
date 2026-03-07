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
        let existing = thread.read(cx).custom_system_prompt_template().map(|s| s.to_string());
        
        let initial_text = existing.unwrap_or_else(|| agent::Templates::system_prompt_source());

        let input = cx.new(|cx| {
            let mut editor = editor::Editor::multi_line(window, cx);
            editor.set_text(initial_text, window, cx);
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
        let template = Some(text);
        self.thread.update(cx, |thread, cx| {
            thread.set_custom_system_prompt_template(template, cx);
        });
        cx.emit(DismissEvent);
    }

    fn clear_and_save(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.thread.update(cx, |thread, cx| {
            thread.set_custom_system_prompt_template(None, cx);
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
        let has_override = self.thread.read(cx).custom_system_prompt_template().is_some();

        v_flex()
            .w(rems(64.))
            .p_4()
            .gap_3()
            .elevation_3(cx)
            .key_context("CustomPromptModal")
            .on_action(cx.listener(|_this, _: &menu::Cancel, _, cx| cx.emit(DismissEvent)))
            .on_action(cx.listener(|this, _: &menu::Confirm, _window, cx| {
                let text = this.input.read(cx).text(cx);
                this.thread.update(cx, |thread, cx| {
                    thread.set_custom_system_prompt_template(Some(text), cx);
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
                            .child(Headline::new("System Prompt Template Editor").size(HeadlineSize::Small)),
                    ),
            )
            .child(
                Label::new(
                    "Edit the complete system prompt for this thread. \
                     Keep the {{...}} tags if you want the agent's sensors (active files, errors, memories) to remain dynamic.",
                )
                .size(LabelSize::Small)
                .color(Color::Muted),
            )
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        div()
                            .h(rems(30.))
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
                        h_flex().when(has_override, |this| {
                            this.child(
                                Button::new("clear", "Reset to Default")
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
                                Button::new("save", "Apply Template")
                                    .style(ButtonStyle::Filled)
                                    .on_click(cx.listener(Self::save)),
                            ),
                    ),
            )
    }
}
