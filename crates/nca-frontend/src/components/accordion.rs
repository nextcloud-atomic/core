use dioxus::prelude::*;

#[derive(Clone, Debug, Props, PartialEq)]
pub struct AccordionProps {
    name: String,
    #[props(default = String::default())]
    class: String,
    title: Element,
    is_active: bool,
    children: Element
}

#[component]
pub fn Accordion(props: AccordionProps) -> Element {

    rsx!{
        li {
            class: props.class + " collapse bg-base-100 border border-base-300 border",
            input {
                r#type: "radio",
                name: props.name,
                checked: props.is_active,
            },
            div {
                class: "collapse-title font-semibold flex flex-row justify-between",
                {{ props.title }}
            },
            div {
                class: "collapse-content text-sm",
                {{ props.children }}
            }
        }
    }
}

// #[component]
// pub fn AccordionSteps(props: Vec<AccordionProps>) -> Element {
//     
//     let child_props = use_memo(move || {
//         props.iter().enumerate().map(|(idx, prop)| {
//             let new_title = rsx!(format!("{}", idx), prop.title);
//             AccordionProps{
//                 title: new_title,
//                 is_active: prop.is_active,
//                 class: prop.class.clone(),
//                 children: prop.children.clone(),
//                 name: prop.name.clone()
//             }
//         })
//     });
//     
//     rsx!{
//         for prop in child_props {
//             Accordion(child_props)
//         }
//     }
// }