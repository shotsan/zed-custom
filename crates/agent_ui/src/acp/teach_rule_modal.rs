use gpui::*;
use ui::*;
use workspace::ModalView;

pub struct TeachRuleModal {
    thread: Entity<agent::Thread>,
    input: Entity<editor::Editor>,
    focus_handle: FocusHandle,
}

impl TeachRuleModal {
    pub fn new(
        thread: Entity<agent::Thread>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let input = cx.new(|cx| {
            let mut editor = editor::Editor::single_line(window, cx);
            editor.set_placeholder_text("Enter your rule (e.g., 'this project converts emails to PDF')", window, cx);
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
        let text = text.trim();
        if !text.is_empty() {
            self.thread.update(cx, |thread, cx| {
                thread.remember_rule(text, cx);
            });
        }
        cx.emit(DismissEvent);
    }
    
    fn cancel(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        cx.emit(DismissEvent);
    }
}

impl Focusable for TeachRuleModal {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<DismissEvent> for TeachRuleModal {}

impl ModalView for TeachRuleModal {}

impl Render for TeachRuleModal {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w(rems(40.))
            .p_4()
            .gap_3()
            .elevation_3(cx)
            .key_context("TeachRuleModal")
            .on_action(cx.listener(|_this, _: &menu::Cancel, _, cx| cx.emit(DismissEvent)))
            .on_action(cx.listener(|this, _: &menu::Confirm, _window, cx| {
                let text = this.input.read(cx).text(cx);
                let text = text.trim();
                if !text.is_empty() {
                    this.thread.update(cx, |thread, cx| {
                        thread.remember_rule(text, cx);
                    });
                }
                cx.emit(DismissEvent);
            }))
            .child(
                h_flex()
                    .justify_between()
                    .child(Headline::new("Teach me a new project rule").size(HeadlineSize::Small))
            )
            .child(
                v_flex()
                    .gap_2()
                    .child(Label::new("Enter a fact, preference, or architectural rule about this project:"))
                    .child(self.input.clone())
            )
            .child(
                h_flex()
                    .justify_end()
                    .gap_2()
                    .child(
                        Button::new("cancel", "Cancel")
                            .on_click(cx.listener(Self::cancel))
                    )
                    .child(
                        Button::new("save", "Save")
                            .style(ButtonStyle::Filled)
                            .on_click(cx.listener(Self::save))
                    )
            )
    }
}
