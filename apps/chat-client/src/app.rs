use dtos::{ChatMessage, CreateChatDto, GetChatDto, JsonResponse};
use eyre::Result;
use leptos::{web_sys::KeyboardEvent, *};
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, json, to_string};

#[derive(Serialize, Deserialize, Default)]
struct ClientState {
    username: String,
    username_hash: String,
}

fn get_client_state() -> ClientState {
    let storage = window().local_storage().unwrap().unwrap();
    let json = match storage.get_item("client-state") {
        Ok(res) => match res {
            Some(it) => it,
            None => "".to_string(),
        },
        Err(_) => "".to_string(),
    };
    if json.is_empty() {
        return ClientState::default();
    }
    match from_str(&json) {
        Ok(it) => it,
        Err(_) => ClientState::default(),
    }
}

fn set_client_state(state: &ClientState) {
    let storage = window().local_storage().ok().flatten().unwrap();
    let json = to_string(state).unwrap();
    storage.set_item("client-state", &json).unwrap()
}

#[component]
pub fn App(cx: Scope) -> Element {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_context(cx, MetaContext::default());
    view! {
        cx,
        <div>
            <Title text="cargo-leptos starter"/>
            <Stylesheet href="/style.css"/>
            <Router>
                <main>
                    <Routes>
                        <Route
                            path="/"
                            element=move |cx| view! { cx,  <CreateChatPage /> }
                        >
                        </Route>
                        <Route
                            path="/chats/:chat_id"
                            element=move |cx| view! { cx,  <ChatPage /> }
                        >
                        </Route>
                    </Routes>

                </main>
            </Router>
        </div>
    }
}

pub async fn create_chat(username: String) -> Result<JsonResponse<CreateChatDto>> {
    let json = json!({
        "username": username,
    })
    .to_json()
    .unwrap();

    let res = reqwasm::http::Request::post("http://localhost:8081/create-chat")
        .body(json)
        .header("Content-Type", "application/json")
        .send()
        .await?
        .json::<JsonResponse<CreateChatDto>>()
        .await?;
    Ok(res)
}

pub async fn create_chat_wrapper(username: String) -> JsonResponse<CreateChatDto> {
    let res = create_chat(username).await.unwrap();
    res
}

#[component]
pub fn CreateChatPage(cx: Scope) -> Element {
    let navigator = use_navigate(cx);
    let (username, set_username) = create_signal::<String>(cx, "".to_string());

    let action = create_action(cx, |username: &String| {
        let mut state = get_client_state();
        state.username = username.clone();
        set_client_state(&state);
        create_chat_wrapper(username.clone())
    });
    let update_username = move |ev: web_sys::InputEvent| {
        let value = event_target_value(&ev);
        set_username(value);
    };

    create_effect(cx, move |_| {
        let mby_rsp = action.value.get();
        if let Some(resp) = mby_rsp {
            let mut state = get_client_state();
            state.username_hash = resp.data.user_id;
            set_client_state(&state);
            let route = format!("/chats/{}", resp.data.chat_id);
            navigator(&route, NavigateOptions::default()).unwrap();
        };
    });

    view! {
        cx,
        <div class="card" >
            <h3>"Chat with me! 🌟"</h3>
            <div class="input-container">
                <label>"Username"</label>
                <input type="text" on:input=update_username  />
            </div>
            <div style="width: 100%;" >
                <button class="button" style="margin-left: auto;" on:click=move|_| action.dispatch(username())>"Lets go 🎙️"</button>
            </div>
        </div>
    }
}

async fn fetch_messages(chat_id: String) -> Result<GetChatDto> {
    let res = reqwasm::http::Request::get(&format!("http://localhost:8080/chats/{}", chat_id))
        .send()
        .await?
        .json::<GetChatDto>()
        .await?;
    Ok(res)
}

async fn unwrap_fetch(chat_id: String) -> Result<GetChatDto, ()> {
    fetch_messages(chat_id).await.map_err(|_| ())
}

async fn send_chat_message(message: String, chat_id: String, username: String) -> Result<()> {
    let dto = SendChatMessageDto {
        message,
        chat_id,
        username,
    };

    let json = serde_json::to_string(&dto).unwrap();

    reqwasm::http::Request::post("http://localhost:8081/send-chat-message")
        .body(json)
        .header("Content-Type", "application/json")
        .send()
        .await?;
    Ok(())
}

async fn send_chat_message_wrapper(message: String, chat_id: String, username: String) {
    send_chat_message(message, chat_id, username).await.unwrap()
}

#[component]
pub fn ChatPage(cx: Scope) -> Element {
    // Code to fetch messages on startup
    let chat_id = move || {
        use_params_map(cx)
            .with(|p| p.get("chat_id").cloned())
            .unwrap()
    };
    let dto = create_resource(cx, chat_id, unwrap_fetch);
    let messages = move || {
        dto.with(|it| it.as_ref().unwrap().clone().messages)
            .unwrap_or(vec![])
    };

    // Code to bind a message in order to send chat messages
    let (message, set_message) = create_signal(cx, "".to_string());
    let update_message = move |ev: web_sys::InputEvent| {
        let value = event_target_value(&ev);
        set_message(value);
    };

    let btn_action = create_action(cx, move |message: &String| {
        let message = message.clone();
        let username = get_client_state().username;
        set_message("".to_string());
        send_chat_message_wrapper(message, chat_id(), username)
    });

    let enter_action = create_action(cx, move |message: &String| {
        let message = message.clone();
        let username = get_client_state().username;
        set_message("".to_string());
        send_chat_message_wrapper(message, chat_id(), username)
    });

    create_effect(cx, move |_| {
        if btn_action.value.get().is_some() {
            dto.refetch()
        };
        if enter_action.value.get().is_some() {
            dto.refetch()
        };
    });

    let on_keydown = move |e: KeyboardEvent| {
        // Means that enter was pressed
        if e.key_code() == 13 {
            enter_action.dispatch(message());
        }
    };

    view! {
        cx,
        <section>
            <div class="chat-messages">
                <For
                each=messages
                key=|it| it.message.clone() >
                {
                    |cx:Scope, it:&ChatMessage|
                    {
                        let hash = get_client_state().username_hash;
                        view! {
                            cx,
                            <ChatMessage
                                text=it.message.clone()
                                is_mine=it.sent_by == hash
                            />
                        }
                    }
                }
                </For>
            </div>
            <div class="chat-input" >
                <input type="text" class="input" value=message  on:input=update_message on:keydown=on_keydown />
                <button class="button" on:click=move |_| btn_action.dispatch(message()) >"Send!"</button>
            </div>
        </section>
    }
}

#[component]
pub fn ChatMessage(cx: Scope, text: String, is_mine: bool) -> Element {
    let who = if is_mine { "mine" } else { "theirs" };
    let message_class = format!("chat-message {}", who);

    view! {
        cx,
        <div class=message_class>
            <div class="text">
                {text}
            </div>
        </div>
    }
}
