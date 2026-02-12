    fn show_teach_rule_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let connection = match self.as_native_connection(cx) {
            Some(conn) => conn,
            None => {
                log::error!("Cannot teach rule: not a native connection");
                return;
            }
        };

        window.prompt(
            gpui::PromptLevel::Info,
            "Teach me a new project rule",
            Some("Enter a fact, preference, or architectural rule about this project:"),
            &["Save", "Cancel"],
            cx,
            move |answer, window, cx| {
                if answer == Some(0) {
                    // User clicked "Save"
                    if let Some(input) = window.take_prompt_text() {
                        if !input.trim().is_empty() {
                            log::info!("=== Teach Rule Dialog: Saving rule ===");
                            log::info!("Rule content: {}", input);
                            
                            connection.update(cx, |conn, cx| {
                                if let Some(memory_store) = conn.memory_store() {
                                    use agent::memory_store::{Memory, MemoryCategory};
                                    use chrono::Utc;
                                    use uuid::Uuid;
                                    
                                    let memory = Memory {
                                        id: Uuid::new_v4(),
                                        category: MemoryCategory::ProjectInfo,
                                        content: input.trim().to_string(),
                                        metadata: serde_json::Value::Null,
                                        created_at: Utc::now(),
                                        last_accessed: Utc::now(),
                                    };
                                    
                                    log::info!("Calling memory_store.remember() with id: {}", memory.id);
                                    memory_store.remember(memory).detach_and_log_err(cx);
                                    log::info!("Rule saved successfully");
                                } else {
                                    log::error!("Memory store not available");
                                }
                            });
                        }
                    }
                }
            },
        );
    }
