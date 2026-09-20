use leptos::prelude::*;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Clone, Deserialize)]
struct DayData {
    day: usize,
    focus: String,
    sentences: Vec<Item>,
    words: Vec<Item>,
}

#[derive(Clone, Deserialize)]
struct Item {
    tamil: String,
    english: String,
}

#[derive(Clone)]
struct Card {
    day: usize,
    kind: &'static str,
    front: String,
    back: String,
}

fn load_data() -> Vec<DayData> {
    serde_json::from_str(include_str!("../data/flashcards.json"))
        .expect("data/flashcards.json should be valid flashcard data")
}

fn random_index(len: usize) -> usize {
    if len == 0 {
        return 0;
    }

    #[cfg(target_arch = "wasm32")]
    {
        (js_sys::Math::random() * len as f64).floor() as usize
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        0
    }
}

fn google_translate_url(text: &str) -> String {
    let mut encoded = String::new();

    for byte in text.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            b' ' => encoded.push_str("%20"),
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }

    format!("https://translate.google.com/?sl=ta&tl=en&text={encoded}&op=translate")
}

fn cards_until(data: &[DayData], max_day: usize) -> Vec<Card> {
    data.iter()
        .filter(|day| day.day <= max_day)
        .flat_map(|day| {
            let sentence_cards = day.sentences.iter().map(|item| Card {
                day: day.day,
                kind: "Sentence",
                front: item.english.clone(),
                back: item.tamil.clone(),
            });

            let word_cards = day.words.iter().map(|item| Card {
                day: day.day,
                kind: "Word",
                front: item.english.clone(),
                back: item.tamil.clone(),
            });

            sentence_cards.chain(word_cards)
        })
        .collect()
}

fn main() {
    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let data = Arc::new(load_data());
    let total_days = data.len();

    let (max_day, set_max_day) = signal(1usize);
    let (day_input, set_day_input) = signal("1".to_string());
    let (card_index, set_card_index) = signal(random_index(cards_until(&data, 1).len()));
    let (show_answer, set_show_answer) = signal(false);
    let (show_list, set_show_list) = signal(false);
    let (dark_mode, set_dark_mode) = signal(false);

    let current_cards = {
        let data = Arc::clone(&data);
        move || cards_until(&data, max_day.get())
    };

    let current_card = {
        let current_cards = current_cards.clone();
        move || {
            let cards = current_cards();
            if cards.is_empty() {
                None
            } else {
                Some(cards[card_index.get().min(cards.len() - 1)].clone())
            }
        }
    };

    let reset_card = {
        let current_cards = current_cards.clone();
        move || {
            let len = current_cards().len();
            set_card_index.set(random_index(len));
            set_show_answer.set(false);
        }
    };

    let apply_day = move || {
        let day = day_input.get().parse::<usize>().unwrap_or(1).clamp(1, total_days);
        set_max_day.set(day);
        set_day_input.set(day.to_string());
        reset_card();
    };

    view! {
        <main class="app" class:dark=move || dark_mode.get()>
            <style>{STYLE}</style>

            <header class="topbar">
                <div>
                    <h1>"Learn Tamil Flashcards"</h1>
                    <p>"Spoken Tamil practice using English transliteration."</p>
                </div>
                <nav>
                    <button
                        class:active=move || !show_list.get()
                        on:click=move |_| set_show_list.set(false)
                    >"Flashcards"</button>
                    <button
                        class:active=move || show_list.get()
                        on:click=move |_| set_show_list.set(true)
                    >"List"</button>
                    <button on:click=move |_| set_dark_mode.update(|dark| *dark = !*dark)>
                        {move || if dark_mode.get() { "Light mode" } else { "Dark mode" }}
                    </button>
                </nav>
            </header>

            <section class="controls">
                <label>
                    "Study until day "
                    <input
                        type="number"
                        min="1"
                        max=total_days
                        prop:value=move || day_input.get()
                        on:input=move |ev| set_day_input.set(event_target_value(&ev))
                    />
                    <span>" / " {total_days}</span>
                </label>
                <button on:click=move |_| apply_day()>"Enter"</button>
                <span class="selected-day">{move || format!("Showing through day {}", max_day.get())}</span>
            </section>

            <Show
                when=move || show_list.get()
                fallback=move || view! {
                    <FlashcardPage
                        current_cards=current_cards.clone()
                        current_card=current_card.clone()
                        card_index=card_index
                        set_card_index=set_card_index
                        show_answer=show_answer
                        set_show_answer=set_show_answer
                    />
                }
            >
                <ListPage data=Arc::clone(&data) max_day=max_day />
            </Show>
        </main>
    }
}

#[component]
fn FlashcardPage(
    current_cards: impl Fn() -> Vec<Card> + Clone + Send + Sync + 'static,
    current_card: impl Fn() -> Option<Card> + Clone + Send + Sync + 'static,
    card_index: ReadSignal<usize>,
    set_card_index: WriteSignal<usize>,
    show_answer: ReadSignal<bool>,
    set_show_answer: WriteSignal<bool>,
) -> impl IntoView {
    let total_cards = {
        let current_cards = current_cards.clone();
        move || current_cards().len()
    };

    let next = {
        let current_cards = current_cards.clone();
        move || {
            let len = current_cards().len();
            if len > 0 {
                let current = card_index.get();
                let mut next = random_index(len);
                if len > 1 {
                    while next == current {
                        next = random_index(len);
                    }
                }
                set_card_index.set(next);
                set_show_answer.set(false);
            }
        }
    };

    let previous = {
        let current_cards = current_cards.clone();
        move || {
            let len = current_cards().len();
            if len > 0 {
                set_card_index.update(|index| *index = if *index == 0 { len - 1 } else { *index - 1 });
                set_show_answer.set(false);
            }
        }
    };

    let next_for_card = next.clone();
    let next_for_show_button = next.clone();

    view! {
        <section class="page">
            <div class="progress">
                {move || format!("Card {} of {}", card_index.get() + 1, total_cards())}
            </div>

            <div
                class="card"
                role="button"
                tabindex="0"
                on:click=move |_| {
                    if show_answer.get() {
                        next_for_card();
                    } else {
                        set_show_answer.set(true);
                    }
                }
            >
                {move || match current_card() {
                    Some(card) => {
                        let answer = card.back.clone();
                        let translate_url = google_translate_url(&answer);
                        view! {
                            <div>
                                <div class="meta">{format!("Day {} · {}", card.day, card.kind)}</div>
                                <div class="front">{card.front}</div>
                                <div class="hint">"Tap to reveal Tamil. Tap again for next random card."</div>
                                {move || show_answer.get().then(|| view! {
                                    <div class="answer-block">
                                        <div class="back">{answer.clone()}</div>
                                        <a
                                            class="translate-link"
                                            href=translate_url.clone()
                                            target="_blank"
                                            rel="noopener noreferrer"
                                            on:click=move |ev| ev.stop_propagation()
                                        >
                                            "Google Translate"
                                        </a>
                                    </div>
                                })}
                            </div>
                        }.into_any()
                    },
                    None => view! { <div>"No cards available."</div> }.into_any(),
                }}
            </div>

            <div class="actions">
                <button on:click=move |_| previous()>"Previous"</button>
                <button on:click=move |_| {
                    if show_answer.get() {
                        next_for_show_button();
                    } else {
                        set_show_answer.set(true);
                    }
                }>
                    {move || if show_answer.get() { "Next Random" } else { "Show Tamil" }}
                </button>
                <button on:click=move |_| next()>"Random"</button>
            </div>
        </section>
    }
}

#[component]
fn ListPage(data: Arc<Vec<DayData>>, max_day: ReadSignal<usize>) -> impl IntoView {
    view! {
        <section class="page list-page">
            <h2>{move || format!("All data through day {}", max_day.get())}</h2>
            <For
                each={move || data.iter().filter(|day| day.day <= max_day.get()).cloned().collect::<Vec<_>>()}
                key={|day| day.day}
                children={move |day| view! {
                    <article class="day-block">
                        <h3>{format!("Day {}: {}", day.day, day.focus)}</h3>
                        <h4>"Sentences"</h4>
                        <ul>
                            {day.sentences.into_iter().map(|item| {
                                let translate_url = google_translate_url(&item.tamil);
                                view! {
                                    <li>
                                        <strong>{item.tamil}</strong>" = "{item.english}
                                        " "
                                        <a class="translate-link small" href=translate_url target="_blank" rel="noopener noreferrer">"Translate"</a>
                                    </li>
                                }
                            }).collect_view()}
                        </ul>
                        <h4>"Words"</h4>
                        <ul>
                            {day.words.into_iter().map(|item| {
                                let translate_url = google_translate_url(&item.tamil);
                                view! {
                                    <li>
                                        <strong>{item.tamil}</strong>" = "{item.english}
                                        " "
                                        <a class="translate-link small" href=translate_url target="_blank" rel="noopener noreferrer">"Translate"</a>
                                    </li>
                                }
                            }).collect_view()}
                        </ul>
                    </article>
                }}
            />
        </section>
    }
}

const STYLE: &str = r#"
    * { box-sizing: border-box; }
    html { min-height: 100%; overflow-x: hidden; }
    body { min-height: 100%; margin: 0; font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; background: #f8fafc; color: #172033; overflow-x: hidden; }
    button, input { font: inherit; }
    button, a, .card { -webkit-tap-highlight-color: transparent; }
    .app { width: min(960px, 100%); margin: 0 auto; padding: max(16px, env(safe-area-inset-top)) 16px max(18px, env(safe-area-inset-bottom)); }
    .topbar { display: flex; align-items: center; justify-content: space-between; gap: 20px; margin-bottom: 16px; }
    h1 { margin: 0 0 4px; font-size: clamp(1.55rem, 6vw, 2.6rem); line-height: 1.05; }
    p { margin: 0; color: #64748b; }
    nav, .actions { display: flex; gap: 10px; flex-wrap: wrap; }
    button { border: 0; border-radius: 12px; padding: 10px 14px; min-height: 44px; background: #e2e8f0; color: #172033; cursor: pointer; }
    button:hover, button.active { background: #2563eb; color: white; }
    .controls, .page { background: white; border: 1px solid #e2e8f0; border-radius: 20px; box-shadow: 0 12px 30px rgba(15, 23, 42, 0.06); }
    .controls { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; padding: 14px; margin-bottom: 14px; }
    .controls input { width: 84px; margin-left: 8px; padding: 10px; min-height: 44px; border: 1px solid #cbd5e1; border-radius: 10px; }
    .selected-day { color: #64748b; }
    .page { padding: clamp(14px, 4vw, 22px); }
    .progress { text-align: center; color: #64748b; margin-bottom: 12px; }
    .card { display: flex; align-items: center; justify-content: center; width: 100%; min-height: min(52svh, 420px); padding: clamp(18px, 5vw, 34px); border: 2px dashed #cbd5e1; border-radius: 12px; background: #f8fafc; color: inherit; text-align: center; touch-action: manipulation; cursor: pointer; }
    .card:hover { background: #eff6ff; color: inherit; border-color: #2563eb; }
    .meta { color: #64748b; font-size: 0.95rem; margin-bottom: clamp(16px, 4vw, 24px); }
    .front { font-size: clamp(1.55rem, 8vw, 4rem); font-weight: 800; line-height: 1.12; overflow-wrap: anywhere; }
    .hint { margin-top: clamp(16px, 4vw, 22px); color: #94a3b8; font-size: 0.95rem; }
    .answer-block { display: flex; align-items: center; justify-content: center; flex-direction: column; gap: 12px; }
    .back { margin: 26px auto 0; width: fit-content; max-width: 100%; padding: 14px 18px; border-radius: 14px; background: #dcfce7; color: #166534; font-size: clamp(1.1rem, 5vw, 1.4rem); font-weight: 700; overflow-wrap: anywhere; }
    .translate-link { display: inline-block; border-radius: 999px; padding: 8px 12px; background: #dbeafe; color: #1d4ed8; font-size: 0.95rem; font-weight: 700; text-decoration: none; }
    .translate-link:hover { background: #2563eb; color: white; }
    .translate-link.small { padding: 4px 8px; font-size: 0.8rem; }
    .actions { justify-content: center; margin-top: 14px; }
    .list-page h2 { margin-top: 0; }
    .day-block { padding: 16px 0; border-top: 1px solid #e2e8f0; }
    .day-block:first-of-type { border-top: 0; }
    .day-block h3 { margin: 0 0 12px; }
    .day-block h4 { margin: 12px 0 6px; color: #475569; }
    li { margin: 6px 0; overflow-wrap: anywhere; }

    body:has(.app.dark) { background: #000; }
    .app.dark { color: #e5e7eb; background: #000; min-height: 100vh; border-radius: 0; padding-left: 16px; padding-right: 16px; }
    .app.dark p,
    .app.dark .progress,
    .app.dark .meta,
    .app.dark .selected-day { color: #94a3b8; }
    .app.dark button { background: #334155; color: #e5e7eb; }
    .app.dark button:hover,
    .app.dark button.active { background: #60a5fa; color: #0f172a; }
    .app.dark input { background: #020617; border-color: #475569; color: #e5e7eb; }
    .app.dark .controls,
    .app.dark .page { background: #111827; border-color: #334155; box-shadow: 0 12px 30px rgba(0, 0, 0, 0.35); }
    .app.dark .card { background: #020617; border-color: #475569; color: #e5e7eb; }
    .app.dark .card:hover { background: #172554; border-color: #60a5fa; color: #e5e7eb; }
    .app.dark .hint { color: #64748b; }
    .app.dark .back { background: #14532d; color: #dcfce7; }
    .app.dark .translate-link { background: #1e3a8a; color: #dbeafe; }
    .app.dark .translate-link:hover { background: #60a5fa; color: #0f172a; }
    .app.dark .day-block { border-top-color: #334155; }
    .app.dark .day-block h4 { color: #cbd5e1; }

    @media (max-width: 640px) {
        .app { width: 100%; padding-left: 10px; padding-right: 10px; }
        .topbar { align-items: stretch; flex-direction: column; gap: 12px; text-align: center; }
        nav { width: 100%; display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
        nav button { width: 100%; padding-left: 8px; padding-right: 8px; }
        nav button:last-child { grid-column: 1 / -1; }
        .controls { align-items: stretch; gap: 10px; padding: 12px; }
        .controls label { display: grid; grid-template-columns: auto minmax(72px, 1fr) auto; align-items: center; gap: 8px; width: 100%; }
        .controls input { width: 100%; min-width: 0; margin-left: 0; }
        .controls button { width: 100%; }
        .selected-day { width: 100%; text-align: center; font-size: 0.95rem; }
        .page { padding: 12px; border-radius: 16px; }
        .card { min-height: 44svh; padding: 18px 12px; }
        .meta { font-size: 0.85rem; }
        .hint { font-size: 0.85rem; }
        .answer-block { gap: 10px; }
        .back { margin-top: 18px; padding: 12px 14px; }
        .actions { display: grid; grid-template-columns: 1fr; gap: 8px; }
        .actions button { width: 100%; }
        .list-page h2 { font-size: 1.25rem; }
        .day-block h3 { font-size: 1.05rem; }
        .day-block ul { padding-left: 18px; }
        .translate-link.small { display: inline-block; margin-top: 4px; }
    }

    @media (max-width: 380px) {
        .controls label { grid-template-columns: 1fr; text-align: center; }
        .front { font-size: clamp(1.35rem, 9vw, 2rem); }
        .card { min-height: 42svh; }
    }
"#;
