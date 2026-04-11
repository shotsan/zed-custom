use anyhow::{anyhow, Result};
use futures::{FutureExt, StreamExt, future::BoxFuture};
use gpui::{AnyView, App, AsyncApp, Context, Entity, SharedString, Task, Window};
use http_client::HttpClient;
use language_model::{
    ApiKeyState, AuthenticateError, EnvVar, IconOrSvg, LanguageModel, LanguageModelCompletionError,
    LanguageModelCompletionEvent, LanguageModelId, LanguageModelName, LanguageModelProvider,
    LanguageModelProviderId, LanguageModelProviderName, LanguageModelProviderState,
    LanguageModelRequest, LanguageModelToolChoice, LanguageModelToolSchemaFormat, RateLimiter, env_var,
};
use menu;
use open_ai::{
    ResponseStreamEvent,
    responses::{Request as ResponseRequest, StreamEvent as ResponsesStreamEvent, stream_response},
    stream_completion,
};
use settings::{Settings, SettingsStore, update_settings_file};
use std::sync::{Arc, LazyLock};
use fs::Fs;

use ui::{prelude::*, ButtonLink, List, ListBulletItem};
use ui_input::InputField;
use util::ResultExt;

use crate::provider::open_ai::{
    OpenAiEventMapper, OpenAiResponseEventMapper, into_open_ai, into_open_ai_response,
};
pub use settings::AzureOpenAiAvailableModel as AvailableModel;
pub use settings::OpenAiModelCapabilities as ModelCapabilities;
pub use settings::AzureOpenAiSettingsContent as AzureOpenAiSettings;

const PROVIDER_ID: LanguageModelProviderId = language_model::AZURE_OPENAI_PROVIDER_ID;
const PROVIDER_NAME: LanguageModelProviderName = language_model::AZURE_OPENAI_PROVIDER_NAME;

const API_KEY_ENV_VAR_NAME: &str = "AZURE_OPENAI_API_KEY";
static API_KEY_ENV_VAR: LazyLock<EnvVar> = env_var!(API_KEY_ENV_VAR_NAME);

pub struct AzureOpenAiLanguageModelProvider {
    http_client: Arc<dyn HttpClient>,
    state: Entity<State>,
}

pub struct State {
    api_key_state: ApiKeyState,
}

impl State {
    fn is_authenticated(&self) -> bool {
        self.api_key_state.has_key()
    }

    fn set_api_key(&mut self, api_key: Option<String>, cx: &mut Context<Self>) -> Task<Result<()>> {
        let api_url = AzureOpenAiLanguageModelProvider::api_url(cx);
        self.api_key_state
            .store(api_url, api_key, |this| &mut this.api_key_state, cx)
    }

    fn authenticate(&mut self, cx: &mut Context<Self>) -> Task<Result<(), AuthenticateError>> {
        let api_url = AzureOpenAiLanguageModelProvider::api_url(cx);
        self.api_key_state
            .load_if_needed(api_url, |this| &mut this.api_key_state, cx)
    }
}

impl AzureOpenAiLanguageModelProvider {
    pub fn new(http_client: Arc<dyn HttpClient>, cx: &mut App) -> Self {
        let state = cx.new(|cx| {
            cx.observe_global::<SettingsStore>(|this: &mut State, cx| {
                let api_url = Self::api_url(cx);
                this.api_key_state
                    .handle_url_change(api_url, |this| &mut this.api_key_state, cx);
                cx.notify();
            })
            .detach();
            State {
                api_key_state: ApiKeyState::new(Self::api_url(cx), (*API_KEY_ENV_VAR).clone()),
            }
        });

        Self { http_client, state }
    }

    fn create_language_model(&self, model: AvailableModel) -> Arc<dyn LanguageModel> {
        Arc::new(AzureOpenAiLanguageModel {
            id: LanguageModelId::from(model.name.clone()),
            model,
            state: self.state.clone(),
            http_client: self.http_client.clone(),
            request_limiter: RateLimiter::new(4),
        })
    }

    fn settings(cx: &App) -> AzureOpenAiSettings {
        crate::AllLanguageModelSettings::get_global(cx).azure_openai.clone()
    }

    fn api_url(cx: &App) -> SharedString {
        let api_url = Self::settings(cx).api_url.unwrap_or_default();
        SharedString::new(api_url)
    }

    fn api_version(cx: &App) -> SharedString {
        let api_version = Self::settings(cx).api_version.unwrap_or_default();
        SharedString::new(api_version)
    }
}

impl LanguageModelProviderState for AzureOpenAiLanguageModelProvider {
    type ObservableEntity = State;

    fn observable_entity(&self) -> Option<Entity<Self::ObservableEntity>> {
        Some(self.state.clone())
    }
}

impl LanguageModelProvider for AzureOpenAiLanguageModelProvider {
    fn id(&self) -> LanguageModelProviderId {
        PROVIDER_ID
    }

    fn name(&self) -> LanguageModelProviderName {
        PROVIDER_NAME
    }

    fn icon(&self) -> IconOrSvg {
        IconOrSvg::Icon(IconName::AiOpenAi)
    }

    fn default_model(&self, cx: &App) -> Option<Arc<dyn LanguageModel>> {
        Self::settings(cx)
            .available_models
            .unwrap_or_default()
            .first()
            .map(|model| self.create_language_model(model.clone()))
    }

    fn default_fast_model(&self, _cx: &App) -> Option<Arc<dyn LanguageModel>> {
        None
    }

    fn provided_models(&self, cx: &App) -> Vec<Arc<dyn LanguageModel>> {
        Self::settings(cx)
            .available_models
            .unwrap_or_default()
            .iter()
            .map(|model| self.create_language_model(model.clone()))
            .collect()
    }

    fn is_authenticated(&self, cx: &App) -> bool {
        self.state.read(cx).is_authenticated()
    }

    fn authenticate(&self, cx: &mut App) -> Task<Result<(), AuthenticateError>> {
        self.state.update(cx, |state, cx| state.authenticate(cx))
    }

    fn configuration_view(
        &self,
        _target_agent: language_model::ConfigurationViewTargetAgent,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyView {
        cx.new(|cx| ConfigurationView::new(self.state.clone(), _target_agent, window, cx))
            .into()
    }

    fn reset_credentials(&self, cx: &mut App) -> Task<Result<()>> {
        self.state
            .update(cx, |state, cx| state.set_api_key(None, cx))
    }
}

pub struct AzureOpenAiLanguageModel {
    id: LanguageModelId,
    model: AvailableModel,
    state: Entity<State>,
    http_client: Arc<dyn HttpClient>,
    request_limiter: RateLimiter,
}

impl AzureOpenAiLanguageModel {
    fn stream_completion(
        &self,
        request: open_ai::Request,
        cx: &AsyncApp,
    ) -> BoxFuture<
        'static,
        Result<
            futures::stream::BoxStream<'static, Result<ResponseStreamEvent>>,
            LanguageModelCompletionError,
        >,
    > {
        let http_client = self.http_client.clone();

        let (api_key, api_url) = self.state.read_with(cx, |state, _cx| {
            let base_url = AzureOpenAiLanguageModelProvider::api_url(_cx);
            let api_version = AzureOpenAiLanguageModelProvider::api_version(_cx);
            let deployment_name = &self.model.name;

            let mut full_url = base_url.trim_end_matches('/').to_string();
            if !full_url.contains("/openai/deployments/") {
                full_url.push_str("/openai/deployments/");
                full_url.push_str(deployment_name);
            }
            if !api_version.is_empty() {
                full_url.push_str("?api-version=");
                full_url.push_str(&api_version);
            }

            (
                state.api_key_state.key(&base_url),
                SharedString::from(full_url),
            )
        });

        let provider = PROVIDER_NAME.clone();
        if request.messages.is_empty() {
            return async move { Err(anyhow!("messages must not be empty").into()) }.boxed();
        }
        let future = self.request_limiter.stream(async move {
            let Some(api_key) = api_key else {
                return Err(LanguageModelCompletionError::NoApiKey { provider });
            };
            let request = stream_completion(
                http_client.as_ref(),
                provider.0.as_str(),
                &api_url,
                &api_key,
                request,
            );
            let response = request.await?;
            Ok(response)
        });

        async move { Ok(future.await?.boxed()) }.boxed()
    }

    fn stream_response(
        &self,
        request: ResponseRequest,
        cx: &AsyncApp,
    ) -> BoxFuture<'static, Result<futures::stream::BoxStream<'static, Result<ResponsesStreamEvent>>>>
    {
        let http_client = self.http_client.clone();

        let (api_key, api_url) = self.state.read_with(cx, |state, _cx| {
            let base_url = AzureOpenAiLanguageModelProvider::api_url(_cx);
            let api_version = AzureOpenAiLanguageModelProvider::api_version(_cx);

            let mut full_url = base_url.trim_end_matches('/').to_string();
            // Azure's Responses API is at the resource level, not deployment-scoped.
            // Correct: {base}/openai/responses?api-version=...
            if !full_url.ends_with("/openai") {
                full_url.push_str("/openai");
            }
            full_url.push_str("/responses");
            if !api_version.is_empty() {
                full_url.push_str("?api-version=");
                full_url.push_str(&api_version);
            }

            (
                state.api_key_state.key(&base_url),
                SharedString::from(full_url),
            )
        });

        let provider = PROVIDER_NAME.clone();
        if request.input.is_empty() {
            return async move { Err(anyhow!("messages must not be empty").into()) }.boxed();
        }
        let future = self.request_limiter.stream(async move {
            let Some(api_key) = api_key else {
                return Err(LanguageModelCompletionError::NoApiKey { provider });
            };
            let request = stream_response(
                http_client.as_ref(),
                provider.0.as_str(),
                &api_url,
                &api_key,
                request,
            );
            let response = request.await?;
            Ok(response)
        });

        async move { Ok(future.await?.boxed()) }.boxed()
    }
}

impl LanguageModel for AzureOpenAiLanguageModel {
    fn id(&self) -> LanguageModelId {
        self.id.clone()
    }

    fn name(&self) -> LanguageModelName {
        LanguageModelName::from(
            self.model
                .display_name
                .clone()
                .unwrap_or_else(|| self.model.name.clone()),
        )
    }

    fn provider_id(&self) -> LanguageModelProviderId {
        PROVIDER_ID
    }

    fn provider_name(&self) -> LanguageModelProviderName {
        PROVIDER_NAME
    }

    fn supports_tools(&self) -> bool {
        true
    }

    fn tool_input_format(&self) -> LanguageModelToolSchemaFormat {
        LanguageModelToolSchemaFormat::JsonSchemaSubset
    }

    fn supports_images(&self) -> bool {
        false
    }

    fn supports_tool_choice(&self, _choice: LanguageModelToolChoice) -> bool {
        true
    }

    fn supports_split_token_display(&self) -> bool {
        true
    }

    fn telemetry_id(&self) -> String {
        format!("azure_openai/{}", self.model.name)
    }

    fn max_token_count(&self) -> u64 {
        self.model.max_tokens
    }

    fn max_output_tokens(&self) -> Option<u64> {
        self.model.max_output_tokens
    }

    fn count_tokens(
        &self,
        request: LanguageModelRequest,
        cx: &App,
    ) -> BoxFuture<'static, Result<u64>> {
        let max_token_count = self.max_token_count();
        cx.background_spawn(async move {
            let messages = crate::provider::open_ai::collect_tiktoken_messages(request);
            let model = if max_token_count >= 100_000 {
                "gpt-4o"
            } else {
                "gpt-4"
            };
            tiktoken_rs::num_tokens_from_messages(model, &messages).map(|tokens| tokens as u64)
        })
        .boxed()
    }

    fn stream_completion(
        &self,
        request: LanguageModelRequest,
        cx: &AsyncApp,
    ) -> BoxFuture<
        'static,
        Result<
            futures::stream::BoxStream<
                'static,
                Result<LanguageModelCompletionEvent, LanguageModelCompletionError>,
            >,
            LanguageModelCompletionError,
        >,
    > {
        if self.model.capabilities.chat_completions {
            let request = into_open_ai(
                request,
                &self.model.name,
                true,
                false,
                self.max_output_tokens(),
                self.model.reasoning_effort.clone(),
            );
            let completions = self.stream_completion(request, cx);
            async move {
                let mapper = OpenAiEventMapper::new();
                Ok(mapper.map_stream(completions.await?).boxed())
            }
            .boxed()
        } else {
            let request = into_open_ai_response(
                request,
                &self.model.name,
                true,
                false,
                self.max_output_tokens(),
                self.model.reasoning_effort.clone(),
            );
            let completions = self.stream_response(request, cx);
            async move {
                let mapper = OpenAiResponseEventMapper::new();
                Ok(mapper.map_stream(completions.await?).boxed())
            }
            .boxed()
        }
    }
}

use language_model::ConfigurationViewTargetAgent;

struct ConfigurationView {
    api_key_editor: Entity<InputField>,
    api_url_editor: Entity<InputField>,
    api_version_editor: Entity<InputField>,
    deployment_name_editor: Entity<InputField>,
    max_tokens_editor: Entity<InputField>,
    max_output_tokens_editor: Entity<InputField>,
    state: Entity<State>,
    load_credentials_task: Option<Task<()>>,
    target_agent: ConfigurationViewTargetAgent,
    chat_completions: bool,
    tool_use: bool,
}

impl ConfigurationView {
    fn new(state: Entity<State>, target_agent: ConfigurationViewTargetAgent, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let api_key_editor = cx.new(|cx| {
            InputField::new(
                window,
                cx,
                "000000000000000000000000000000000000000000000000000",
            ).label("API Key")
        });

        let api_url_editor = cx.new(|cx| {
            let input = InputField::new(window, cx, "https://<resource>.cognitiveservices.azure.com/").label("API URL");
            input.set_text(&AzureOpenAiLanguageModelProvider::api_url(cx), window, cx);
            input
        });

        let api_version_editor = cx.new(|cx| {
            let input = InputField::new(window, cx, "2025-04-01-preview").label("API Version");
            input.set_text(&AzureOpenAiLanguageModelProvider::api_version(cx), window, cx);
            input
        });

        let model = AzureOpenAiLanguageModelProvider::settings(cx)
            .available_models
            .unwrap_or_default()
            .first()
            .cloned();

        let deployment_name_editor = cx.new(|cx| {
            let editor = InputField::new(window, cx, "gpt-4o").label("Deployment Name");
            if let Some(model) = &model {
                editor.set_text(&model.name, window, cx);
            }
            editor
        });

        let max_tokens_editor = cx.new(|cx| {
            let editor = InputField::new(window, cx, "128000").label("Max Tokens");
            if let Some(model) = &model {
                editor.set_text(&model.max_tokens.to_string(), window, cx);
            }
            editor
        });

        let max_output_tokens_editor = cx.new(|cx| {
            let editor = InputField::new(window, cx, "16384").label("Max Output Tokens");
            if let Some(model) = &model {
                if let Some(max_output_tokens) = model.max_output_tokens {
                    editor.set_text(&max_output_tokens.to_string(), window, cx);
                }
            }
            editor
        });

        cx.observe(&state, |_, _, cx| {
            cx.notify();
        })
        .detach();

        let load_credentials_task = Some(cx.spawn_in(window, {
            let state = state.clone();
            async move |this, cx| {
                if let Some(task) = Some(state.update(cx, |state, cx| state.authenticate(cx))) {
                    let _ = task.await;
                }
                this.update(cx, |this, cx| {
                    this.load_credentials_task = None;
                    cx.notify();
                })
                .log_err();
            }
        }));

        Self {
            api_key_editor,
            api_url_editor,
            api_version_editor,
            deployment_name_editor,
            max_tokens_editor,
            max_output_tokens_editor,
            state,
            load_credentials_task,
            target_agent,
            chat_completions: model.as_ref().map_or(true, |m| m.capabilities.chat_completions),
            tool_use: model.as_ref().map_or(true, |m| m.capabilities.tool_use),
        }
    }

    fn save_settings(&mut self, _: &menu::Confirm, window: &mut Window, cx: &mut Context<Self>) {
        let api_key = self.api_key_editor.read(cx).text(cx);
        let api_url = self.api_url_editor.read(cx).text(cx).trim().to_string();
        let api_version = self.api_version_editor.read(cx).text(cx).trim().to_string();
        let deployment_name = self.deployment_name_editor.read(cx).text(cx).trim().to_string();
        let max_tokens = self.max_tokens_editor.read(cx).text(cx).trim().parse::<u64>().unwrap_or(128000);
        let max_output_tokens = self.max_output_tokens_editor.read(cx).text(cx).trim().parse::<u64>().ok();

        if !api_key.is_empty() {
            let state = self.state.clone();
            cx.spawn_in(window, async move |_, cx| {
                state
                    .update(cx, |state, cx| state.set_api_key(Some(api_key), cx))
                    .await
            })
            .detach_and_log_err(cx);
            self.api_key_editor
                .update(cx, |editor, cx| editor.set_text("", window, cx));
        }

        if !api_url.is_empty() && !deployment_name.is_empty() {
            let target_agent = self.target_agent.clone();
            let chat_completions = self.chat_completions;
            let tool_use = self.tool_use;
            let fs = <dyn Fs>::global(cx);
            update_settings_file(fs, cx, move |settings, _| {
                let language_models = settings.language_models.get_or_insert_default();
                let azure = language_models.azure_openai.get_or_insert_default();
                
                azure.api_url = Some(api_url);
                
                if !api_version.is_empty() {
                    azure.api_version = Some(api_version);
                }
                
                let models = azure.available_models.get_or_insert_default();
                if let Some(model) = models.first_mut() {
                    model.name = deployment_name.clone();
                    model.display_name = Some(deployment_name.clone());
                    model.max_tokens = max_tokens;
                    model.max_output_tokens = max_output_tokens;
                    model.capabilities = settings::OpenAiModelCapabilities {
                        chat_completions,
                        tool_use,
                    };
                } else {
                    models.push(settings::AzureOpenAiAvailableModel {
                        name: deployment_name.clone(),
                        display_name: Some(deployment_name.clone()),
                        max_tokens,
                        max_output_tokens,
                        max_completion_tokens: None,
                        reasoning_effort: None,
                        capabilities: settings::OpenAiModelCapabilities {
                            chat_completions,
                            tool_use,
                        },
                    });
                }

                let agent = settings.agent.get_or_insert_default();
                let provider_setting = settings::LanguageModelProviderSetting("azure_openai".into());
                let default_model_selection = settings::LanguageModelSelection {
                    provider: provider_setting.clone(),
                    model: deployment_name.clone(),
                };

                match target_agent {
                    ConfigurationViewTargetAgent::ZedAgent => {
                        agent.default_model = Some(default_model_selection.clone());
                    }
                    ConfigurationViewTargetAgent::Other(profile_name) => {
                        if let Some(profiles) = agent.profiles.as_mut() {
                            if let Some(profile) = profiles.get_mut(profile_name.as_ref()) {
                                profile.default_model = Some(default_model_selection);
                            }
                        }
                    }
                }
            });

            if let Ok(new_thread_action) = cx.build_action("agent::NewTextThread", None) {
                window.dispatch_action(new_thread_action, cx);
            }
        }
        cx.notify();
    }

    fn reset_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.api_key_editor
            .update(cx, |editor, cx| editor.set_text("", window, cx));

        let state = self.state.clone();
        cx.spawn_in(window, async move |_, cx| {
            state
                .update(cx, |state, cx| state.set_api_key(None, cx))
                .await
        })
        .detach_and_log_err(cx);

        let fs = <dyn Fs>::global(cx);
        update_settings_file(fs, cx, |settings, _| {
            if let Some(azure) = settings
                .language_models
                .as_mut()
                .and_then(|models| models.azure_openai.as_mut())
            {
                azure.api_url = None;
                azure.api_version = None;
                azure.available_models = None;
            }
        });
        cx.notify();
    }
}

impl Render for ConfigurationView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.load_credentials_task.is_some() {
            div()
                .child(Label::new("Loading credentials..."))
                .into_any_element()
        } else {
            let env_var_name = self.state.read(cx).api_key_state.env_var_name();
            v_flex()
                .size_full()
                .gap_4()
                .on_action(cx.listener(Self::save_settings))
                .child(
                    v_flex()
                        .gap_2()
                        .child(Label::new(format!("To use {}, you need to configure your endpoint and deployment. Follow these steps:", match &self.target_agent {
                            ConfigurationViewTargetAgent::ZedAgent => "Zed's agent with Azure OpenAI".into(),
                            ConfigurationViewTargetAgent::Other(agent) => agent.clone(),
                        })))
                        .child(
                            List::new()
                                .child(
                                    ListBulletItem::new("")
                                        .child(Label::new("Create an account and resource at"))
                                        .child(ButtonLink::new("Azure AI Foundry", "https://ai.azure.com/"))
                                )
                                .child(
                                    ListBulletItem::new("Set up your deployment with a custom model name, and get the API key and endpoint URL.")
                                )
                        )
                )
                .child(
                    v_flex()
                        .gap_3()
                        .child(self.api_url_editor.clone())
                        .child(self.api_version_editor.clone())
                        .child(self.api_key_editor.clone())
                        .child(self.deployment_name_editor.clone())
                        .child(
                            h_flex()
                                .gap_3()
                                .child(self.max_tokens_editor.clone())
                                .child(self.max_output_tokens_editor.clone())
                        )
                )
                .child(
                    h_flex()
                        .gap_4()
                        .child(
                            ui::checkbox("chat_completions", if self.chat_completions { ui::ToggleState::Selected } else { ui::ToggleState::Unselected })
                                .label("Chat Model")
                                .on_click(cx.listener(|this, state, _, _| {
                                    this.chat_completions = *state == ui::ToggleState::Selected;
                                }))
                        )
                        .child(
                            ui::checkbox("tool_use", if self.tool_use { ui::ToggleState::Selected } else { ui::ToggleState::Unselected })
                                .label("Support Tools")
                                .on_click(cx.listener(|this, state, _, _| {
                                    this.tool_use = *state == ui::ToggleState::Selected;
                                }))
                        )
                )
                .child(
                    h_flex()
                        .justify_end()
                        .gap_2()
                        .child(
                            Button::new("reset-settings", "Reset Settings")
                                .style(ButtonStyle::Subtle)
                                .on_click(cx.listener(|this, _, window, cx| this.reset_settings(window, cx)))
                        )
                        .child(
                            Button::new("save-settings", "Save Settings")
                                .style(ButtonStyle::Filled)
                                .on_click(cx.listener(|this, _, window, cx| this.save_settings(&menu::Confirm, window, cx)))
                        )
                )
                .child(
                    Label::new(
                        format!("You can also set the {env_var_name} environment variable and restart zed-custom."),
                    )
                    .size(LabelSize::Small)
                    .color(Color::Muted)
                )
                .into_any_element()
        }
    }
}
