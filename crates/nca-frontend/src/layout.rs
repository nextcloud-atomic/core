#![allow(non_snake_case)]

use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;
use daisy_rsx::*;
use dioxus::document::EvalError;
use dioxus::prelude::*;
use dioxus_logger::tracing;
use crate::assets;
use reqwest;
use nca_system_api::types;
use web_sys::{window, Document};
// use web_sys::wasm_bindgen::JsValue;
// use web_sys::wasm_bindgen::prelude::wasm_bindgen;
// use web_assets::files::index_js;
// use nca_system_api::types::ServiceStatus;


// fn document() -> Document {
//     window().expect("no global `window` exists")
//         .document().expect("should have a document")
// }
//
// #[wasm_bindgen(start)]
// pub fn run() -> Result<(), JsValue> {
//     let doc = document();
//
// }


// static DROPIN_JS: Asset = asset!("assets/js/dropin.js");
// static TAILWIND_CSS: Asset = asset!("assets/css/tailwind.css");
#[derive(PartialEq, Clone, Eq, Debug)]
pub enum SideBar {
    Activation,
    None
}

impl std::fmt::Display for SideBar {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct NcaLayoutProps {
    title: String,
    headline: Option<Element>,
    children: Element,
    enable_sidebar: bool,
    selected_item: SideBar,
    breadcrumbs: Vec<String>,
}

#[component]
pub fn Layout(props: NcaLayoutProps) -> Element {

    rsx! {
        BaseLayout {
            title: props.title.clone(),
            stylesheets: vec![],
            header: rsx!(
                nav {
                    aria_label: "breadcrumb",
                    ol {
                        class: "flex flex-wrap items-center gap-1.5 break-words text-sm sm:gap-2.5",
                        li {
                            class: "ml-3 items-center gap-1.5 hidden md:block",
                            {props.headline.unwrap_or(rsx!("{props.title}"))}
                        }
                        for breadcrumb in &props.breadcrumbs {
                            li {
                                ">"
                            }
                            li {
                                "{breadcrumb}"
                            }
                        }
                    }
                }
            ),
            enable_sidebar: props.enable_sidebar,
            sidebar: rsx!(
                NavGroup {
                    heading: "Your Menu",
                    content:  rsx!(
                        NavItem {
                            id: SideBar::Activation.to_string(),
                            selected_item_id: props.selected_item.to_string(),
                            icon: "",
                            href: "/",
                            title: "Activation"
                        }
                    )
                }
            ),
            sidebar_header: rsx!(
                div {
                    class: "flex aspect-square size-8 items-center justify-center rounded-lg bg-neutral text-neutral-content",
                    svg {
                        xmlns: "http://www.w3.org/2000/svg",
                        width: "24",
                        height: "24",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        class: "lucide lucide-gallery-vertical-end size-4",
                        path {
                            d: "M7 2h10",
                        }
                        path {
                            d: "M5 6h14",
                        }
                        rect {
                            width: "18",
                            height: "12",
                            x: "3",
                            y: "10",
                            rx: "2",
                        }
                    }
                }
                div {
                    class: "ml-3 flex flex-col gap-0.5 leading-none",
                    span {
                        class: "font-semibold uppercase",
                        "Nextcloud Atomic"
                    }
                    span {
                        class: "",
                        "v1.0.1"
                    }
                }
            ),
            sidebar_footer: rsx!(),
            js_href: None,
            children: props.children,
        }
    }
}


#[derive(Props, Clone, PartialEq)]
pub struct BaseLayoutProps {
    title: String,
    fav_icon_src: Option<String>,
    stylesheets: Vec<Asset>,
    js_href: Option<Asset>,
    header: Element,
    children: Element,
    sidebar: Element,
    sidebar_footer: Element,
    sidebar_header: Element,
    enable_sidebar: bool,
}

#[component]
pub fn BaseLayout(props: BaseLayoutProps) -> Element {
    rsx!(
        head {
            title {
                "{props.title}"
            }
            meta {
                charset: "utf-8"
            }
            meta {
                "http-equiv": "X-UA-Compatible",
                content: "IE=edge"
            }
            meta {
                name: "viewport",
                content: "width=device-width, initial-scale=1"
            }
            for href in &props.stylesheets {
                link {
                    rel: "stylesheet",
                    href: "{href}",
                    "type": "text/css"
                }
            }
            if let Some(js_href) = props.js_href {
                script {
                    "type": "module",
                    src: "{js_href}"
                }
            }
            if let Some(fav_icon_src) = props.fav_icon_src {
                link {
                    rel: "icon",
                    "type": "image/svg+xml",
                    href: "{fav_icon_src}"
                }
            }
        }
        body {
            div {
                class: "flex h-screen overflow-hidden",
                if props.enable_sidebar {
                    nav {
                        id: "sidebar",
                        class: "
                            border-r border-base-300
                            fixed
                            bg-base-200
                            inset-y-0
                            left-0
                            w-64
                            transform
                            -translate-x-full
                            transition-transform
                            duration-200
                            ease-in-out
                            flex
                            flex-col
                            lg:translate-x-0
                            lg:static
                            lg:inset-auto
                            lg:transform-none
                            z-20",
                        div {
                            class: "flex items-center p-4",
                            {props.sidebar_header}
                        }
                        div {
                            class: "flex-1 overflow-y-auto",
                            {props.sidebar}
                        }
                        div {
                            class: "p-4",
                            {props.sidebar_footer}
                        }
                    }
                }
                main {
                    id: "main-content",
                    class: "flex-1 flex flex-col",
                    header {
                        class: "flex items-center p-4 border-b border-base-300",
                        if props.enable_sidebar {
                            button {
                                id: "toggleButton",
                                svg {
                                    xmlns: "http://www.w3.org/2000/svg",
                                    width: "24",
                                    height: "24",
                                    view_box: "0 0 24 24",
                                    fill: "none",
                                    stroke: "currentColor",
                                    stroke_width: "2",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    class: "lucide lucide-panel-left",
                                    rect {
                                        width: "18",
                                        height: "18",
                                        x: "3",
                                        y: "3",
                                        rx: "2",
                                    }
                                    path {
                                        d: "M9 3v18",
                                    }
                                }
                            }
                        }
                        {props.header}
                    }
                    section {
                        class: "flex flex-col flex-1 overflow-y-auto",
                        {props.children}
                    }
                }
            }
        }
    )
}
