extern crate midir;
extern crate console_error_panic_hook;
extern crate js_sys;
extern crate web_sys;
extern crate wasm_bindgen;

use js_sys::{Array};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{console};

use std::sync::{Arc, Mutex};
use std::error::Error;

use midir::{MidiInput, Ignore};

pub fn log(s: String) {
    console::log(&Array::of1(&s.into()));
}

macro_rules! println {
    ()              => (log("".to_owned()));
    ($($arg:tt)*)   => (log(format!($($arg)*)));
}

#[wasm_bindgen(start)]
pub fn start() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let token_outer = Arc::new(Mutex::new(None));
    let token = token_outer.clone();
    let closure : Closure<dyn FnMut()> = Closure::wrap(Box::new(move ||{
        if run().unwrap() == true {
            if let Some(token) = *token.lock().unwrap() {
                web_sys::window().unwrap().clear_interval_with_handle(token);
            }
        }
    }));
    *token_outer.lock().unwrap() = web_sys::window().unwrap().set_interval_with_callback_and_timeout_and_arguments_0(
        closure.as_ref().unchecked_ref(),
        200,
    ).ok();
    closure.forget();
}

fn run() -> Result<bool, Box<dyn Error>> {
    let mut midi_in = MidiInput::new("midir reading input")?;
    midi_in.ignore(Ignore::None);

    // Get an input port (read from console if multiple are available)
    let in_port = match midi_in.port_count() {
        0 => return Ok(false),
        1 => {
            println!("Choosing the only available input port: {}", midi_in.port_name(0).unwrap());
            0
        },
        _ => {
            println!("\nAvailable input ports:");
            for i in 0..midi_in.port_count() {
                println!("{}: {}", i, midi_in.port_name(i).unwrap());
            }
            println!("Using the first input port");
            0
        }
    };

    println!("\nOpening connection");
    let in_port_name = midi_in.port_name(in_port)?;

    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _conn_in = midi_in.connect(in_port, "midir-read-input", move |stamp, message, _| {
        println!("{}: {:?} (len = {})", stamp, message, message.len());
    }, ())?;

    println!("Connection open, reading input from '{}'", in_port_name);
    Box::leak(Box::new(_conn_in));
    Ok(true)
}
