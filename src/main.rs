mod markdown;

use markdown::renderer::{self, BlockType as MdBlockType};
use slint::VecModel;
use std::rc::Rc;

slint::include_modules!();

fn to_slint_blocks(blocks: &[renderer::MarkdownBlock]) -> Vec<MarkdownBlock> {
    blocks
        .iter()
        .map(|b| MarkdownBlock {
            block_type: match b.block_type {
                MdBlockType::Paragraph => slint_generatedAppWindow::BlockType::Paragraph,
                MdBlockType::CodeBlock => slint_generatedAppWindow::BlockType::CodeBlock,
                MdBlockType::Heading1 => slint_generatedAppWindow::BlockType::Heading1,
                MdBlockType::Heading2 => slint_generatedAppWindow::BlockType::Heading2,
                MdBlockType::Heading3 => slint_generatedAppWindow::BlockType::Heading3,
                MdBlockType::ListItem => slint_generatedAppWindow::BlockType::ListItem,
                MdBlockType::HorizontalRule => slint_generatedAppWindow::BlockType::HorizontalRule,
            },
            text: b.text.clone().into(),
            lang: b.lang.clone().into(),
        })
        .collect()
}

fn main() {
    let ui = AppWindow::new().unwrap();

    // 测试用 Markdown 内容
    let sample = r#"
# 你好，Slint Chat

这是一段**普通段落**，包含 `内联代码` 示例。

## 功能列表

- 流式输出支持
- Markdown 分块渲染
- 代码块高亮

## 代码示例

```rust
fn main() {
    println!("Hello, Slint!");
}
```

---

### 小结

分块渲染方案覆盖了 AI 回复 90% 的场景。
"#;

    let blocks = renderer::parse(sample);
    let slint_blocks = to_slint_blocks(&blocks);
    let blocks_model: Rc<VecModel<MarkdownBlock>> = Rc::new(VecModel::from(slint_blocks));
    let msg = MessageItem {
        blocks: blocks_model.into(),
        content: sample.into(),
        is_user: false,
    };
    let messages_model = Rc::new(VecModel::from(vec![msg]));
    ui.set_messages(messages_model.into());

    ui.run().unwrap();
}

