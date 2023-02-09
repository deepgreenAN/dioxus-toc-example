#![allow(clippy::derive_partial_eq_without_eq)]
#![allow(non_snake_case)]

use gloo_utils::document;
use gloo_intersection::IntersectionObserverHandler;
use wasm_bindgen::JsCast;
use web_sys::{Element as WebSysElement, ScrollIntoViewOptions, ScrollBehavior};
use dioxus::{core::to_owned, prelude::*};
use std::rc::Rc;
use std::cell::{RefCell, Cell};
use indexmap::IndexMap;

fn TableObContents(cx: Scope) -> Element {
    let heading_elements = use_state(&cx, Vec::<WebSysElement>::new);
    let visible_map = cx.use_hook(|_|{Rc::new(RefCell::new(IndexMap::<String, bool>::new()))});
    let active_id = use_state(&cx, || Option::<String>::None);
    let intersection_observer_handler = cx.use_hook(|_|Rc::new(Cell::new(Option::<IntersectionObserverHandler>::None))); 

    use_effect(&cx, (), {
        to_owned![
            heading_elements,
            visible_map,
            active_id,
            intersection_observer_handler
        ];
        |_| async move {
            // headingに対応するElementのリスト
            let node_list = document().query_selector_all("h2, h3").unwrap();
            let elements = (0..node_list.length())
                .map(|i| {
                    let element:WebSysElement = node_list.item(i).unwrap().unchecked_into();
                    {
                        visible_map.borrow_mut().insert(element.id(), false);
                    }
                    element
                })
                .collect::<Vec<_>>();

            heading_elements.set(elements);


            // Intersection Observer のハンドラ
            let handler = IntersectionObserverHandler::new({
                move |entries, _|{
                    entries.into_iter().for_each(|entry|{
                        if let Some(is_visible) = visible_map.borrow_mut().get_mut(&entry.target().id()) {
                            *is_visible = entry.is_intersecting();
                        };
                    });
                    let visible_map = visible_map.borrow();
                    let (found_key, _found_value) = visible_map.iter().find(|(_id, is_visible)|{**is_visible}).unwrap();
                    active_id.set(Some(found_key.clone()));
                }
            }).unwrap();

            for i in 0..node_list.length() {
                handler.observe(&(node_list.item(i).unwrap().unchecked_into()));
            }

            intersection_observer_handler.set(Some(handler));
        }
    });

    cx.render(rsx! {
        div {
            nav{ aria_label:"Table of Contents", class:"toc",
                {
                    heading_elements.get().iter().enumerate().map(|(i,element)|{
                        let mut class = match element.tag_name().as_str() {
                            "H2" => {"h2-toc-item".to_string()},
                            "H3" => {"h3-toc-item".to_string()},
                            _ => {unreachable!()}
                        };
                        if let Some(active_id) = active_id.get() {
                            if element.id() == *active_id {
                                class.push_str(" active");
                            }
                        }

                        let mut scroll_options = ScrollIntoViewOptions::new();
                        scroll_options.behavior(ScrollBehavior::Smooth);

                        rsx! {
                            div {key:"{i}", class:"{class}", 
                                onclick: move |_|{element.scroll_into_view_with_scroll_into_view_options(&scroll_options)}, 
                                [element.inner_html()]
                            }
                        }
                    })
                }
            }
        }
    })
}

fn App(cx: Scope) -> Element {
    let contents_str = include_str!("../contents/contents.html");

    cx.render(rsx! {
        div { class:"container",
            div { class:"main-contents", dangerous_inner_html: "{contents_str}" }
            TableObContents{}
        }
    })
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    dioxus::web::launch(App);
}
