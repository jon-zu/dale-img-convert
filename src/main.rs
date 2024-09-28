#![allow(non_snake_case)]

pub mod img_conv;

use std::rc::Rc;

use base64::{prelude::BASE64_STANDARD, Engine};
use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};
use image::DynamicImage;
use img_conv::DalImageConverter;

pub struct ImageResult {
    pub img: DynamicImage,
    pub name: String,
    pub base64: String,
}

impl ImageResult {
    pub fn new(img: DynamicImage, name: String) -> Self {
        let mut buf = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buf, image::ImageFormat::Png).unwrap();
        let enc = BASE64_STANDARD.encode(buf.get_ref());
        let base64 = format!("data:image/png;base64,{enc}");
        Self { img, name, base64 }
    }
}

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
}

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    info!("starting app");
    launch(App);
}

fn App() -> Element {
    rsx! {
        div {
            class: "container",
            Router::<Route> {}
        }
    }
}

fn convert(conv: &DalImageConverter, auto_rotate: bool, buf: &[u8], name: String) -> anyhow::Result<Rc<ImageResult>> {
    let img = image::load_from_memory(buf)?;
    let img = conv.process(img, auto_rotate);
    Ok(Rc::new(ImageResult::new(img, name)))
}

#[component]
fn file_picker(mut images: Signal<Vec<Rc<ImageResult>>>) -> Element {
    let mut auto_rotate = use_signal(|| true);
    let conv = use_signal(|| DalImageConverter::default());
    rsx! {
        form {
            div {
                class: "form-check form-switch mb-3",
                label {
                    class: "form-check-label",
                    "Auto Rotate"
                }
                input {
                    class: "form-check-input",
                    role: "switch",
                    r#type: "checkbox",
                    checked: auto_rotate,
                    onchange: move |evt| {
                        auto_rotate.set(evt.checked());
                    }
                }
            }
            div {
                class: "mb-3",
                input {
                    r#type: "file",
                    class: "form-control",
                    accept: ".png,.jpg,.jpeg,.webp",
                    multiple: true,
                    onchange: move |evt| {
                        async move {
                            if let Some(file_engine) = &evt.files() {
                                let files = file_engine.files();
                                for file_name in files {
                                    // Read the data
                                    let Some(data) = file_engine.read_file(&file_name).await else {
                                        dioxus_logger::tracing::error!("Failed to read file: {}", file_name);
                                        continue;
                                    };

                                    // Convert the data
                                    match convert(&conv.read(), *auto_rotate.read(), &data, file_name.clone()) {
                                        Ok(img) => {
                                            dioxus_logger::tracing::info!("Image loaded: {} {}", img.img.height(), img.img.width());
                                            images.push(img);
                                        }
                                        Err(e) => {
                                            dioxus_logger::tracing::error!("Failed to load image: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div {
                class: "mb-3",
                input {
                    class: "button",
                    r#type: "button",
                    value: "Clear",
                    onclick: move |_| {
                        images.set(vec![]);
                    }
                }
            }
        }
    }
}

#[component]
fn Home() -> Element {
    let images = use_signal(|| vec![]);

    rsx! {
        div {
            h1 { "Dale & Dawson Image Converter" }
            file_picker { images }

            div {
                class: "row row-cols-1 row-cols-md-3 g-4",
                for img in images.iter() {

                    div {
                        class: "col",
                        div {
                            class: "card",
                            img {
                                class: "card-img-top",
                                r#src: "{img.base64}",
                                r#alt: "{img.name}",
                            }
                            p {
                                class: "card-text",
                                "{img.name}"
                            }
                            a {
                                href: "{img.base64}",
                                download: "image.png", // Specify the default filename
                                button {
                                    class: "btn btn-primary",
                                    "Download"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
