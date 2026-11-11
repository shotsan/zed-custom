import re

with open("crates/collab/tests/integration/azure_ad_tests.rs", "r") as f:
    content = f.read()

content = content.replace(
    "let mut modified_app_state = (*app_state).clone();\n    modified_app_state.config.azure_ad_client_id = None;\n    let modified_app_state = Arc::new(modified_app_state);",
    """let mut config = app_state.config.clone();
    config.azure_ad_client_id = None;
    let modified_app_state = Arc::new(collab::AppState {
        config,
        db: app_state.db.clone(),
        livekit_client: app_state.livekit_client.clone(),
        blob_store_client: app_state.blob_store_client.clone(),
        kinesis_client: app_state.kinesis_client.clone(),
        executor: app_state.executor.clone(),
    });"""
)

with open("crates/collab/tests/integration/azure_ad_tests.rs", "w") as f:
    f.write(content)
