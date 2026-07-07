use gpui::*;
use workspace::{dock::Panel, Workspace};
use crate::slack_store::{SlackStore, ConnectionState};
use ui::{prelude::*, Button, ButtonStyle, ButtonSize, TintColor, Icon, IconName, IconPosition, Label, LabelSize, Color};
use editor::Editor;
use menu;

pub struct SlackPanel {
    workspace: WeakEntity<Workspace>,
    focus_handle: FocusHandle,
    input_editor: Entity<Editor>,
    width: Option<Pixels>,
}

actions!(
    slack,
    [
        /// Toggle the visibility of the Slack panel.
        ToggleSlackPanel
    ]
);

impl SlackPanel {
    pub fn new(workspace: WeakEntity<Workspace>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let store = SlackStore::global(cx);
        cx.observe(&store, |_, _, cx| cx.notify()).detach();

        let input_editor = cx.new(|cx| Editor::single_line(window, cx));
        
        Self {
            workspace,
            focus_handle: cx.focus_handle(),
            input_editor,
            width: None,
        }
    }

    pub fn load(
        workspace: WeakEntity<Workspace>,
        cx: AsyncWindowContext,
    ) -> Task<anyhow::Result<Entity<Self>>> {
        cx.spawn(async move |cx| {
            workspace.update_in(cx, |workspace, window, cx| {
                workspace.register_action(|workspace, _: &ToggleSlackPanel, window, cx| {
                    workspace.toggle_panel_focus::<SlackPanel>(window, cx);
                });
                let workspace_weak = cx.entity().downgrade();
                cx.new(|cx| Self::new(workspace_weak, window, cx))
            })
        })
    }

    fn send_message(&mut self, _: &menu::Confirm, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.input_editor.read(cx).text(cx);
        if text.trim().is_empty() {
            return;
        }

        SlackStore::global(cx).update(cx, |store, cx| {
            store.send_message(text, cx);
        });

        self.input_editor.update(cx, |editor, cx| {
            editor.set_text("", window, cx);
        });
    }
}

impl Focusable for SlackPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Panel for SlackPanel {
    fn persistent_name() -> &'static str {
        "SlackPanel"
    }

    fn panel_key() -> &'static str {
        "SlackPanel"
    }

    fn activation_priority(&self) -> u32 {
        0
    }

    fn position(&self, _window: &Window, _cx: &App) -> workspace::dock::DockPosition {
        workspace::dock::DockPosition::Right
    }

    fn position_is_valid(&self, _position: workspace::dock::DockPosition) -> bool {
        true
    }

    fn set_position(&mut self, _position: workspace::dock::DockPosition, _window: &mut Window, _cx: &mut Context<Self>) {
    }

    fn size(&self, _window: &Window, _cx: &App) -> Pixels {
        self.width.unwrap_or(px(300.))
    }

    fn set_size(&mut self, size: Option<Pixels>, _window: &mut Window, _cx: &mut Context<Self>) {
        self.width = size;
    }

    fn icon(&self, _window: &Window, _cx: &App) -> Option<ui::IconName> {
        Some(ui::IconName::Hash)
    }

    fn icon_tooltip(&self, _window: &Window, _cx: &App) -> Option<&'static str> {
        Some("Slack")
    }

    fn toggle_action(&self) -> Box<dyn Action> {
        Box::new(ToggleSlackPanel)
    }
}

impl EventEmitter<workspace::dock::PanelEvent> for SlackPanel {}

impl Render for SlackPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let store = SlackStore::global(cx);
        let _store_handle = store.clone();
        let store_ref = store.read(cx);

        let status_text = match store_ref.connection_state() {
            ConnectionState::Connected => "Connected",
            ConnectionState::Connecting => "Connecting...",
            ConnectionState::Disconnected => "Disconnected",
        };

        let status_color = match store_ref.connection_state() {
            ConnectionState::Connected => gpui::rgb(0x10b981), // Emerald 500
            ConnectionState::Connecting => gpui::rgb(0xf59e0b), // Amber 500
            ConnectionState::Disconnected => gpui::rgb(0xef4444), // Red 500
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(cx.theme().colors().panel_background)
            .child(
                div()
                    .w_full()
                    .p_4()
                    .flex()
                    .items_center()
                    .justify_between()
                    .border_b_1()
                    .border_color(cx.theme().colors().border)
                    .child(
                        div()
                            .text_color(cx.theme().colors().text)
                            .font_weight(FontWeight::BOLD)
                            .child("Slack (#general)"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .w_2()
                                    .h_2()
                                    .rounded_full()
                                    .bg(status_color),
                            )
                            .child(
                                div()
                                    .text_color(gpui::rgb(0x94a3b8)) // Slate 400
                                    .text_size(px(12.))
                                    .child(status_text),
                            ),
                    ),
            )
            .child(
                if *store_ref.connection_state() == ConnectionState::Disconnected {
                    let theme = cx.theme().colors();
                    
                    v_flex()
                        .id("slack-login")
                        .flex_1()
                        .items_center()
                        .justify_center()
                        .p_6()
                        .child(
                            v_flex()
                                .bg(theme.elevated_surface_background)
                                .border_1()
                                .border_color(theme.border)
                                .rounded_md()
                                .p_6()
                                .w_full()
                                .max_w(px(320.))
                                .items_center()
                                .gap_6()
                                .child(
                                    div()
                                        .bg(theme.element_background)
                                        .p_3()
                                        .rounded_full()
                                        .child(
                                            Icon::new(IconName::Hash)
                                                .color(Color::Accent)
                                                .size(IconSize::XLarge)
                                        )
                                )
                                .child(
                                    v_flex()
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            Label::new("Connect to Slack")
                                                .size(LabelSize::Large)
                                                .weight(FontWeight::BOLD)
                                                .color(Color::Default)
                                        )
                                        .child(
                                            Label::new("Authenticate to securely access your workspace.")
                                                .color(Color::Muted)
                                                .size(LabelSize::Small)
                                        )
                                )
                                .child(
                                    v_flex()
                                        .w_full()
                                        .gap_3()
                                        .when_some(store_ref.token(), |this, saved_token| {
                                            this.child(
                                                Button::new("reconnect_keychain_btn", "Reconnect (Keychain)")
                                                    .icon(IconName::Link)
                                                    .style(ButtonStyle::Tinted(TintColor::Accent))
                                                    .size(ButtonSize::Large)
                                                    .full_width()
                                                    .on_click(cx.listener(move |this, _, _window, cx| {
                                                        SlackStore::global(cx).update(cx, |store, cx| store.connect(this.workspace.clone(), saved_token.clone(), cx));
                                                    }))
                                            )
                                        })
                                        .when_some(store_ref.error_message(), |this, err| {
                                            this.child(
                                                div()
                                                    .p_3()
                                                    .bg(gpui::rgb(0x7f1d1d)) // Red 900
                                                    .border_1()
                                                    .border_color(gpui::rgb(0xef4444)) // Red 500
                                                    .rounded_md()
                                                    .child(
                                                        div()
                                                            .text_color(gpui::rgb(0xfca5a5)) // Red 300
                                                            .text_size(px(12.))
                                                            .child(err.clone())
                                                    )
                                            )
                                        })
                                        .child(
                                            Button::new("connect_slack_btn", "Authenticate with Slack")
                                                .icon(IconName::ArrowUpRight)
                                                .icon_position(IconPosition::End)
                                                .style(if store_ref.token().is_some() { ButtonStyle::Filled } else { ButtonStyle::Tinted(TintColor::Accent) })
                                                .size(ButtonSize::Large)
                                                .full_width()
                                                .on_click(cx.listener(|_, _, _window, cx| {
                                                    let azure_app_url = "https://zed-slack-backend.wittygrass-2327b171.eastus2.azurecontainerapps.io/slack/install";
                                                    cx.open_url(azure_app_url);
                                                }))
                                        )
                                        .child(
                                            Button::new("paste_token_btn", "Paste Token & Connect")
                                                .icon(IconName::Copy)
                                                .style(ButtonStyle::Filled)
                                                .size(ButtonSize::Large)
                                                .full_width()
                                                .on_click(cx.listener(|this, _, _window, cx| {
                                                    if let Some(clipboard) = cx.read_from_clipboard() {
                                                        if let Some(token) = clipboard.text() {
                                                            SlackStore::global(cx).update(cx, |store, cx| store.connect(this.workspace.clone(), token, cx));
                                                        }
                                                    }
                                                }))
                                        )
                                )
                        ).into_any_element()
                } else if *store_ref.connection_state() == ConnectionState::Connecting {
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .text_color(gpui::rgb(0xcbd5e1))
                        .child("Connecting to Slack backend...")
                        .into_any_element()
                } else {
                    let messages = store_ref.messages();
                    
                    let messages_view = if messages.is_empty() {
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .text_color(gpui::rgb(0x94a3b8))
                            .child("No messages yet.")
                            .into_any_element()
                    } else {
                        let messages_list = div()
                            .id("messages-scroll")
                            .flex_1()
                            .h_full()
                            .overflow_y_scroll()
                            .p_4()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .children(messages.iter().map(|msg| {
                                let is_me = !msg.is_incoming;
                                
                                let bubble_bg = if is_me {
                                    cx.theme().colors().element_active
                                } else {
                                    cx.theme().colors().elevated_surface_background
                                };
                                
                                div()
                                    .flex()
                                    .flex_col()
                                    .map(|this| {
                                        if is_me {
                                            this.items_end()
                                        } else {
                                            this.items_start()
                                        }
                                    })
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .mb_1()
                                            .child(
                                                div()
                                                    .text_size(px(11.))
                                                    .text_color(cx.theme().colors().text_muted)
                                                    .child(msg.user.clone())
                                            )
                                            .child(
                                                div()
                                                    .text_size(px(10.))
                                                    .text_color(cx.theme().colors().text_disabled)
                                                    .child(msg.timestamp.clone())
                                            )
                                    )
                                    .child(
                                        div()
                                            .p_3()
                                            .bg(bubble_bg)
                                            .border_1()
                                            .border_color(cx.theme().colors().border)
                                            .rounded_lg()
                                            .text_size(px(13.))
                                            .text_color(cx.theme().colors().text)
                                            .child(msg.text.clone())
                                    )
                            }));
                        
                        messages_list.into_any_element()
                    };

                    v_flex()
                        .flex_1()
                        .child(messages_view)
                        .when_some(store_ref.error_message(), |this, err| {
                            this.child(
                                div()
                                    .p_2()
                                    .bg(gpui::rgb(0xfef2f2)) // red 50
                                    .text_color(gpui::rgb(0xef4444)) // red 500
                                    .text_size(px(12.))
                                    .child(err.clone())
                            )
                        })
                        .child(
                            div()
                                .p_4()
                                .border_t_1()
                                .border_color(cx.theme().colors().border)
                                .bg(cx.theme().colors().panel_background)
                                .on_action(cx.listener(Self::send_message))
                                .child(
                                    div()
                                        .bg(cx.theme().colors().editor_background)
                                        .border_1()
                                        .border_color(cx.theme().colors().border)
                                        .rounded_md()
                                        .px_3()
                                        .py_2()
                                        .child(self.input_editor.clone())
                                )
                        )
                        .into_any_element()
                }
            )
    }
}

pub fn init(cx: &mut App) {
    cx.observe_new(|workspace: &mut Workspace, _, _| {
        workspace.register_action(|workspace, _: &ToggleSlackPanel, window, cx| {
            workspace.toggle_panel_focus::<SlackPanel>(window, cx);
        });
    })
    .detach();
}
