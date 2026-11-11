mod headless_platform;

use gpui::{App, Context, Global, BackgroundExecutor, ForegroundExecutor};
use anyhow::Result;
use headless_platform::HeadlessPlatform;
use std::sync::Arc;
use client::{Client, ZED_SERVER_URL};

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let background_executor = BackgroundExecutor::new();
    let foreground_executor = ForegroundExecutor::new();
    let platform = HeadlessPlatform::new(background_executor, foreground_executor);

    let app = App::new().with_platform(platform);
    
    app.run(move |cx| {
        tracing::info!("Starting Headless GPUI Daemon");
        
        // Initialize basic settings
        settings::init(cx);
        
        // Ensure server URL is correctly pointing to our custom backend if not overridden
        if ZED_SERVER_URL.is_none() {
            std::env::set_var("ZED_SERVER_URL", "https://api.zed-custom.dev");
        }

        // Initialize the client connection to zed-custom.dev
        let client = Client::production(cx);
        Client::set_global(client.clone(), cx);
        client::init(&client, cx);

        tracing::info!("Headless daemon initialized successfully. Waiting for SlackStore commands...");
        
        // The background daemon logic goes here.
        // It will listen on websockets or polling slack for agentic actions, 
        // interacting directly with the filesystem and LSP via GPUI context.
        cx.spawn(|cx| async move {
            // Keep the app alive
            std::future::pending::<()>().await;
        }).detach();
    });

    Ok(())
}
