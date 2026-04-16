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
        let settings = Self::settings(cx);
        let api_url = settings
            .api_url
            .clone()
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string());
        SharedString::from(api_url)
    }

    fn api_version(cx: &App) -> Option<String> {
        Self::settings(cx).api_version.clone()
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
        let settings = Self::settings(cx);
        if let Some(deployment_name) = &settings.deployment_name {
            Some(self.create_language_model(AvailableModel {
                name: deployment_name.clone(),
                display_name: Some(deployment_name.clone()),
                max_tokens: 128000,
                max_output_tokens: Some(16384),
                max_completion_tokens: None,
                reasoning_effort: None,
                capabilities: ModelCapabilities {
                    chat_completions: true,
                    tool_use: true,
                },
            }))
        } else {
            settings
                .available_models
                .unwrap_or_default()
                .first()
                .map(|model| self.create_language_model(model.clone()))
        }
    }

    fn default_fast_model(&self, _cx: &App) -> Option<Arc<dyn LanguageModel>> {
        None
    }

    fn provided_models(&self, cx: &App) -> Vec<Arc<dyn LanguageModel>> {
        let settings = Self::settings(cx);
        let mut models = Vec::new();
        
        if let Some(deployment_name) = &settings.deployment_name {
            models.push(self.create_language_model(AvailableModel {
                name: deployment_name.clone(),
                display_name: Some(deployment_name.clone()),
                max_tokens: 128000,
                max_output_tokens: Some(16384),
                max_completion_tokens: None,
                reasoning_effort: None,
                capabilities: ModelCapabilities {
                    chat_completions: true,
                    tool_use: true,
                },
            }));
        }

        let available_models = settings.available_models.unwrap_or_default();
        for model in available_models {
            if !models.iter().any(|m| m.id().0 == model.name) {
                models.push(self.create_language_model(model.clone()));
            }
        }
        
        models
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
            let settings = AzureOpenAiLanguageModelProvider::settings(_cx);
            let base_url = AzureOpenAiLanguageModelProvider::api_url(_cx);
            let api_key = settings.api_key.clone().or_else(|| state.api_key_state.key(&base_url).map(|key| key.to_string()));
            let api_version = AzureOpenAiLanguageModelProvider::api_version(_cx);

            let mut full_url = base_url.to_string();
            // If the URL already contains key segments, treat it as a direct link and stop appending.
            let is_direct_link = full_url.contains("/v1/") || full_url.contains("/openai/") || full_url.contains("/chat/completions") || full_url.contains("/responses");
            
            if !is_direct_link {
                let deployment_name = &self.model.name;
                let trimmed = full_url.trim_end_matches('/');
                full_url = format!("{trimmed}/openai/deployments/{deployment_name}/chat/completions");
            }
            
            if let Some(api_version) = api_version.as_deref() {
                if !full_url.contains("api-version=") {
                    full_url.push_str(if full_url.contains('?') { "&" } else { "?" });
                    full_url.push_str("api-version=");
                    full_url.push_str(api_version);
                }
            }
            
            (api_key, SharedString::from(full_url))
        });

        let provider = PROVIDER_NAME.clone();
        if request.messages.is_empty() && request.input.as_ref().map_or(true, |i| i.is_empty()) {
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
            let settings = AzureOpenAiLanguageModelProvider::settings(_cx);
            let base_url = AzureOpenAiLanguageModelProvider::api_url(_cx);
            let api_key = settings.api_key.clone().or_else(|| state.api_key_state.key(&base_url).map(|key| key.to_string()));
            let api_version = AzureOpenAiLanguageModelProvider::api_version(_cx);

            let mut full_url = base_url.to_string();
            // If the URL already contains key segments, treat it as a direct link and stop appending.
            let is_direct_link = full_url.contains("/v1/") || full_url.contains("/openai/") || full_url.contains("/chat/completions") || full_url.contains("/responses");
            
            if !is_direct_link {
                let trimmed = full_url.trim_end_matches('/');
                // Default to the modern Responses API path
                full_url = format!("{trimmed}/openai/v1/responses");
            }
            
            if let Some(api_version) = api_version.as_deref() {
                if !full_url.contains("api-version=") {
                    full_url.push_str(if full_url.contains('?') { "&" } else { "?" });
                    full_url.push_str("api-version=");
                    full_url.push_str(api_version);
                }
            }

            (api_key, SharedString::from(full_url))
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
        let deployment_name = self.model.name.clone();
        cx.background_spawn(async move {
            let messages = crate::provider::open_ai::collect_tiktoken_messages(request);
            let model = if deployment_name.contains("gpt-4o") {
                "gpt-4o"
            } else if deployment_name.contains("gpt-4") {
                "gpt-4"
            } else if deployment_name.contains("o1") {
                "o1-preview"
            } else {
                "gpt-4o"
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
        let use_responses_api = self.state.read_with(cx, |_, _cx| {
            let api_url = AzureOpenAiLanguageModelProvider::api_url(_cx);
            // Default to Responses API for Azure unless chat/completions is explicitly requested
            let explicitly_chat = api_url.contains("/chat/completions");
            (api_url.contains("/responses") || !explicitly_chat) || !self.model.capabilities.chat_completions
        });

        if !use_responses_api {
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
    deployment_name_editor: Entity<InputField>,
    state: Entity<State>,
    load_credentials_task: Option<Task<()>>,
    target_agent: ConfigurationViewTargetAgent,
}

impl ConfigurationView {
    fn new(state: Entity<State>, target_agent: ConfigurationViewTargetAgent, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let settings = AzureOpenAiLanguageModelProvider::settings(cx);
        
        let api_key_editor = cx.new(|cx| {
            let input = InputField::new(
                window,
                cx,
                "000000000000000000000000000000000000000000000000000",
            ).label("API Key")
            .password(true, window, cx);
            if let Some(api_key) = &settings.api_key {
                input.set_text(api_key, window, cx);
            }
            input
        });

        let api_url_editor = cx.new(|cx| {
            let input = InputField::new(window, cx, "https://<resource>.openai.azure.com/").label("API URL / Endpoint");
            if let Some(api_url) = &settings.api_url {
                input.set_text(api_url, window, cx);
            }
            input
        });

        let deployment_name_editor = cx.new(|cx| {
            let editor = InputField::new(window, cx, "gpt-4o").label("Deployment / Model Name");
            if let Some(deployment_name) = &settings.deployment_name {
                editor.set_text(deployment_name, window, cx);
            } else if let Some(model) = settings.available_models.as_ref().and_then(|m| m.first()) {
                editor.set_text(&model.name, window, cx);
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
            deployment_name_editor,
            state,
            load_credentials_task,
            target_agent,
        }
    }

    fn save_settings(&mut self, _: &menu::Confirm, window: &mut Window, cx: &mut Context<Self>) {
        let api_key = self.api_key_editor.read(cx).text(cx).trim().to_string();
        let api_url = self.api_url_editor.read(cx).text(cx).trim().to_string();
        let deployment_name = self.deployment_name_editor.read(cx).text(cx).trim().to_string();

        if !api_url.is_empty() && !deployment_name.is_empty() {
            let target_agent = self.target_agent.clone();
            let fs = <dyn Fs>::global(cx);
            update_settings_file(fs, cx, move |settings, _| {
                let language_models = settings.language_models.get_or_insert_default();
                let azure = language_models.azure_openai.get_or_insert_default();
                
                azure.api_url = Some(api_url);
                azure.api_key = if api_key.is_empty() { None } else { Some(api_key) };
                let deployment_name = deployment_name.clone();
                azure.deployment_name = Some(deployment_name.clone());
                
                let available_models = azure.available_models.get_or_insert_default();
                if !available_models.iter().any(|m| m.name == deployment_name) {
                    available_models.push(settings::AzureOpenAiAvailableModel {
                        name: deployment_name.clone(),
                        display_name: Some(deployment_name.clone()),
                        max_tokens: 128000,
                        max_output_tokens: Some(16384),
                        max_completion_tokens: None,
                        reasoning_effort: None,
                        capabilities: settings::OpenAiModelCapabilities {
                            chat_completions: true,
                            tool_use: true,
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
        self.api_key_editor.update(cx, |editor, cx| editor.set_text("", window, cx));
        self.api_url_editor.update(cx, |editor, cx| editor.set_text("", window, cx));
        self.deployment_name_editor.update(cx, |editor, cx| editor.set_text("", window, cx));

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
                azure.api_key = None;
                azure.deployment_name = None;
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
                        .child(self.api_key_editor.clone())
                        .child(self.deployment_name_editor.clone())
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
