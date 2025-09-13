use dioxus::prelude::Element;

pub trait View {
    fn view(&self) -> Element;
}
