use pulldown_cmark::{Event, Parser, Tag, TagEnd};

pub fn format_for_slack(markdown_text: &str) -> String {
    let parser = Parser::new(markdown_text);
    let mut slack_text = String::new();
    let mut list_stack: Vec<Option<u64>> = Vec::new();

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
                Tag::List(first_item) => {
                    list_stack.push(first_item);
                }
                Tag::Item => {
                    if let Some(list_type) = list_stack.last_mut() {
                        let indent = "  ".repeat(list_stack.len().saturating_sub(1));
                        slack_text.push_str(&indent);
                        if let Some(num) = list_type {
                            slack_text.push_str(&format!("{}. ", num));
                            *num += 1;
                        } else {
                            slack_text.push_str("• ");
                        }
                    }
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
                TagEnd::List(_) => {
                    list_stack.pop();
                }
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

fn main() {
    let md = "- item 1\n- item 2\n  - nested 1\n  - nested 2\n\n1. First\n2. Second";
    println!("{}", format_for_slack(md));
}
