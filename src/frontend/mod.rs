#![allow(non_snake_case)]

use crate::common::Scores;
use chat::Chat;
use dioxus::prelude::*;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use web_sys::console;
use web_sys::WebSocket;

mod chat;

#[wasm_bindgen(start)]
pub fn run_app() {
    launch(App);
}

#[derive(Clone, Default)]
struct State {
    inner: Arc<Mutex<InnerState>>,
}

#[derive(Default)]
struct InnerState {
    scores: Option<Scores>,
    peer_scores: Option<Scores>,
    socket: Option<WebSocket>,
}

impl State {
    fn set_scores(&self, scores: Scores) {
        self.inner.lock().unwrap().scores = Some(scores);
    }

    fn set_peer_scores(&self, scores: Scores) {
        self.inner.lock().unwrap().peer_scores = Some(scores);
    }

    fn scores(&self) -> Option<Scores> {
        self.inner.lock().unwrap().scores
    }

    fn set_socket(&self, socket: WebSocket) {
        self.inner.lock().unwrap().socket = Some(socket);
    }

    fn send_message(&self, msg: &str) -> bool {
        if let Some(socket) = &self.inner.lock().unwrap().socket {
            let _ = socket.send_with_str(msg);
            true
        } else {
            log_to_console("attempted to send msg without a socket configured");
            false
        }
    }
}

fn scores_from_formdata(form: &FormData) -> Option<Scores> {
    let data = form.values();

    let o: f32 = data.get("o")?.as_value().parse().ok()?;
    let c: f32 = data.get("c")?.as_value().parse().ok()?;
    let e: f32 = data.get("e")?.as_value().parse().ok()?;
    let a: f32 = data.get("a")?.as_value().parse().ok()?;
    let n: f32 = data.get("n")?.as_value().parse().ok()?;

    if !(0. ..=100.).contains(&o) {
        return None;
    }
    if !(0. ..=100.).contains(&c) {
        return None;
    }
    if !(0. ..=100.).contains(&e) {
        return None;
    }
    if !(0. ..=100.).contains(&a) {
        return None;
    }
    if !(0. ..=100.).contains(&n) {
        return None;
    }

    Some(Scores { o, c, e, a, n })
}

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/invalid")]
    Invalid {},
    #[route("/chat")]
    Chat {},
}

fn App() -> Element {
    use_context_provider(State::default);
    rsx!(Router::<Route> {})
}

// Call this function to log a message
fn log_to_console(message: &str) {
    console::log_1(&JsValue::from_str(message));
}

#[component]
pub fn Invalid() -> Element {
    rsx! {
        "invalid input! all values must be between 0 and 100",
        Link { to: Route::Home {}, "try again" }
    }
}

#[component]
fn Home() -> Element {
    let navigator = use_navigator();
    let state = use_context::<State>();

    rsx! {
    form { onsubmit:  move |event| {
         match scores_from_formdata(&event.data()) {
             Some(scores) => {
                 state.set_scores(scores);
                 navigator.replace(Route::Chat{});
             }
             None => {
                 navigator.replace(Route::Invalid {});
             }

         }

    },
    div { class: "form-group",
                label { "Openness: " }
                input { name: "o", value: "50"}
                }
                div { class: "form-group",
                    label { "Conscientiousness: " }
                    input { name: "c" , value: "50"}
                }
                div { class: "form-group",
                    label { "Extraversion: " }
                    input { name: "e", value: "50"}
                }
                div { class: "form-group",
                    label { "Agreeableness: " }
                    input { name: "a" , value: "50"}
                }
                div { class: "form-group",
                    label { "Neuroticism: " }
                    input { name: "n", value: "50"}
                }
                div { class: "form-group",
                    input { r#type: "submit", value: "Submit" }
            }
        }
    }
}
