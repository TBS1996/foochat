#![allow(non_snake_case)]

use crate::common::Scores;
use chat::Chat;
use dioxus::prelude::*;
use futures::executor::block_on;
use once_cell::sync::Lazy;
use std::ops::Deref;
use std::str::FromStr;
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

    fn clear_peer(&self) {
        let mut lock = self.inner.lock().unwrap();
        if let Some(socket) = &lock.socket {
            socket.close().unwrap();
        }
        lock.peer_scores = None;
        lock.socket = None;
    }
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
fn log_to_console(message: impl std::fmt::Debug) {
    let message = format!("{:?}", message);
    console::log_1(&JsValue::from_str(&message));
}

#[component]
pub fn Invalid() -> Element {
    rsx! {
        "invalid input! all values must be between 0 and 100",
        Link { to: Route::Home {}, "try again" }
    }
}

fn default_scores() -> Scores {
    static COOKIE: Lazy<Option<Scores>> = Lazy::new(|| {
        let scores = block_on(fetch_scores_cookie());
        scores
    });

    COOKIE.unwrap_or_else(Scores::mid)
}

async fn fetch_scores_cookie() -> Option<Scores> {
    let mut eval = eval(
        r#"
        let value = "; " + document.cookie;
        let parts = value.split("; scores=");
        if (parts.length == 2) {
            let scores = parts.pop().split(";").shift();
            dioxus.send(scores);
        } else {
            dioxus.send(null);
        }
        "#,
    );

    let cookies = eval.recv().await.unwrap().to_string();
    log_to_console(&cookies);
    Scores::from_str(&cookies).ok()
}

#[component]
fn Home() -> Element {
    let navigator = use_navigator();
    let state = use_context::<State>();
    let score = default_scores();

    rsx! {
    form { onsubmit:  move |event| {
         match Scores::try_from(event.data().deref()) {
             Ok(scores) => {
                 state.set_scores(scores);
                 save_scores(scores);
                 navigator.replace(Route::Chat{});
             }
             Err(_) => {
                 navigator.replace(Route::Invalid {});
             }

         }

    },
    div { class: "form-group",
                label { "Openness: " }
                input { name: "o", value: "{score.o}"}
                }
                div { class: "form-group",
                    label { "Conscientiousness: " }
                    input { name: "c" , value: "{score.c}"}
                }
                div { class: "form-group",
                    label { "Extraversion: " }
                    input { name: "e", value: "{score.e}"}
                }
                div { class: "form-group",
                    label { "Agreeableness: " }
                    input { name: "a" , value: "{score.a}"}
                }
                div { class: "form-group",
                    label { "Neuroticism: " }
                    input { name: "n", value: "{score.n}"}
                }
                div { class: "form-group",
                    input { r#type: "submit", value: "Submit" }
            }
        }
    }
}

fn save_scores(scores: Scores) {
    let script = format!(
        "document.cookie = 'scores={}; expires=Fri, 31 Dec 9999 23:59:59 GMT; path=/';",
        scores
    );

    eval(&script);
    log_to_console("storing scores cookie");
}
