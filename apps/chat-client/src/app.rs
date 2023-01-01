use dtos::{ChatMessage, CreateChatDto, GetChatDto, JsonResponse, SendChatMessageDto};
use eyre::Result;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

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
    let json = format!("{{ \"username\":\"{}\"}}", username);

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
        create_chat_wrapper(username.clone())
    });
    let update_username = move |ev: web_sys::InputEvent| {
        let value = event_target_value(&ev);
        set_username(value);
    };

    create_effect(cx, move |_| {
        let mby_rsp = action.value.get();
        if let Some(resp) = mby_rsp {
            let storage = window().local_storage().unwrap().unwrap();
            storage.set_item("username", &resp.data.user_id).unwrap();
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

async fn send_chat_message(message: String, chat_id: String) -> Result<()> {
    let storage = window().local_storage().unwrap().unwrap();
    let username = storage.get_item("username").unwrap().unwrap();

    let dto = SendChatMessageDto {
        message,
        chat_id,
        username,
    };

    let json = serde_json::to_string(&dto).unwrap();

    reqwasm::http::Request::post("http://localhost:8081/send-chat-message")
        .body(json)
        .send()
        .await?
        .json::<GetChatDto>()
        .await?;
    Ok(())
}

#[component]
pub fn ChatPage(cx: Scope) -> Element {
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

    let action = create_action(cx, |username: &String| {
        create_chat_wrapper(username.clone())
    });

    view! {
        cx,
        <section>
            <div>
                <For
                each=messages
                key=|it| it.message.clone() >
                { |cx:Scope, it:&ChatMessage| view! {cx, <ChatMessage text=it.message.clone()  />}}
                </For>
            </div>
            <div class="chat-input" >
                <input type="text" class="input" />
                <button class="button" >"Send!"</button>
            </div>
        </section>
    }
}

#[component]
pub fn ChatMessage(cx: Scope, text: String) -> Element {
    view! {
        cx,
        <div class="chat-message">
            <div class="text">
                {text}
            </div>
        </div>
    }
}
