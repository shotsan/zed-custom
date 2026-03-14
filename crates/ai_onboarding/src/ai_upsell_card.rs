use std::sync::Arc;

use client::{Client, UserStore};
use cloud_api_types::Plan;
use gpui::{Action, AnyElement, App, Entity, IntoElement, RenderOnce, Window};
use ui::prelude::*;

#[derive(IntoElement, RegisterComponent)]
pub struct AiUpsellCard {
    tab_index: Option<isize>,
}

impl AiUpsellCard {
    pub fn new(
        _client: Arc<Client>,
        _user_store: &Entity<UserStore>,
        _user_plan: Option<Plan>,
        _cx: &mut App,
    ) -> Self {
        Self {
            tab_index: None,
        }
    }

    pub fn tab_index(mut self, tab_index: Option<isize>) -> Self {
        self.tab_index = tab_index;
        self
    }
}

impl RenderOnce for AiUpsellCard {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let card = v_flex()
            .relative()
            .flex_grow()
            .p_4()
            .pt_3()
            .border_1()
            .border_color(cx.theme().colors().border)
            .rounded_lg()
            .overflow_hidden();

        card.child(Label::new("Zed AI").size(LabelSize::Large))
            .child(
                div()
                    .max_w_3_4()
                    .mb_2()
                    .child(Label::new("Add your keys to get unlimited coding.").color(Color::Muted)),
            )
            .child(
                Button::new("configure_keys", "Configure Keys")
                    .full_width()
                    .style(ButtonStyle::Tinted(ui::TintColor::Accent))
                    .when_some(self.tab_index, |this, tab_index| this.tab_index(tab_index))
                    .on_click({
                        move |_, window, cx| {
                            window.dispatch_action(zed_custom_actions::agent::OpenSettings.boxed_clone(), cx);
                        }
                    }),
            )
    }
}

impl Component for AiUpsellCard {
    fn scope() -> ComponentScope {
        ComponentScope::Onboarding
    }

    fn name() -> &'static str {
        "AI Upsell Card"
    }

    fn sort_name() -> &'static str {
        "AI Upsell Card"
    }

    fn description() -> Option<&'static str> {
        Some("A card presenting the Zed AI product during user's first-open onboarding flow.")
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            v_flex()
                .gap_4()
                .items_center()
                .max_w_4_5()
                .child(single_example(
                    "Signed Out State",
                    AiUpsellCard {
                        tab_index: Some(0),
                    }
                    .into_any_element(),
                ))
                .child(example_group_with_title(
                    "Signed In States",
                    vec![
                        single_example(
                            "Free Plan",
                            AiUpsellCard {
                                tab_index: Some(1),
                            }
                            .into_any_element(),
                        ),
                        single_example(
                            "Free Plan but Young Account",
                            AiUpsellCard {
                                tab_index: Some(1),
                            }
                            .into_any_element(),
                        ),
                        single_example(
                            "Pro Trial",
                            AiUpsellCard {
                                tab_index: Some(1),
                            }
                            .into_any_element(),
                        ),
                        single_example(
                            "Pro Plan",
                            AiUpsellCard {
                                tab_index: Some(1),
                            }
                            .into_any_element(),
                        ),
                    ],
                ))
                .into_any_element(),
        )
    }
}
