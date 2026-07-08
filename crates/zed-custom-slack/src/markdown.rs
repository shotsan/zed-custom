use pulldown_cmark::{Event, Parser, Tag, TagEnd};

pub fn format_for_slack(markdown_text: &str) -> String {
    let parser = Parser::new(markdown_text);
    let mut slack_text = String::new();

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Strong => slack_text.push('*'),
                Tag::Emphasis => slack_text.push('_'),
                Tag::Strikethrough => slack_text.push('~'),
                Tag::CodeBlock(_) => {
                    slack_text.push_str("```\n");
                }
                Tag::Heading { .. } => slack_text.push('*'),
                Tag::Link { dest_url, .. } => {
                    slack_text.push('<');
                    slack_text.push_str(&dest_url);
                    slack_text.push('|');
                }
                Tag::List(_) | Tag::Item => {
                    // Pulldown handles the bullets as text, so we just pass through
                }
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Strong => slack_text.push('*'),
                TagEnd::Emphasis => slack_text.push('_'),
                TagEnd::Strikethrough => slack_text.push('~'),
                TagEnd::CodeBlock => {
                    slack_text.push_str("\n```\n");
                }
                TagEnd::Heading { .. } => slack_text.push('*'),
                TagEnd::Link => slack_text.push('>'),
                _ => {}
            },
            Event::Text(text) => {
                slack_text.push_str(&text);
            }
            Event::Code(text) => {
                slack_text.push('`');
                slack_text.push_str(&text);
                slack_text.push('`');
            }
            Event::SoftBreak | Event::HardBreak => {
                slack_text.push('\n');
            }
            Event::Html(html) | Event::InlineHtml(html) => {
                slack_text.push_str(&html);
            }
            _ => {}
        }
    }

    slack_text
}
