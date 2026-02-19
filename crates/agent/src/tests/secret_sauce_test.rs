use crate::{
    Thread, AnyAgentTool, ContextServerRegistry, Templates, MemoryStore, SemanticIndex,
    MemoryDatabase, LspGetDefinitionTool, LspFindReferencesTool, LspGetImplementationsTool,
    SaveReflectionTool, ThreadsDatabase, ThreadEnvironment, TerminalHandle,
};
use gpui::{AsyncApp, Task, TestAppContext, AppContext};
use anyhow::Result;
use std::rc::Rc;
use project::{Project, context_server_store::ContextServerStore, worktree_store::WorktreeStore, context_server_store::registry::ContextServerDescriptorRegistry};
use std::sync::Arc;
use fs::FakeFs;
use prompt_store::ProjectContext;
use settings::SettingsStore;
use std::path::PathBuf;
use sqlez::connection::Connection;

#[gpui::test]
async fn test_secret_sauce_tool_registration(cx: &mut TestAppContext) {
    let fs = FakeFs::new(cx.executor());
    let background = cx.background_executor.clone();
    
    cx.update(|cx| {
        let settings = SettingsStore::test(cx);
        cx.set_global(settings);
    });

    let project = Project::test(fs.clone(), [], cx).await;
    let project_context = cx.update(|cx| cx.new(|_| ProjectContext::new(vec![], vec![], None)));
    
    let worktree_store = cx.update(|cx| cx.new(|_| WorktreeStore::local(false, fs.clone(), Default::default())));
    let registry = cx.update(|cx| ContextServerDescriptorRegistry::default_global(cx));
    let context_server_store = cx.update(|cx| {
        cx.new(|cx| ContextServerStore::test(registry, worktree_store, None, cx))
    });
    let context_server_registry = cx.update(|cx| cx.new(|cx| ContextServerRegistry::new(context_server_store, cx)));
    
    let templates = Templates::new();
    
    // MemoryStore setup
    let memory_db_conn = Connection::open_memory(Some("test_memory_db"));
    let memory_db = Arc::new(MemoryDatabase::new(background.clone(), memory_db_conn).unwrap());
    let memory_store = Arc::new(MemoryStore::new(memory_db, PathBuf::from("/tmp/zed-memory")));
    
    let semantic_index = Arc::new(parking_lot::RwLock::new(SemanticIndex::new()));
    
    let thread = cx.update(|cx| {
        cx.new(|cx| {
            let mut thread = Thread::new(
                project.clone(),
                project_context,
                context_server_registry,
                templates,
                memory_store,
                semantic_index,
                None,
                cx
            );
            
            struct TestThreadEnvironment;
            impl ThreadEnvironment for TestThreadEnvironment {
                fn create_terminal(
                    &self,
                    _command: String,
                    _cwd: Option<std::path::PathBuf>,
                    _output_byte_limit: Option<u64>,
                    _cx: &mut AsyncApp,
                ) -> Task<Result<Rc<dyn TerminalHandle>>> {
                    Task::ready(Err(anyhow::anyhow!("Not implemented")))
                }
            }

            thread.add_default_tools(Rc::new(TestThreadEnvironment), cx);
            
            thread
        })
    });
    
    cx.update(|cx| {
        let tools = thread.read(cx).tools();
        let tool_names: Vec<String> = tools.values().map(|t: &Arc<dyn AnyAgentTool>| t.name().to_string()).collect();
        
        assert!(tool_names.contains(&"lsp_get_definition".to_string()));
        assert!(tool_names.contains(&"lsp_find_references".to_string()));
        assert!(tool_names.contains(&"lsp_get_implementations".to_string()));
        assert!(tool_names.contains(&"lsp_save_reflection".to_string()));
    });
}

#[gpui::test]
async fn test_hippocampus_schema(cx: &mut TestAppContext) {
    let executor = cx.background_executor.clone();
    let db = crate::db::ThreadsDatabase::new(executor).unwrap();
    
    // Check if the new tables exist
    let tables: Vec<String> = db.connection_for_test().lock().select::<String>("SELECT name FROM sqlite_master WHERE type='table'").unwrap()().unwrap();
    
    assert!(tables.contains(&"sensory_logs".to_string()));
    assert!(tables.contains(&"reflections".to_string()));
}
