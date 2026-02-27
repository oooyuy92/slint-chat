use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

#[derive(Clone, Debug)]
pub enum BlockType {
    Paragraph,
    CodeBlock,
    Heading1,
    Heading2,
    Heading3,
    ListItem,
    HorizontalRule,
}

#[derive(Clone, Debug)]
pub struct MarkdownBlock {
    pub block_type: BlockType,
    pub text: String,
    pub lang: String,
}

pub fn parse(input: &str) -> Vec<MarkdownBlock> {
    let mut blocks = Vec::new();
    let mut buf = String::new();
    let mut in_code = false;
    let mut code_lang = String::new();
    let mut in_list_item = false;

    let parser = Parser::new_ext(input, Options::all());

    for event in parser {
        match event {
            // ── 代码块 ──────────────────────────────────────────
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code = true;
                code_lang = match kind {
                    CodeBlockKind::Fenced(lang) => lang.to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
            }
            Event::End(TagEnd::CodeBlock) => {
                blocks.push(MarkdownBlock {
                    block_type: BlockType::CodeBlock,
                    text: buf.trim_end().to_string(),
                    lang: code_lang.clone(),
                });
                buf.clear();
                in_code = false;
            }

            // ── 标题 ────────────────────────────────────────────
            Event::Start(Tag::Heading { .. }) => buf.clear(),
            Event::End(TagEnd::Heading(level)) => {
                let bt = match level {
                    HeadingLevel::H1 => BlockType::Heading1,
                    HeadingLevel::H2 => BlockType::Heading2,
                    _ => BlockType::Heading3,
                };
                blocks.push(MarkdownBlock {
                    block_type: bt,
                    text: buf.trim().to_string(),
                    lang: String::new(),
                });
                buf.clear();
            }

            // ── 段落 ────────────────────────────────────────────
            Event::End(TagEnd::Paragraph) => {
                let t = buf.trim().to_string();
                if !t.is_empty() {
                    blocks.push(MarkdownBlock {
                        block_type: BlockType::Paragraph,
                        text: t,
                        lang: String::new(),
                    });
                }
                buf.clear();
            }

            // ── 列表项 ──────────────────────────────────────────
            Event::Start(Tag::Item) => {
                in_list_item = true;
                buf.clear();
            }
            Event::End(TagEnd::Item) => {
                let t = buf.trim().to_string();
                if !t.is_empty() {
                    blocks.push(MarkdownBlock {
                        block_type: BlockType::ListItem,
                        text: t,
                        lang: String::new(),
                    });
                }
                buf.clear();
                in_list_item = false;
            }

            // ── 分割线 ──────────────────────────────────────────
            Event::Rule => {
                blocks.push(MarkdownBlock {
                    block_type: BlockType::HorizontalRule,
                    text: String::new(),
                    lang: String::new(),
                });
            }

            // ── 文本内容 ─────────────────────────────────────────
            Event::Text(text) => buf.push_str(&text),
            Event::Code(code) => {
                // 内联代码：用反引号包裹降级显示（Slint Text 不支持内联样式）
                buf.push('`');
                buf.push_str(&code);
                buf.push('`');
            }
            Event::SoftBreak => buf.push(' '),
            Event::HardBreak => buf.push('\n'),

            _ => {}
        }
    }

    blocks
}
