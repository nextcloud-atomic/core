pub mod layout;
pub mod assets;
pub mod components;

pub use crate::components::*;

use web_sys::window;


pub fn base_url() -> String {
    window().unwrap().location().origin().unwrap()
}