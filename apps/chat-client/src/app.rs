use leptos::*;
use leptos_meta::*;

#[component]
pub fn App(cx: Scope) -> Element {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_context(cx, MetaContext::default());

    // Creates a reactive value to update the button
    let (count, set_count) = create_signal(cx, 0);
    let on_click = move |_| set_count.update(|count| *count += 1);

    view! {
        cx,
        <main>
            <Title text="cargo-leptos starter"/>
            <Stylesheet href="/style.css"/>
            <h1>"Welcome to Leptos!"</h1>
            <button on:click=on_click>"Click Me: " {count}</button>
            <div> "Hello World!" {count} </div>
        </main>
    }
}
