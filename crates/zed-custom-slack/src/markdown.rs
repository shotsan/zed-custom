use pulldown_cmark::{Event, Parser, Tag, TagEnd};

pub fn format_for_slack(markdown_text: &str) -> String {
    let mut options = pulldown_cmark::Options::empty();
    options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(markdown_text, options);
    let mut slack_text = String::new();
    let mut list_stack: Vec<Option<u64>> = Vec::new();
    let mut blockquote_depth = 0;

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Strong => slack_text.push('*'),
                Tag::Emphasis => slack_text.push('_'),
                Tag::Strikethrough => slack_text.push('~'),
                Tag::CodeBlock(_) => {
                    if !slack_text.is_empty() && !slack_text.ends_with('\n') {
                        slack_text.push('\n');
                    }
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
                    if !slack_text.is_empty() && !slack_text.ends_with('\n') {
                        slack_text.push('\n');
                    }
                    let stack_len = list_stack.len();
                    if let Some(list_type) = list_stack.last_mut() {
                        let indent = "  ".repeat(stack_len.saturating_sub(1));
                        slack_text.push_str(&indent);
                        if let Some(num) = list_type {
                            slack_text.push_str(&format!("{}. ", num));
                            *num += 1;
                        } else {
                            slack_text.push_str("• ");
                        }
                    }
                }
                Tag::BlockQuote(_) => {
                    blockquote_depth += 1;
                    if !slack_text.is_empty() && !slack_text.ends_with('\n') {
                        slack_text.push('\n');
                    }
                    slack_text.push_str("> ");
                }
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Strong => slack_text.push('*'),
                TagEnd::Emphasis => slack_text.push('_'),
                TagEnd::Strikethrough => slack_text.push('~'),
                TagEnd::CodeBlock => {
                    if !slack_text.ends_with('\n') {
                        slack_text.push('\n');
                    }
                    slack_text.push_str("```\n\n");
                }
                TagEnd::Heading { .. } => {
                    slack_text.push_str("*\n\n");
                }
                TagEnd::Link => slack_text.push('>'),
                TagEnd::List(_) => {
                    list_stack.pop();
                    if list_stack.is_empty() {
                        slack_text.push('\n');
                    }
                }
                TagEnd::Item => {
                    if !slack_text.ends_with('\n') {
                        slack_text.push('\n');
                    }
                }
                TagEnd::Paragraph => {
                    slack_text.push_str("\n\n");
                }
                TagEnd::BlockQuote(_) => {
                    blockquote_depth -= 1;
                    slack_text.push_str("\n\n");
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
                for _ in 0..blockquote_depth {
                    slack_text.push_str("> ");
                }
                if !list_stack.is_empty() {
                    slack_text.push_str(&"  ".repeat(list_stack.len()));
                }
            }
            Event::Html(html) | Event::InlineHtml(html) => {
                slack_text.push_str(&html);
            }
            _ => {}
        }
    }

    // Normalize multiple newlines down to a max of two to prevent massive gaps.
    let mut normalized = String::with_capacity(slack_text.len());
    let mut consecutive_newlines = 0;
    for c in slack_text.chars() {
        if c == '\n' {
            consecutive_newlines += 1;
            if consecutive_newlines <= 2 {
                normalized.push(c);
            }
        } else {
            consecutive_newlines = 0;
            normalized.push(c);
        }
    }

    normalized.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paragraphs() {
        let md = "Paragraph 1\n\nParagraph 2\n\nParagraph 3";
        let expected = "Paragraph 1\n\nParagraph 2\n\nParagraph 3";
        assert_eq!(format_for_slack(md), expected);
    }

    #[test]
    fn test_unordered_lists() {
        let md = "- Item A\n- Item B\n- Item C";
        let expected = "• Item A\n• Item B\n• Item C";
        assert_eq!(format_for_slack(md), expected);
    }

    #[test]
    fn test_ordered_lists() {
        let md = "1. First\n2. Second\n3. Third";
        let expected = "1. First\n2. Second\n3. Third";
        assert_eq!(format_for_slack(md), expected);
    }

    #[test]
    fn test_nested_lists() {
        let md = "- Top 1\n  - Nested 1\n  - Nested 2\n- Top 2";
        let expected = "• Top 1\n  • Nested 1\n  • Nested 2\n• Top 2";
        assert_eq!(format_for_slack(md), expected);
    }

    #[test]
    fn test_inline_formatting() {
        let md = "**Bold** and *Italic* and ~~Strikethrough~~";
        let expected = "*Bold* and _Italic_ and ~Strikethrough~";
        assert_eq!(format_for_slack(md), expected);
    }

    #[test]
    fn test_headings() {
        let md = "# Title\n\n## Subtitle";
        let expected = "*Title*\n\n*Subtitle*";
        assert_eq!(format_for_slack(md), expected);
    }

    #[test]
    fn test_links() {
        let md = "Check out [Zed](https://zed.dev)";
        let expected = "Check out <https://zed.dev|Zed>";
        assert_eq!(format_for_slack(md), expected);
    }

    #[test]
    fn test_code_blocks() {
        let md = "```rust\nfn main() {}\n```";
        let expected = "```\nfn main() {}\n```";
        assert_eq!(format_for_slack(md), expected);
    }

    #[test]
    fn test_inline_code() {
        let md = "Use `cargo run`";
        let expected = "Use `cargo run`";
        assert_eq!(format_for_slack(md), expected);
    }

    #[test]
    fn test_blockquotes() {
        let md = "> Line 1\n> Line 2";
        let expected = "> Line 1\n> Line 2";
        assert_eq!(format_for_slack(md), expected);
    }

    #[test]
    fn test_list_followed_by_paragraph() {
        let md = "- Bullet 1\n- Bullet 2\n\nAnd now a paragraph.";
        let expected = "• Bullet 1\n• Bullet 2\n\nAnd now a paragraph.";
        assert_eq!(format_for_slack(md), expected);
    }

    #[test]
    fn test_complex_nesting() {
        let md = "- **Bold item** with `code`\n- *Italic item*";
        let expected = "• *Bold item* with `code`\n• _Italic item_";
        assert_eq!(format_for_slack(md), expected);
    }
    
    #[test]
    fn test_html_pass_through() {
        let md = "This is <strong>HTML</strong>";
        let expected = "This is <strong>HTML</strong>";
        assert_eq!(format_for_slack(md), expected);
    }

    #[test]
    fn test_multiline_list_item() {
        let md = "- Line 1\n  Line 2\n- Next Item";
        let expected = "• Line 1\n  Line 2\n• Next Item";
        assert_eq!(format_for_slack(md), expected);
    }

    #[test]
    fn test_excessive_newlines_are_trimmed() {
        let md = "Paragraph 1\n\n\n\n\n\nParagraph 2\n\n\n";
        let expected = "Paragraph 1\n\nParagraph 2";
        assert_eq!(format_for_slack(md), expected);
    }
}
