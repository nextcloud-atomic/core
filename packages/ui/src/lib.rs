mod layout;
pub mod disks;
use dioxus::prelude::*;

// Method to render components to HTML
pub fn render(mut virtual_dom: VirtualDom) -> String {
    virtual_dom.rebuild_in_place();
    let html = dioxus_ssr::render(&virtual_dom);
    format!("<!DOCTYPE html><html lang='en'>{}</html>", html)
}

