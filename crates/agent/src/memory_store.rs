use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use gpui::{BackgroundExecutor, Task};
use indoc::indoc;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use sqlez::{
    bindable::{Bind, Column},
    connection::Connection,
    statement::Statement,
};
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryCategory {
    Architecture,
    Patterns,
    Issues,
    Procedures,
    Notes,
}

impl Bind for MemoryCategory {
    fn bind(&self, statement: &Statement, start_index: i32) -> Result<i32> {
        let value = match self {
            MemoryCategory::Architecture => "architecture",
            MemoryCategory::Patterns => "patterns",
            MemoryCategory::Issues => "issues",
            MemoryCategory::Procedures => "procedures",
            MemoryCategory::Notes => "notes",
        };
        value.bind(statement, start_index)
    }
}

impl Column for MemoryCategory {
    fn column(statement: &mut Statement, start_index: i32) -> Result<(Self, i32)> {
        let (value, next_index) = String::column(statement, start_index)?;
        let category = match value.as_str() {
            "architecture" => MemoryCategory::Architecture,
            "patterns" => MemoryCategory::Patterns,
            "issues" => MemoryCategory::Issues,
            "procedures" => MemoryCategory::Procedures,
            "notes" => MemoryCategory::Notes,
            _ => MemoryCategory::Notes,
        };
        Ok((category, next_index))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: Uuid,
    pub category: MemoryCategory,
    pub content: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
}

pub struct MemoryDatabase {
    executor: BackgroundExecutor,
    connection: Arc<Mutex<Connection>>,
}

impl MemoryDatabase {
    pub fn new(executor: BackgroundExecutor, connection: Connection) -> Result<Self> {
        log::info!("MemoryDatabase::new called - creating tables");
        
        match connection.exec(indoc! {"
            CREATE TABLE IF NOT EXISTS memories (
                id TEXT PRIMARY KEY,
                project_path TEXT NOT NULL,
                category TEXT NOT NULL,
                content TEXT NOT NULL,
                metadata TEXT,
                created_at TEXT NOT NULL,
                last_accessed TEXT NOT NULL
            )
        "}) {
            Ok(mut statement) => {
                if let Err(e) = statement() {
                    log::error!("Failed to execute CREATE TABLE memories: {}", e);
                    return Err(e);
                }
                log::info!("Successfully created 'memories' table");
            },
            Err(e) => {
                log::error!("Failed to prepare CREATE TABLE memories: {}", e);
                return Err(e);
            }
        }

        connection.exec(indoc! {"
            CREATE INDEX IF NOT EXISTS idx_memories_project 
                ON memories(project_path)
        "})?()?;
        log::info!("Created idx_memories_project");

        connection.exec(indoc! {"
            CREATE INDEX IF NOT EXISTS idx_memories_category 
                ON memories(category)
        "})?()?;
        log::info!("Created idx_memories_category");

        connection.exec(indoc! {"
            CREATE INDEX IF NOT EXISTS idx_memories_accessed 
                ON memories(last_accessed)
        "})?()?;
        log::info!("Created idx_memories_accessed");

        log::info!("MemoryDatabase initialization complete");

        Ok(Self {
            executor,
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    pub fn remember(
        &self,
        project_path: PathBuf,
        memory: Memory,
    ) -> Task<Result<()>> {
        let connection = self.connection.clone();
        let project_path_str = project_path.to_string_lossy().to_string();

        log::info!("=== MemoryDatabase::remember called ===");
        log::info!("Project path: {}", project_path_str);
        log::info!("Memory id: {}", memory.id);
        log::info!("Memory category: {:?}", memory.category);
        log::info!("Memory content: {}", memory.content);

        self.executor.spawn(async move {
            log::info!("Database write task started");
            
            let connection = connection.lock();
            log::info!("Database connection acquired");
            
            let mut insert = connection.exec_bound::<(
                String,
                String,
                MemoryCategory,
                String,
                String,
                String,
                String,
            )>(indoc! {"
                INSERT OR REPLACE INTO memories 
                (id, project_path, category, content, metadata, created_at, last_accessed) 
                VALUES (?, ?, ?, ?, ?, ?, ?)
            "})?;

            log::info!("Prepared INSERT statement");

            insert((
                memory.id.to_string(),
                project_path_str,
                memory.category,
                memory.content,
                serde_json::to_string(&memory.metadata)?,
                memory.created_at.to_rfc3339(),
                memory.last_accessed.to_rfc3339(),
            ))?;

            log::info!("=== Memory successfully written to database ===");

            Ok(())
        })
    }

    pub fn recall(
        &self,
        project_path: PathBuf,
        query: Option<String>,
        category: Option<MemoryCategory>,
        limit: usize,
    ) -> Task<Result<Vec<Memory>>> {
        let connection = self.connection.clone();
        let project_path_str = project_path.to_string_lossy().to_string();

        self.executor.spawn(async move {
            let connection = connection.lock();

            let (sql, params): (String, Vec<String>) = if let Some(ref cat) = category {
                (
                    indoc! {"
                        SELECT id, category, content, metadata, created_at, last_accessed
                        FROM memories
                        WHERE project_path = ? AND category = ?
                        ORDER BY last_accessed DESC
                        LIMIT ?
                    "}.to_string(),
                    vec![
                        project_path_str,
                        match cat {
                            MemoryCategory::Architecture => "architecture",
                            MemoryCategory::Patterns => "patterns",
                            MemoryCategory::Issues => "issues",
                            MemoryCategory::Procedures => "procedures",
                            MemoryCategory::Notes => "notes",
                        }.to_string(),
                        limit.to_string(),
                    ],
                )
            } else {
                (
                    indoc! {"
                        SELECT id, category, content, metadata, created_at, last_accessed
                        FROM memories
                        WHERE project_path = ?
                        ORDER BY last_accessed DESC
                        LIMIT ?
                    "}.to_string(),
                    vec![project_path_str, limit.to_string()],
                )
            };

            let mut select = connection.select_bound::<
                (String, String, String),
                (String, MemoryCategory, String, String, String, String),
            >(&sql)?;

            let rows = if category.is_some() {
                select((params[0].clone(), params[1].clone(), params[2].clone()))?
            } else {
                let mut select2 = connection.select_bound::<
                    (String, String),
                    (String, MemoryCategory, String, String, String, String),
                >(&sql)?;
                select2((params[0].clone(), params[1].clone()))?
            };

            let mut memories = Vec::new();
            for (id, category, content, metadata, created_at, last_accessed) in rows {
                let memory = Memory {
                    id: Uuid::parse_str(&id)?,
                    category,
                    content: if let Some(q) = &query {
                        if content.to_lowercase().contains(&q.to_lowercase()) {
                            content
                        } else {
                            continue;
                        }
                    } else {
                        content
                    },
                    metadata: serde_json::from_str(&metadata).unwrap_or(serde_json::Value::Null),
                    created_at: DateTime::parse_from_rfc3339(&created_at)?.with_timezone(&Utc),
                    last_accessed: DateTime::parse_from_rfc3339(&last_accessed)?
                        .with_timezone(&Utc),
                };
                memories.push(memory);
            }

            Ok(memories)
        })
    }
    pub fn recall_sync(
        &self,
        project_path: PathBuf,
        query: Option<String>,
        category: Option<MemoryCategory>,
        limit: usize,
    ) -> Vec<Memory> {
        let connection = self.connection.lock();
        let project_path_str = project_path.to_string_lossy().to_string();

        let (sql, params): (String, Vec<String>) = if let Some(ref cat) = category {
            (
                indoc! {"
                    SELECT id, category, content, metadata, created_at, last_accessed
                    FROM memories
                    WHERE project_path = ? AND category = ?
                    ORDER BY last_accessed DESC
                    LIMIT ?
                "}.to_string(),
                vec![
                    project_path_str,
                    match cat {
                        MemoryCategory::Architecture => "architecture",
                        MemoryCategory::Patterns => "patterns",
                        MemoryCategory::Issues => "issues",
                        MemoryCategory::Procedures => "procedures",
                        MemoryCategory::Notes => "notes",
                    }.to_string(),
                    limit.to_string(),
                ],
            )
        } else {
            (
                indoc! {"
                    SELECT id, category, content, metadata, created_at, last_accessed
                    FROM memories
                    WHERE project_path = ?
                    ORDER BY last_accessed DESC
                    LIMIT ?
                "}.to_string(),
                vec![project_path_str, limit.to_string()],
            )
        };

        let result: Result<Vec<Memory>> = (|| {
            let mut select = connection.select_bound::<
                (String, String, String),
                (String, MemoryCategory, String, String, String, String),
            >(&sql)?;

            let rows = if category.is_some() {
                select((params[0].clone(), params[1].clone(), params[2].clone()))?
            } else {
                let mut select2 = connection.select_bound::<
                    (String, String),
                    (String, MemoryCategory, String, String, String, String),
                >(&sql)?;
                select2((params[0].clone(), params[1].clone()))?
            };

            let mut memories = Vec::new();
            for (id, category, content, metadata, created_at, last_accessed) in rows {
                let memory = Memory {
                    id: Uuid::parse_str(&id)?,
                    category,
                    content: if let Some(q) = &query {
                        if content.to_lowercase().contains(&q.to_lowercase()) {
                            content
                        } else {
                            continue;
                        }
                    } else {
                        content
                    },
                    metadata: serde_json::from_str(&metadata).unwrap_or(serde_json::Value::Null),
                    created_at: DateTime::parse_from_rfc3339(&created_at)?.with_timezone(&Utc),
                    last_accessed: DateTime::parse_from_rfc3339(&last_accessed)?
                        .with_timezone(&Utc),
                };
                memories.push(memory);
            }
            Ok(memories)
        })();

        result.unwrap_or_default()
    }

    pub fn forget(&self, id: Uuid) -> Task<Result<()>> {
        let connection = self.connection.clone();

        self.executor.spawn(async move {
            let connection = connection.lock();
            let mut delete = connection.exec_bound::<String>(indoc! {"
                DELETE FROM memories WHERE id = ?
            "})?;

            delete(id.to_string())?;
            Ok(())
        })
    }

    pub fn update_access_time(&self, id: Uuid) -> Task<Result<()>> {
        let connection = self.connection.clone();
        let now = Utc::now();

        self.executor.spawn(async move {
            let connection = connection.lock();
            let mut update = connection.exec_bound::<(String, String)>(indoc! {"
                UPDATE memories SET last_accessed = ? WHERE id = ?
            "})?;

            update((now.to_rfc3339(), id.to_string()))?;
            Ok(())
        })
    }
}

#[derive(Clone)]
pub struct MemoryStore {
    db: Arc<MemoryDatabase>,
    project_path: PathBuf,
}

impl MemoryStore {
    pub fn new(db: Arc<MemoryDatabase>, project_path: PathBuf) -> Self {
        Self { db, project_path }
    }

    pub fn remember(&self, memory: Memory) -> Task<Result<()>> {
        self.db.remember(self.project_path.clone(), memory)
    }

    pub fn recall(
        &self,
        query: Option<String>,
        category: Option<MemoryCategory>,
        limit: usize,
    ) -> Task<Result<Vec<Memory>>> {
        self.db
            .recall(self.project_path.clone(), query, category, limit)
    }

    pub fn recall_sync(
        &self,
        query: &str,
        category: Option<MemoryCategory>,
        limit: usize,
    ) -> Vec<Memory> {
        let query = if query.is_empty() { None } else { Some(query.to_string()) };
        self.db
            .recall_sync(self.project_path.clone(), query, category, limit)
    }

    pub fn recall_all(&self) -> Task<Result<Vec<Memory>>> {
        self.db.recall(self.project_path.clone(), None, None, 1000)
    }


    pub fn forget(&self, id: Uuid) -> Task<Result<()>> {
        self.db.forget(id)
    }

    pub fn compress(&self, _threshold: Duration) -> Task<Result<()>> {
        // TODO: Implement compression logic using LLM
        // For now, just a placeholder
        Task::ready(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::TestAppContext;
    use sqlez::connection::Connection;

    #[gpui::test]
    async fn test_remember_and_recall(cx: &mut TestAppContext) {
        let connection = Connection::open_memory(Some("test_memory_db"));
        let db = Arc::new(MemoryDatabase::new(cx.executor(), connection).unwrap());
        let store = MemoryStore::new(db, PathBuf::from("/test/project"));

        let memory = Memory {
            id: Uuid::new_v4(),
            category: MemoryCategory::Architecture,
            content: "This project uses MVC architecture".to_string(),
            metadata: serde_json::Value::Null,
            created_at: Utc::now(),
            last_accessed: Utc::now(),
        };

        store.remember(memory.clone()).await.unwrap();

        let recalled = store.recall(None, Some(MemoryCategory::Architecture), 10)
            .await
            .unwrap();

        assert_eq!(recalled.len(), 1);
        assert_eq!(recalled[0].content, memory.content);
    }

    #[gpui::test]
    async fn test_query_filtering(cx: &mut TestAppContext) {
        let connection = Connection::open_memory(Some("test_memory_query_db"));
        let db = Arc::new(MemoryDatabase::new(cx.executor(), connection).unwrap());
        let store = MemoryStore::new(db, PathBuf::from("/test/project"));

        let memory1 = Memory {
            id: Uuid::new_v4(),
            category: MemoryCategory::Notes,
            content: "Use CMake for building".to_string(),
            metadata: serde_json::Value::Null,
            created_at: Utc::now(),
            last_accessed: Utc::now(),
        };

        let memory2 = Memory {
            id: Uuid::new_v4(),
            category: MemoryCategory::Notes,
            content: "Use Ninja generator".to_string(),
            metadata: serde_json::Value::Null,
            created_at: Utc::now(),
            last_accessed: Utc::now(),
        };

        store.remember(memory1).await.unwrap();
        store.remember(memory2).await.unwrap();

        let recalled = store.recall(Some("CMake".to_string()), None, 10)
            .await
            .unwrap();

        assert_eq!(recalled.len(), 1);
        assert!(recalled[0].content.contains("CMake"));
    }
}
