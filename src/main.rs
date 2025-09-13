use dioxus::prelude::*;

mod app;
mod component;

fn main() {
    launch(component::App);
}
