pub mod slack_panel;
pub mod slack_store;

pub fn init(cx: &mut gpui::App) {
    slack_store::SlackStore::init(cx);
    slack_panel::init(cx);
}
