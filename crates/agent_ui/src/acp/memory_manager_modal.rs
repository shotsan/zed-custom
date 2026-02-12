use gpui::{WeakEntity, Styled, InteractiveElement, *};
use util::ResultExt;
use ui::*;
use workspace::ModalView;

pub struct MemoryManagerModal {
    thread: Entity<agent::Thread>,
    focus_handle: FocusHandle,
    memories: Vec<agent::Memory>,
}

impl MemoryManagerModal {
    pub fn new(
        thread: Entity<agent::Thread>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let memories = thread.read(cx).memories();
        Self {
            thread,
            focus_handle: cx.focus_handle(),
            memories,
        }
    }

    fn delete_memory(&mut self, id: uuid::Uuid, _window: &mut Window, cx: &mut Context<Self>) {
        let task = self.thread.update(cx, |thread, cx| {
            thread.delete_memory(id, cx)
        });
        
        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            task.await.log_err();
            this.update(cx, |this: &mut Self, cx| {
                this.refresh(cx);
            }).ok();
        }).detach();
    }

    fn refresh(&mut self, cx: &mut Context<Self>) {
        self.memories = self.thread.read(cx).memories();
        cx.notify();
    }
}

impl Focusable for MemoryManagerModal {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<DismissEvent> for MemoryManagerModal {}

impl ModalView for MemoryManagerModal {}

impl Render for MemoryManagerModal {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w(rems(45.))
            .max_h(vh(0.7, window))
            .p_4()
            .gap_3()
            .elevation_3(cx)
            .key_context("MemoryManagerModal")
            .on_action(cx.listener(|_, _: &menu::Cancel, _, cx| cx.emit(DismissEvent)))
            .child(
                h_flex()
                    .justify_between()
                    .child(Headline::new("Project Memories & Rules").size(HeadlineSize::Small))
                    .child(
                        IconButton::new("close", IconName::Close)
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(DismissEvent)))
                    )
            )
            .child(
                v_flex()
                    .gap_2()
                    .when(self.memories.is_empty(), |this: Div| {
                        this.child(Label::new("No project memories or rules yet.").color(Color::Muted))
                    })
                    .children(self.memories.iter().map(|memory| {
                        let id = memory.id;
                        h_flex()
                            .p_2()
                            .bg(cx.theme().colors().element_background)
                            .rounded_md()
                            .justify_between()
                            .gap_2()
                            .child(
                                v_flex()
                                    .flex_1()
                                    .child(
                                        Label::new(format!("{:?}", memory.category))
                                            .size(LabelSize::XSmall)
                                            .color(Color::Accent)
                                    )
                                    .child(
                                        Label::new(memory.content.clone())
                                            .size(LabelSize::Small)
                                    )
                            )
                            .child(
                                IconButton::new(format!("delete-{}", id), IconName::Trash)
                                    .icon_color(Color::Muted)
                                    .icon_size(IconSize::Small)
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.delete_memory(id, window, cx);
                                    }))
                            )
                    }))
            )
            .child(
                h_flex()
                    .justify_end()
                    .child(
                        Button::new("done", "Done")
                            .on_click(cx.listener(|_this, _, _, cx| cx.emit(DismissEvent)))
                    )
            )
    }
}
