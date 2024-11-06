#![allow(non_snake_case)]

use dioxus::prelude::*;

#[component]
pub fn Layout(title: String, children: Element) -> Element {
    rsx!(
        head {
            title {
                "{title}"
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
        }
        body {
            {children}
        }
    )
}
