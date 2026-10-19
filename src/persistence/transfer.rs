use std::sync::mpsc::Sender;

pub enum TransferEvent {
    Imported(Result<String, String>),
    Exported(Result<(), String>),
}

#[cfg(not(target_arch = "wasm32"))]
pub fn import(sender: Sender<TransferEvent>, ctx: egui::Context) {
    std::thread::spawn(move || {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("TMTinker project", &["json"])
            .pick_file()
        {
            let result = std::fs::read_to_string(path).map_err(|e| e.to_string());
            drop(sender.send(TransferEvent::Imported(result)));
            ctx.request_repaint();
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub fn export(name: &str, json: String, sender: Sender<TransferEvent>, ctx: egui::Context) {
    let filename = format!(
        "{}.json",
        name.chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            })
            .collect::<String>()
    );
    std::thread::spawn(move || {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("TMTinker project", &["json"])
            .set_file_name(filename)
            .save_file()
        {
            let result = std::fs::write(path, json).map_err(|e| e.to_string());
            drop(sender.send(TransferEvent::Exported(result)));
            ctx.request_repaint();
        }
    });
}

#[cfg(target_arch = "wasm32")]
pub fn import(sender: Sender<TransferEvent>, ctx: egui::Context) {
    use wasm_bindgen::{JsCast, closure::Closure};
    let error_sender = sender.clone();
    let error_ctx = ctx.clone();
    let result = (|| -> Result<(), String> {
        let document = web_sys::window()
            .and_then(|w| w.document())
            .ok_or("Browser document unavailable")?;
        let input = document
            .create_element("input")
            .map_err(|e| format!("{e:?}"))?
            .dyn_into::<web_sys::HtmlInputElement>()
            .map_err(|_| "File input unavailable")?;
        input.set_type("file");
        input.set_accept(".json,application/json");
        let reader = input.clone();
        let callback = Closure::<dyn FnMut(web_sys::Event)>::once(move |_| {
            let Some(file) = reader.files().and_then(|files| files.get(0)) else {
                return;
            };
            wasm_bindgen_futures::spawn_local(async move {
                let result = wasm_bindgen_futures::JsFuture::from(file.text())
                    .await
                    .map_err(|e| format!("{e:?}"))
                    .and_then(|value| {
                        value
                            .as_string()
                            .ok_or_else(|| "Cannot read file text".into())
                    });
                drop(sender.send(TransferEvent::Imported(result)));
                ctx.request_repaint();
            });
        });
        input.set_onchange(Some(callback.as_ref().unchecked_ref()));
        callback.forget();
        input.click();
        Ok(())
    })();
    if let Err(error) = result {
        drop(error_sender.send(TransferEvent::Imported(Err(error))));
        error_ctx.request_repaint();
    }
}

#[cfg(target_arch = "wasm32")]
pub fn export(name: &str, json: String, sender: Sender<TransferEvent>, ctx: egui::Context) {
    use wasm_bindgen::JsCast;
    let result = (|| -> Result<(), String> {
        let document = web_sys::window()
            .and_then(|w| w.document())
            .ok_or("Browser document unavailable")?;
        let parts = js_sys::Array::new();
        parts.push(&wasm_bindgen::JsValue::from_str(&json));
        let blob = web_sys::Blob::new_with_str_sequence(&parts).map_err(|e| format!("{e:?}"))?;
        let url = web_sys::Url::create_object_url_with_blob(&blob).map_err(|e| format!("{e:?}"))?;
        let link = document
            .create_element("a")
            .map_err(|e| format!("{e:?}"))?
            .dyn_into::<web_sys::HtmlAnchorElement>()
            .map_err(|_| "Download unavailable")?;
        link.set_href(&url);
        link.set_download(&format!("{name}.json"));
        let body = document.body().ok_or("Browser document unavailable")?;
        body.append_child(&link).map_err(|e| format!("{e:?}"))?;
        link.click();
        link.remove();
        web_sys::Url::revoke_object_url(&url).map_err(|e| format!("{e:?}"))?;
        Ok(())
    })();
    drop(sender.send(TransferEvent::Exported(result)));
    ctx.request_repaint();
}
