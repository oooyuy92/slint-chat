# ChatGPT-Style UI Redesign Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Redesign all Slint UI files to match ChatGPT web interface (light theme), keeping parallel mode and quick phrases integrated.

**Architecture:** Pure UI change — no Rust logic touched. All 6 .slint files get rewritten with ChatGPT light-theme colors, layout, and interaction patterns.

**Tech Stack:** Slint 1.x, existing component structure preserved

---

## Color Tokens (reference throughout)

| Token | Value | Usage |
|---|---|---|
| bg-main | #ffffff | Chat area background |
| bg-sidebar | #f9f9f9 | Sidebar background |
| bg-hover | #efefef | Hover state |
| bg-active | #e8e8e8 | Active/selected |
| bg-user-bubble | #f4f4f4 | User message bubble |
| bg-code | #f4f4f4 | Code block background |
| bg-input | #ffffff | Input box |
| border | #e5e5e5 | Borders |
| text-primary | #0d0d0d | Main text |
| text-secondary | #6e6e80 | Placeholder, secondary |
| text-muted | #9ca3af | Muted labels |
| ai-green | #10a37f | AI avatar accent |
| send-btn | #0d0d0d | Send button bg |

---

### Task 1: app.slint — light theme root

**Files:**
- Modify: `ui/app.slint`

**Step 1: Rewrite app.slint**

```slint
import { MarkdownBlock, BlockType } from "components/markdown_view.slint";
import { MessageItem } from "components/message_bubble.slint";
import { TopicItem, Sidebar } from "sidebar.slint";
import { ChatView } from "chat_view.slint";
import { InputBar } from "input_bar.slint";

import "./fonts/NotoSansSC-subset.ttf";

export component AppWindow inherits Window {
    title: "Slint Chat";
    width: 1100px;
    height: 720px;
    background: #ffffff;
    default-font-family: "Noto Sans CJK SC";

    in-out property <[TopicItem]> topics <=> sidebar.topics;
    in-out property <[MessageItem]> messages <=> chat.messages;
    in property <bool> sending <=> input.sending;

    callback new-topic <=> sidebar.new-topic;
    callback topic-selected <=> sidebar.topic-selected;
    callback send <=> input.send;

    HorizontalLayout {
        sidebar := Sidebar {}

        VerticalLayout {
            chat := ChatView {}
            input := InputBar {}
        }
    }
}
```

**Step 2: Build**
```bash
cargo build 2>&1 | grep -E "^error"
```
Expected: no errors.

**Step 3: Commit**
```bash
git add ui/app.slint
git commit -m "style: light theme root window"
```

---

### Task 2: sidebar.slint — ChatGPT sidebar

**Files:**
- Modify: `ui/sidebar.slint`

**Step 1: Rewrite sidebar.slint**

```slint
export struct TopicItem {
    id: string,
    title: string,
}

export component Sidebar inherits Rectangle {
    in property <[TopicItem]> topics;
    in-out property <int> active-index: -1;
    callback topic-selected(int);
    callback new-topic();

    width: 260px;
    background: #f9f9f9;

    VerticalLayout {
        padding: 8px;
        spacing: 2px;

        // Header: logo area
        Rectangle {
            height: 48px;
            HorizontalLayout {
                padding-left: 12px;
                alignment: center-y;
                Text {
                    text: "Slint Chat";
                    color: #0d0d0d;
                    font-size: 15px;
                    font-weight: 600;
                }
            }
        }

        // New chat button
        new-btn-ta := TouchArea {
            height: 40px;
            clicked => { root.new-topic(); }

            Rectangle {
                background: new-btn-ta.has-hover ? #efefef : transparent;
                border-radius: 8px;

                HorizontalLayout {
                    padding-left: 12px;
                    spacing: 10px;
                    alignment: center-y;

                    Text {
                        text: "✏";
                        font-size: 15px;
                        color: #0d0d0d;
                        vertical-alignment: center;
                    }
                    Text {
                        text: "新建对话";
                        color: #0d0d0d;
                        font-size: 14px;
                        vertical-alignment: center;
                    }
                }
            }
        }

        Rectangle { height: 4px; }

        // Section label
        Text {
            text: "最近对话";
            color: #9ca3af;
            font-size: 11px;
            padding-left: 12px;
            padding-bottom: 2px;
        }

        // Topic list
        for topic[i] in root.topics : item-ta := TouchArea {
            height: 36px;
            clicked => {
                root.active-index = i;
                root.topic-selected(i);
            }

            Rectangle {
                background: root.active-index == i ? #e8e8e8
                          : item-ta.has-hover ? #efefef
                          : transparent;
                border-radius: 8px;

                Text {
                    text: topic.title;
                    color: #0d0d0d;
                    font-size: 14px;
                    overflow: elide;
                    padding-left: 12px;
                    padding-right: 8px;
                    vertical-alignment: center;
                }
            }
        }
    }
}
```

**Step 2: Build**
```bash
cargo build 2>&1 | grep -E "^error"
```

**Step 3: Commit**
```bash
git add ui/sidebar.slint
git commit -m "style: ChatGPT-style sidebar with hover states"
```

---

### Task 3: components/markdown_view.slint — light theme

**Files:**
- Modify: `ui/components/markdown_view.slint`

**Step 1: Update colors to light theme**

Change all dark colors:
- Paragraph text: `#d4d4d4` → `#374151`
- H1 text: `#ffffff` → `#111827`
- H2 text: `#eeeeee` → `#1f2937`
- H3 text: `#dddddd` → `#374151`
- List item bullet + text: `#888` / `#d4d4d4` → `#6b7280` / `#374151`
- Code block bg: `#1e1e2e` → `#f4f4f4`
- Code block border: `#333` → `#e5e5e5`
- Code lang label: `#888` → `#9ca3af`
- Code text: `#cdd6f4` → `#1f2937`
- HR: `#444` → `#e5e5e5`

Full rewrite:

```slint
export enum BlockType {
    Paragraph,
    CodeBlock,
    Heading1,
    Heading2,
    Heading3,
    ListItem,
    HorizontalRule,
}

export struct MarkdownBlock {
    block-type: BlockType,
    text: string,
    lang: string,
}

export component MarkdownView {
    in property <[MarkdownBlock]> blocks;

    VerticalLayout {
        spacing: 8px;
        alignment: start;

        for block[i] in root.blocks : Rectangle {
            if block.block-type == BlockType.Paragraph : Text {
                text: block.text;
                wrap: word-wrap;
                color: #374151;
                font-size: 15px;
                line-height: 1.6;
            }

            if block.block-type == BlockType.Heading1 : Text {
                text: block.text;
                wrap: word-wrap;
                color: #111827;
                font-size: 22px;
                font-weight: 700;
            }

            if block.block-type == BlockType.Heading2 : Text {
                text: block.text;
                wrap: word-wrap;
                color: #1f2937;
                font-size: 18px;
                font-weight: 700;
            }

            if block.block-type == BlockType.Heading3 : Text {
                text: block.text;
                wrap: word-wrap;
                color: #374151;
                font-size: 16px;
                font-weight: 600;
            }

            if block.block-type == BlockType.ListItem : HorizontalLayout {
                spacing: 8px;
                alignment: start;
                Text { text: "•"; color: #6b7280; font-size: 15px; }
                Text {
                    text: block.text;
                    wrap: word-wrap;
                    color: #374151;
                    font-size: 15px;
                    line-height: 1.6;
                }
            }

            if block.block-type == BlockType.CodeBlock : Rectangle {
                background: #f4f4f4;
                border-radius: 8px;
                border-width: 1px;
                border-color: #e5e5e5;

                VerticalLayout {
                    padding: 14px;
                    spacing: 4px;

                    if block.lang != "" : Text {
                        text: block.lang;
                        color: #9ca3af;
                        font-size: 11px;
                    }

                    Text {
                        text: block.text;
                        font-family: "monospace";
                        font-size: 13px;
                        color: #1f2937;
                        wrap: no-wrap;
                    }
                }
            }

            if block.block-type == BlockType.HorizontalRule : Rectangle {
                height: 1px;
                background: #e5e5e5;
            }
        }
    }
}
```

**Step 2: Build + Commit**
```bash
cargo build 2>&1 | grep -E "^error"
git add ui/components/markdown_view.slint
git commit -m "style: markdown view light theme"
```

---

### Task 4: components/message_bubble.slint — ChatGPT message style

**Files:**
- Modify: `ui/components/message_bubble.slint`

ChatGPT style:
- User: right-aligned rounded pill, #f4f4f4 bg, dark text, max-width 520px
- AI: left-aligned, small green avatar + MarkdownView, no bubble bg

**Step 1: Rewrite**

```slint
import { MarkdownView, MarkdownBlock, BlockType } from "markdown_view.slint";

export struct MessageItem {
    is-user: bool,
    content: string,
    blocks: [MarkdownBlock],
}

export component MessageBubble {
    in property <MessageItem> message;

    // User message: right-aligned pill bubble
    if root.message.is-user : HorizontalLayout {
        alignment: end;
        padding-left: 80px;

        Rectangle {
            background: #f4f4f4;
            border-radius: 18px;
            padding-top: 10px;
            padding-bottom: 10px;
            padding-left: 16px;
            padding-right: 16px;
            max-width: 520px;

            Text {
                text: root.message.content;
                wrap: word-wrap;
                color: #0d0d0d;
                font-size: 15px;
                line-height: 1.6;
            }
        }
    }

    // AI message: avatar + markdown, no bubble
    if !root.message.is-user : HorizontalLayout {
        alignment: start;
        spacing: 12px;
        padding-right: 80px;

        // AI avatar
        Rectangle {
            width: 28px;
            height: 28px;
            background: #10a37f;
            border-radius: 6px;
            vertical-stretch: 0;

            Text {
                text: "AI";
                color: #ffffff;
                font-size: 10px;
                font-weight: 700;
                horizontal-alignment: center;
                vertical-alignment: center;
            }
        }

        MarkdownView {
            blocks: root.message.blocks;
            horizontal-stretch: 1;
        }
    }
}
```

**Step 2: Build + Commit**
```bash
cargo build 2>&1 | grep -E "^error"
git add ui/components/message_bubble.slint
git commit -m "style: ChatGPT-style message bubbles with AI avatar"
```

---

### Task 5: chat_view.slint — centered content layout

**Files:**
- Modify: `ui/chat_view.slint`

ChatGPT centers content with max-width ~768px.

**Step 1: Rewrite**

```slint
import { MessageBubble, MessageItem } from "components/message_bubble.slint";
import { ScrollView } from "std-widgets.slint";

export component ChatView inherits Rectangle {
    in property <[MessageItem]> messages;
    in property <bool> parallel-mode: false;
    in property <[MessageItem]> parallel-messages-b;

    background: #ffffff;

    // Normal mode
    if !root.parallel-mode : Rectangle {
        background: #ffffff;

        if root.messages.length == 0 : VerticalLayout {
            alignment: center;
            spacing: 12px;

            Text {
                text: "有什么可以帮你的？";
                color: #0d0d0d;
                font-size: 24px;
                font-weight: 600;
                horizontal-alignment: center;
            }
            Text {
                text: "开始一段新对话";
                color: #6e6e80;
                font-size: 15px;
                horizontal-alignment: center;
            }
        }

        if root.messages.length > 0 : ScrollView {
            HorizontalLayout {
                alignment: center;

                VerticalLayout {
                    max-width: 768px;
                    horizontal-stretch: 1;
                    padding-top: 24px;
                    padding-bottom: 24px;
                    spacing: 24px;
                    alignment: start;

                    for msg in root.messages : MessageBubble {
                        message: msg;
                    }
                }
            }
        }
    }

    // Parallel mode: two columns
    if root.parallel-mode : HorizontalLayout {
        // Left column
        Rectangle {
            background: #ffffff;
            horizontal-stretch: 1;

            if root.messages.length == 0 : Text {
                text: "模型 A";
                color: #9ca3af;
                font-size: 15px;
                horizontal-alignment: center;
                vertical-alignment: center;
            }

            if root.messages.length > 0 : ScrollView {
                VerticalLayout {
                    padding: 24px;
                    spacing: 24px;
                    alignment: start;

                    for msg in root.messages : MessageBubble {
                        message: msg;
                    }
                }
            }
        }

        Rectangle {
            width: 1px;
            background: #e5e5e5;
        }

        // Right column
        Rectangle {
            background: #ffffff;
            horizontal-stretch: 1;

            if root.parallel-messages-b.length == 0 : Text {
                text: "模型 B";
                color: #9ca3af;
                font-size: 15px;
                horizontal-alignment: center;
                vertical-alignment: center;
            }

            if root.parallel-messages-b.length > 0 : ScrollView {
                VerticalLayout {
                    padding: 24px;
                    spacing: 24px;
                    alignment: start;

                    for msg in root.parallel-messages-b : MessageBubble {
                        message: msg;
                    }
                }
            }
        }
    }
}
```

**Step 2: Build + Commit**
```bash
cargo build 2>&1 | grep -E "^error"
git add ui/chat_view.slint
git commit -m "style: centered chat layout with ChatGPT welcome screen"
```

---

### Task 6: components/quick_phrases.slint — light theme

**Files:**
- Modify: `ui/components/quick_phrases.slint`

**Step 1: Update colors**

- bg: `#1a1a2e` → `#ffffff`
- border: `#333` → `#e5e5e5`
- label: `#888` → `#9ca3af`
- empty text: `#555` → `#9ca3af`
- phrase name: keep `#10a37f` (ChatGPT green)
- phrase content: `#aaa` → `#6e6e80`
- delete btn: `#555` → `#9ca3af`
- hover bg: transparent → `#f4f4f4`

```slint
export struct PhraseItem {
    id: string,
    name: string,
    content: string,
}

export component QuickPhrasePanel inherits Rectangle {
    in property <[PhraseItem]> phrases;
    callback select(string);
    callback delete(string);

    width: 300px;
    background: #ffffff;
    border-radius: 10px;
    border-width: 1px;
    border-color: #e5e5e5;
    drop-shadow-blur: 12px;
    drop-shadow-color: #00000018;
    drop-shadow-offset-y: 4px;

    VerticalLayout {
        padding: 8px;
        spacing: 2px;

        Text {
            text: "快捷短语";
            color: #9ca3af;
            font-size: 11px;
            padding-left: 8px;
            padding-bottom: 4px;
        }

        if root.phrases.length == 0 : Text {
            text: "暂无快捷短语";
            color: #9ca3af;
            font-size: 13px;
            horizontal-alignment: center;
            padding-top: 8px;
            padding-bottom: 8px;
        }

        for phrase in root.phrases : HorizontalLayout {
            spacing: 4px;

            row-ta := TouchArea {
                horizontal-stretch: 1;
                clicked => { root.select(phrase.content); }
                mouse-cursor: pointer;

                Rectangle {
                    background: row-ta.has-hover ? #f4f4f4 : transparent;
                    border-radius: 6px;

                    HorizontalLayout {
                        padding: 8px;
                        spacing: 8px;

                        Text {
                            text: "/" + phrase.name;
                            color: #10a37f;
                            font-size: 13px;
                            font-weight: 600;
                            width: 80px;
                            overflow: elide;
                        }

                        Text {
                            text: phrase.content;
                            color: #6e6e80;
                            font-size: 13px;
                            overflow: elide;
                            horizontal-stretch: 1;
                        }
                    }
                }
            }

            del-ta := TouchArea {
                width: 28px;
                clicked => { root.delete(phrase.id); }

                Text {
                    text: "×";
                    color: del-ta.has-hover ? #ef4444 : #9ca3af;
                    font-size: 16px;
                    horizontal-alignment: center;
                    vertical-alignment: center;
                }
            }
        }
    }
}
```

**Step 2: Build + Commit**
```bash
cargo build 2>&1 | grep -E "^error"
git add ui/components/quick_phrases.slint
git commit -m "style: quick phrases panel light theme with shadow"
```

---

### Task 7: input_bar.slint — ChatGPT input style

**Files:**
- Modify: `ui/input_bar.slint`

ChatGPT input: centered rounded box, send button inside (dark circle ↑), parallel toggle as subtle icon button.

**Step 1: Rewrite**

```slint
import { TextEdit } from "std-widgets.slint";
import { QuickPhrasePanel, PhraseItem } from "components/quick_phrases.slint";

export { PhraseItem }

export component InputBar inherits Rectangle {
    in property <bool> sending: false;
    in property <int> token-used: 0;
    in property <int> token-max: 8192;
    in property <float> token-ratio: 0.0;
    in-out property <bool> parallel-mode: false;
    in property <[PhraseItem]> phrases;
    in-out property <bool> show-phrases: false;
    callback send(string);
    callback clear-context();
    callback toggle-parallel();
    callback phrase-selected(string);
    callback phrase-deleted(string);

    property <string> input-text: "";

    background: #ffffff;
    border-color: #e5e5e5;
    border-width: 1px;

    VerticalLayout {
        // Token progress bar (subtle, top of input area)
        Rectangle {
            height: 2px;
            background: #f4f4f4;
            Rectangle {
                width: root.token-ratio * parent.width;
                height: 2px;
                x: 0;
                background: root.token-ratio < 0.6 ? #10a37f
                          : root.token-ratio < 0.85 ? #f59e0b
                          : #ef4444;
            }
        }

        HorizontalLayout {
            alignment: center;
            padding-top: 12px;
            padding-bottom: 16px;
            padding-left: 16px;
            padding-right: 16px;

            // Centered input container
            Rectangle {
                max-width: 768px;
                horizontal-stretch: 1;
                background: #ffffff;
                border-radius: 16px;
                border-width: 1px;
                border-color: #e5e5e5;
                drop-shadow-blur: 8px;
                drop-shadow-color: #0000000a;
                drop-shadow-offset-y: 2px;

                HorizontalLayout {
                    padding-left: 16px;
                    padding-right: 8px;
                    padding-top: 8px;
                    padding-bottom: 8px;
                    spacing: 8px;
                    alignment: center-y;

                    // Parallel mode toggle (subtle icon)
                    parallel-ta := TouchArea {
                        width: 32px;
                        height: 32px;
                        clicked => { root.toggle-parallel(); }

                        Rectangle {
                            background: root.parallel-mode ? #10a37f
                                      : parallel-ta.has-hover ? #f4f4f4
                                      : transparent;
                            border-radius: 8px;

                            Text {
                                text: "⊞";
                                color: root.parallel-mode ? #ffffff : #9ca3af;
                                font-size: 16px;
                                horizontal-alignment: center;
                                vertical-alignment: center;
                            }
                        }
                    }

                    TextEdit {
                        text <=> root.input-text;
                        enabled: !root.sending;
                        placeholder-text: "发消息给 Slint Chat";
                        font-size: 15px;
                        horizontal-stretch: 1;

                        key-pressed(event) => {
                            if event.text == "/" {
                                root.show-phrases = true;
                            } else if event.text == "\n" && !event.modifiers.shift {
                                root.show-phrases = false;
                                if root.input-text != "" {
                                    root.send(root.input-text);
                                    root.input-text = "";
                                }
                                return accept;
                            } else {
                                root.show-phrases = false;
                            }
                            return reject;
                        }
                    }

                    // Token count (subtle)
                    Text {
                        text: root.token-used + "/" + root.token-max;
                        color: #9ca3af;
                        font-size: 11px;
                        vertical-alignment: center;
                    }

                    // Send button: dark circle with ↑
                    send-ta := TouchArea {
                        width: 34px;
                        height: 34px;
                        enabled: !root.sending;
                        clicked => {
                            if root.input-text != "" {
                                root.send(root.input-text);
                                root.input-text = "";
                            }
                        }

                        Rectangle {
                            background: root.sending ? #d1d5db
                                      : root.input-text == "" ? #d1d5db
                                      : #0d0d0d;
                            border-radius: 17px;

                            Text {
                                text: root.sending ? "…" : "↑";
                                color: #ffffff;
                                font-size: 16px;
                                horizontal-alignment: center;
                                vertical-alignment: center;
                            }
                        }
                    }
                }
            }
        }
    }

    // Quick phrase panel (floats above input)
    if root.show-phrases : QuickPhrasePanel {
        phrases: root.phrases;
        x: 16px;
        y: -self.preferred-height - 8px;
        select(content) => {
            root.input-text = content;
            root.show-phrases = false;
            root.phrase-selected(content);
        }
        delete(id) => {
            root.phrase-deleted(id);
        }
    }
}
```

**Step 2: Build + Commit**
```bash
cargo build 2>&1 | grep -E "^error"
git add ui/input_bar.slint
git commit -m "style: ChatGPT-style centered input with send button"
```

---

### Task 8: Final build + push

```bash
cargo build --release 2>&1 | grep -E "^error|Finished"
git push
```
