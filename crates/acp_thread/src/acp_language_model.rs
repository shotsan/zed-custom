use std::path::PathBuf;
use std::rc::Rc;
use std::cell::Cell;
use futures::{stream::BoxStream, StreamExt as _, FutureExt as _};
use gpui::{Entity, App, AsyncApp, Task};
use language_model::{
    LanguageModel, LanguageModelId, LanguageModelName, LanguageModelProviderId,
    LanguageModelProviderName, LanguageModelRequest, LanguageModelCompletionEvent,
    LanguageModelCompletionError, LanguageModelToolChoice,
};
use crate::{AgentConnection, AcpThreadEvent, AgentThreadEntry, AssistantMessageChunk};
use project::Project;
use anyhow::Result;

struct SendWrapper<F>(F);

// SAFETY: This wrapper is used to satisfy the `Send` bound on the future returned by
// `stream_completion`. The future captures non-Send GPUI thread-local types (`Rc`, `AsyncApp`),
// but is always polled on the GPUI foreground thread where these types are valid.
unsafe impl<F> Send for SendWrapper<F> {}
unsafe impl<F> Sync for SendWrapper<F> {}

impl<F: std::future::Future> std::future::Future for SendWrapper<F> {
    type Output = F::Output;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        unsafe {
            self.map_unchecked_mut(|x| &mut x.0).poll(cx)
        }
    }
}

/// A factory that spawns a fresh ACP connection (new subprocess) each time it is called.
/// Using a fresh process avoids the deadlock that arises from calling `prompt` on a
/// connection whose node process is already blocked in the event loop handling a tool call.
pub type ConnectionFactory = Rc<dyn Fn(&mut AsyncApp) -> Task<Result<Rc<dyn AgentConnection>>>>;

pub struct AcpLanguageModel {
    pub connection_factory: ConnectionFactory,
    pub project: Entity<Project>,
    pub cwd: PathBuf,
}

unsafe impl Send for AcpLanguageModel {}
unsafe impl Sync for AcpLanguageModel {}

impl LanguageModel for AcpLanguageModel {
    fn id(&self) -> LanguageModelId {
        LanguageModelId("acp-claude".into())
    }
    fn name(&self) -> LanguageModelName {
        LanguageModelName("Claude Code".into())
    }
    fn provider_id(&self) -> LanguageModelProviderId {
        LanguageModelProviderId("acp-claude-provider".into())
    }
    fn provider_name(&self) -> LanguageModelProviderName {
        LanguageModelProviderName("Claude Code Provider".into())
    }
    fn telemetry_id(&self) -> String {
        "acp-claude".to_string()
    }
    fn supports_images(&self) -> bool {
        false
    }
    fn supports_tools(&self) -> bool {
        false
    }
    fn supports_tool_choice(&self, _choice: LanguageModelToolChoice) -> bool {
        false
    }
    fn max_token_count(&self) -> u64 {
        200000
    }
    fn count_tokens(
        &self,
        request: LanguageModelRequest,
        _cx: &App,
    ) -> futures::future::BoxFuture<'static, Result<u64>> {
        let text = request.messages.iter().map(|m| m.string_contents()).collect::<Vec<_>>().join("\n");
        let token_count = (text.len() / 4) as u64;
        async move { Ok(token_count) }.boxed()
    }

    fn stream_completion(
        &self,
        request: LanguageModelRequest,
        cx: &AsyncApp,
    ) -> futures::future::BoxFuture<
        'static,
        Result<
            BoxStream<'static, Result<LanguageModelCompletionEvent, LanguageModelCompletionError>>,
            LanguageModelCompletionError,
        >,
    > {
        let connection_factory = self.connection_factory.clone();
        let project = self.project.clone();
        let cwd = self.cwd.clone();
        let mut cx = cx.clone();

        SendWrapper(async move {
            // 1. Spawn a fresh subprocess via the factory. This avoids the deadlock that
            //    occurs when `prompt` is called on a node process already blocked in its
            //    event-loop handling another session's tool call.
            let connection = connection_factory(&mut cx)
                .await
                .map_err(|e| {
                    eprintln!("AcpLanguageModel: failed to spawn fresh connection: {:#}", e);
                    LanguageModelCompletionError::Other(e)
                })?;

            let thread = cx.update(|cx| {
                connection.clone().new_thread(project.clone(), &cwd, cx)
            })
            .await
            .map_err(|e| {
                eprintln!("AcpLanguageModel: failed to create thread: {:#}", e);
                LanguageModelCompletionError::Other(e)
            })?;

            // 3. Build prompt text from request messages.
            let mut combined_prompt = String::new();
            for message in request.messages {
                combined_prompt.push_str(&message.string_contents());
                combined_prompt.push('\n');
            }

            // 4. Channel to forward streamed text chunks to the caller.
            let (tx, rx) = futures::channel::mpsc::unbounded();

            // 5. Subscribe to thread events and forward new text deltas.
            let last_streamed_len = Rc::new(Cell::new(0usize));
            let last_streamed_index = Rc::new(Cell::new(None::<usize>));
            let tx_clone = tx.clone();
            let last_streamed_len_clone = last_streamed_len;
            let last_streamed_index_clone = last_streamed_index;

            eprintln!("AcpLanguageModel: subscribing to thread events...");
            let subscription = cx.subscribe(&thread, move |_thread, event, cx| {
                eprintln!("AcpLanguageModel: received event: {:?}", event);
                match event {
                    AcpThreadEvent::NewEntry | AcpThreadEvent::EntryUpdated(_) => {
                        let entries = _thread.read(cx).entries();
                        if let Some((index, AgentThreadEntry::AssistantMessage(message))) = entries.iter().enumerate().rfind(|(_, e)| matches!(e, AgentThreadEntry::AssistantMessage(_))) {
                            let mut full_text = String::new();
                            for chunk in &message.chunks {
                                match chunk {
                                    AssistantMessageChunk::Message { block } => {
                                        full_text.push_str(block.to_markdown(cx));
                                    }
                                    AssistantMessageChunk::Thought { block } => {
                                        full_text.push_str(block.to_markdown(cx));
                                    }
                                }
                            }
                            
                            let mut prev_len = last_streamed_len_clone.get();
                            if last_streamed_index_clone.get() != Some(index) {
                                prev_len = 0;
                                last_streamed_index_clone.set(Some(index));
                            }
                            
                            if full_text.len() > prev_len {
                                let new_text = full_text[prev_len..].to_string();
                                last_streamed_len_clone.set(full_text.len());
                                eprintln!("AcpLanguageModel: streaming {} bytes", new_text.len());
                                if let Err(e) = tx_clone.unbounded_send(Ok(LanguageModelCompletionEvent::Text(new_text))) {
                                    eprintln!("AcpLanguageModel: receiver dropped! Error: {}", e);
                                    // Receiver dropped — normal on cancellation.
                                }
                            }
                        } else {
                            eprintln!("AcpLanguageModel: No AssistantMessage found in entries yet.");
                        }
                    }
                    _ => {}
                }
            });

            // 6. Send the prompt to the fresh session. This runs concurrently on its own
            //    foreground task so the caller can start consuming the stream immediately.
            let session_id = cx.update(|cx| thread.read(cx).session_id().clone());
            let acp_request = agent_client_protocol::PromptRequest::new(
                session_id,
                vec![agent_client_protocol::ContentBlock::Text(
                    agent_client_protocol::TextContent::new(combined_prompt)
                )]
            );

            cx.spawn({
                let thread = thread.clone();
                async move |cx| {
                    eprintln!("AcpLanguageModel: starting connection.prompt...");
                    let prompt_task = cx.update(|cx| connection.prompt(None, acp_request, cx));
                    eprintln!("AcpLanguageModel: prompt_task created, awaiting...");
                    if let Err(e) = prompt_task.await {
                        eprintln!("AcpLanguageModel: prompt failed: {:#}", e);
                        tx.unbounded_send(Err(LanguageModelCompletionError::Other(e))).ok();
                    } else {
                        eprintln!("AcpLanguageModel: prompt_task completed successfully!");
                    }

                    // Keep subscription and thread alive until the prompt completes.
                    eprintln!("AcpLanguageModel: dropping subscription & thread (end of prompt).");
                    drop(subscription);
                    drop(thread);

                    tx.unbounded_send(Ok(LanguageModelCompletionEvent::Stop(
                        language_model::StopReason::EndTurn
                    ))).ok();
                }
            }).detach();

            let thread_holder = thread;
            let boxed_stream: BoxStream<'static, Result<LanguageModelCompletionEvent, LanguageModelCompletionError>> =
                Box::pin(rx.map(move |item| {
                    let _keep_alive = &thread_holder;
                    item
                }));
            Ok(boxed_stream)
        }).boxed()
    }
}
