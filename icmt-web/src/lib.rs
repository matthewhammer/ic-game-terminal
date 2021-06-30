use wasm_bindgen::prelude::*;

use std::f64;

use wasm_bindgen::JsCast;
use web_sys::{self, console};

use std::cell::Cell;
use std::rc::Rc;

extern crate icmt;
use icmt::{
    types::{
        event::{self, Event, KeyEventInfo}, graphics, nat_ceil, ServiceCall
    },
};

fn console_log(m: String) {
    let message: JsValue = m.as_str().clone().into();
    console::log_1(&message);
}

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub async fn main() -> Result<(), JsValue> {

    let closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {

        match format!("{}", event.key()).as_str() {
            "Tab" | "Escape" | "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight" | " "
                | "Backspace" => {
                    let ev =
                        Event::KeyDown(vec![KeyEventInfo {
                            key: event.key(),
                            alt: event.alt_key(),
                            ctrl: event.ctrl_key(),
                            shift: event.shift_key(),
                            meta: event.meta_key(),
                        }]);
                    drop(ev)
                }
            key => {
                console_log(format!("unrecognized key: {}", key));                
            }
        };

    }) as Box<dyn FnMut(_)>);

    let document = web_sys::window().unwrap().document().unwrap();
    document.set_onkeydown(Some(closure.as_ref().unchecked_ref()));
    //document.set_onkeypress(Some(closure.as_ref().unchecked_ref()));
    //document.set_onkeyup(Some(closure.as_ref().unchecked_ref()));
    //document.set_oninput(Some(closure.as_ref().unchecked_ref()));
    closure.forget();

    Ok(())
}
