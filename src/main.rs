use slint::{Model, ModelRc, SharedString, VecModel};
use std::cell::RefCell;
use std::rc::Rc;

use slint_chat::api::client::ApiClient;
use slint_chat::api::stream::{parse_sse_stream, StreamToken};
use slint_chat::app_state::AppState;
use slint_chat::markdown::renderer::{self, BlockType as MdBlockType};
use slint_chat::models::message::{Message, Role};
use slint_chat::models::topic::Topic;
use slint_chat::settings::Settings;
use slint_chat::storage::db::Db;

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

fn sync_topics_to_ui(ui: &AppWindow, state: &AppState) {
    let items: Vec<TopicItem> = state
        .topics
        .iter()
        .map(|t| TopicItem {
            id: t.id.clone().into(),
            title: t.title.clone().into(),
        })
        .collect();
    ui.set_topics(ModelRc::new(VecModel::from(items)));
}

fn messages_to_slint(messages: &[Message]) -> Vec<MessageItem> {
    messages
        .iter()
        .map(|m| {
            let is_user = m.role == Role::User;
            let blocks = if is_user {
                Vec::new()
            } else {
                to_slint_blocks(&renderer::parse(&m.content))
            };
            MessageItem {
                is_user,
                content: m.content.clone().into(),
                blocks: ModelRc::new(VecModel::from(blocks)),
                reasoning: SharedString::default(),
                reasoning_streaming: false,
            }
        })
        .collect()
}

fn sync_messages_to_ui(ui: &AppWindow, messages_model: &Rc<VecModel<MessageItem>>, state: &AppState) {
    let items = messages_to_slint(state.active_messages());
    let count = messages_model.row_count();
    for _ in 0..count {
        messages_model.remove(0);
    }
    for item in items {
        messages_model.push(item);
    }
    ui.set_messages(messages_model.clone().into());
}

fn update_last_assistant_message(
    messages_model: &Rc<VecModel<MessageItem>>,
    content: &str,
    reasoning: &str,
    reasoning_streaming: bool,
) {
    let count = messages_model.row_count();
    if count == 0 {
        return;
    }
    let idx = count - 1;
    let blocks = to_slint_blocks(&renderer::parse(content));
    let item = MessageItem {
        is_user: false,
        content: content.into(),
        blocks: ModelRc::new(VecModel::from(blocks)),
        reasoning: reasoning.into(),
        reasoning_streaming,
    };
    messages_model.set_row_data(idx, item);
}

fn config_dir() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("slint-chat")
}

fn main() {
    // --- Init ---
    let settings = Rc::new(RefCell::new(Settings::load()));
    let db_path = config_dir().join("chat.db");
    std::fs::create_dir_all(db_path.parent().unwrap()).ok();
    let db = Rc::new(Db::init(db_path.to_str().unwrap()).expect("Failed to init database"));

    let mut initial_state = db.load_all().unwrap_or_default();

    // Ensure at least one topic exists
    if initial_state.topics.is_empty() {
        let topic = Topic {
            id: uuid::Uuid::new_v4().to_string(),
            assistant_id: String::new(),
            title: "新对话".into(),
            model: settings.borrow().model.clone(),
            created_at: now_ts(),
        };
        db.save_topic(&topic).ok();
        initial_state.topics.push(topic);
    }

    // Set active topic to the first one
    initial_state.active_topic_id = Some(initial_state.topics[0].id.clone());

    let state = Rc::new(RefCell::new(initial_state));

    // --- UI ---
    let ui = AppWindow::new().unwrap();

    // Populate settings UI
    {
        let s = settings.borrow();
        ui.set_settings_api_key(s.api_key.clone().into());
        ui.set_settings_api_base_url(s.api_base_url.clone().into());
        ui.set_settings_model(s.model.clone().into());
        ui.set_current_model(s.model.clone().into());
    }

    // Populate model options
    {
        let h = |name: &str| ModelOption {
            id: SharedString::default(),
            name: name.into(),
            is_header: true,
        };
        let m = |id: &str, name: &str| ModelOption {
            id: id.into(),
            name: name.into(),
            is_header: false,
        };
        let options = vec![
            h("OpenAI"),
            m("gpt-4o", "GPT-4o"),
            m("gpt-4o-mini", "GPT-4o Mini"),
            m("o1", "o1"),
            m("o1-mini", "o1 Mini"),
            h("Anthropic"),
            m("claude-opus-4-20250514", "Claude 4 Opus"),
            m("claude-sonnet-4-20250514", "Claude 4 Sonnet"),
            m("claude-3-5-sonnet-20241022", "Claude 3.5 Sonnet"),
            m("claude-3-5-haiku-20241022", "Claude 3.5 Haiku"),
            h("Google"),
            m("gemini-2.0-flash", "Gemini 2.0 Flash"),
            m("gemini-1.5-pro", "Gemini 1.5 Pro"),
            h("DeepSeek"),
            m("deepseek-reasoner", "DeepSeek R1"),
            m("deepseek-chat", "DeepSeek V3"),
            h("Meta"),
            m("llama-3.3-70b", "Llama 3.3 70B"),
            m("llama-3.1-8b", "Llama 3.1 8B"),
            h("Mistral AI"),
            m("mistral-large-latest", "Mistral Large"),
            m("mistral-small-latest", "Mistral Small"),
            h("Alibaba"),
            m("qwen-max", "Qwen Max"),
            m("qwen-plus", "Qwen Plus"),
            h("xAI"),
            m("grok-3", "Grok 3"),
            m("grok-2-1212", "Grok 2"),
        ];
        ui.set_model_options(ModelRc::new(VecModel::from(options)));
    }

    // Populate topics
    sync_topics_to_ui(&ui, &state.borrow());

    // Set active index to 0
    ui.set_active_index(0);

    // Populate messages for active topic
    let messages_model = Rc::new(VecModel::<MessageItem>::default());
    sync_messages_to_ui(&ui, &messages_model, &state.borrow());

    // --- Tokio runtime for async HTTP ---
    let rt = Rc::new(
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime"),
    );

    // --- Settings callbacks ---
    {
        let ui_handle = ui.as_weak();
        ui.on_open_settings(move || {
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_show_settings(true);
            }
        });
    }
    {
        let ui_handle = ui.as_weak();
        let settings = settings.clone();
        ui.on_save_settings(move || {
            if let Some(ui) = ui_handle.upgrade() {
                let mut s = settings.borrow_mut();
                s.api_key = ui.get_settings_api_key().to_string();
                s.api_base_url = ui.get_settings_api_base_url().to_string();
                s.model = ui.get_settings_model().to_string();
                if let Err(e) = s.save() {
                    eprintln!("Failed to save settings: {e}");
                }
                ui.set_current_model(s.model.clone().into());
                ui.set_show_settings(false);
            }
        });
    }
    {
        let ui_handle = ui.as_weak();
        let settings = settings.clone();
        ui.on_cancel_settings(move || {
            if let Some(ui) = ui_handle.upgrade() {
                // Reload from saved settings
                let s = Settings::load();
                ui.set_settings_api_key(s.api_key.clone().into());
                ui.set_settings_api_base_url(s.api_base_url.clone().into());
                ui.set_settings_model(s.model.clone().into());
                *settings.borrow_mut() = s;
                ui.set_show_settings(false);
            }
        });
    }

    // --- Model selected ---
    {
        let ui_handle = ui.as_weak();
        let settings = settings.clone();
        ui.on_model_selected(move |model_id| {
            let model_id = model_id.to_string();
            if let Some(ui) = ui_handle.upgrade() {
                let mut s = settings.borrow_mut();
                s.model = model_id.clone();
                if let Err(e) = s.save() {
                    eprintln!("Failed to save settings: {e}");
                }
                ui.set_current_model(model_id.into());
                ui.set_settings_model(s.model.clone().into());
            }
        });
    }

    // --- New topic ---
    {
        let ui_handle = ui.as_weak();
        let state = state.clone();
        let db = db.clone();
        let settings = settings.clone();
        let messages_model = messages_model.clone();
        ui.on_new_topic(move || {
            let ui = ui_handle.upgrade().unwrap();
            let topic = Topic {
                id: uuid::Uuid::new_v4().to_string(),
                assistant_id: String::new(),
                title: "新对话".into(),
                model: settings.borrow().model.clone(),
                created_at: now_ts(),
            };
            db.save_topic(&topic).ok();

            let mut s = state.borrow_mut();
            s.topics.insert(0, topic.clone());
            s.active_topic_id = Some(topic.id.clone());

            sync_topics_to_ui(&ui, &s);
            ui.set_active_index(0);
            sync_messages_to_ui(&ui, &messages_model, &s);
        });
    }

    // --- Topic selected ---
    {
        let ui_handle = ui.as_weak();
        let state = state.clone();
        let messages_model = messages_model.clone();
        ui.on_topic_selected(move |index| {
            let ui = ui_handle.upgrade().unwrap();
            ui.set_show_settings(false);
            let mut s = state.borrow_mut();
            if let Some(topic) = s.topics.get(index as usize) {
                s.active_topic_id = Some(topic.id.clone());
                sync_messages_to_ui(&ui, &messages_model, &s);
            }
        });
    }

    // --- Send message ---
    {
        let ui_handle = ui.as_weak();
        let state = state.clone();
        let db = db.clone();
        let settings = settings.clone();
        let messages_model = messages_model.clone();
        let rt = rt.clone();
        ui.on_send(move |text| {
            let text = text.to_string().trim().to_string();
            if text.is_empty() {
                return;
            }

            let ui = ui_handle.upgrade().unwrap();
            let s = settings.borrow();

            if s.api_key.is_empty() {
                ui.set_show_settings(true);
                return;
            }

            ui.set_sending(true);

            let topic_id;
            let api_messages;
            {
                let mut st = state.borrow_mut();
                topic_id = st.active_topic_id.clone().unwrap();

                // Create user message
                let user_msg = Message {
                    id: uuid::Uuid::new_v4().to_string(),
                    topic_id: topic_id.clone(),
                    role: Role::User,
                    content: text.clone(),
                    created_at: now_ts(),
                };
                db.save_message(&user_msg).ok();
                st.push_message(user_msg);

                // Auto-update topic title on first message
                if let Some(topic) = st.topics.iter_mut().find(|t| t.id == topic_id) {
                    if topic.title == "新对话" {
                        let title: String = text.chars().take(20).collect();
                        topic.title = title;
                        db.save_topic(topic).ok();
                        sync_topics_to_ui(&ui, &st);
                    }
                }

                // Get all messages for API call
                api_messages = st.messages.get(&topic_id).cloned().unwrap_or_default();
            }

            // Add user message to UI
            messages_model.push(MessageItem {
                is_user: true,
                content: SharedString::from(&text),
                blocks: ModelRc::default(),
                reasoning: SharedString::default(),
                reasoning_streaming: false,
            });

            // Add empty assistant message placeholder
            messages_model.push(MessageItem {
                is_user: false,
                content: SharedString::default(),
                blocks: ModelRc::default(),
                reasoning: SharedString::default(),
                reasoning_streaming: false,
            });

            // Create channel for streaming tokens
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<StreamToken>();

            // Spawn HTTP request on tokio runtime
            let api_key = s.api_key.clone();
            let base_url = s.api_base_url.clone();
            let model = s.model.clone();
            rt.spawn(async move {
                let client = ApiClient::new(&base_url, &api_key);
                match client.chat_stream(&model, &api_messages).await {
                    Ok(response) => {
                        let status = response.status();
                        if !status.is_success() {
                            let body = response.text().await.unwrap_or_default();
                            eprintln!("API error {status}: {body}");
                            tx.send(StreamToken::Content(format!("API 错误 ({status}): {body}"))).ok();
                            return;
                        }
                        let tx_clone = tx.clone();
                        if let Err(e) = parse_sse_stream(response, |token| {
                            tx_clone.send(token).ok();
                        })
                        .await
                        {
                            eprintln!("SSE stream error: {e}");
                            tx.send(StreamToken::Content(format!("流式传输错误: {e}"))).ok();
                        }
                    }
                    Err(e) => {
                        eprintln!("Request error: {e}");
                        tx.send(StreamToken::Content(format!("请求错误: {e}"))).ok();
                    }
                }
            });

            // Spawn local task to receive tokens and update UI
            let ui_handle2 = ui.as_weak();
            let messages_model2 = messages_model.clone();
            let state2 = state.clone();
            let db2 = db.clone();
            let topic_id2 = topic_id.clone();
            slint::spawn_local(async move {
                let mut full_content = String::new();
                let mut full_reasoning = String::new();
                let mut is_reasoning = false;

                while let Some(token) = rx.recv().await {
                    match token {
                        StreamToken::Reasoning(t) => {
                            full_reasoning.push_str(&t);
                            is_reasoning = true;
                        }
                        StreamToken::Content(t) => {
                            full_content.push_str(&t);
                            is_reasoning = false;
                        }
                    }
                    if let Some(_ui) = ui_handle2.upgrade() {
                        update_last_assistant_message(
                            &messages_model2,
                            &full_content,
                            &full_reasoning,
                            is_reasoning,
                        );
                    }
                }
                // Stream finished — save assistant message
                if let Some(ui) = ui_handle2.upgrade() {
                    // Final update with reasoning_streaming = false
                    update_last_assistant_message(
                        &messages_model2,
                        &full_content,
                        &full_reasoning,
                        false,
                    );
                    let assistant_msg = Message {
                        id: uuid::Uuid::new_v4().to_string(),
                        topic_id: topic_id2,
                        role: Role::Assistant,
                        content: full_content,
                        created_at: now_ts(),
                    };
                    db2.save_message(&assistant_msg).ok();
                    state2.borrow_mut().push_message(assistant_msg);
                    ui.set_sending(false);
                }
            })
            .ok();
        });
    }

    ui.run().unwrap();
}

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
