use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use gtk::prelude::*;
use gtk::{gdk, gio, glib};

use crate::config::{self, Config};
use crate::languages::{self, AUTO_CODE, LANGS};
use crate::ocr;
use crate::providers::{GoogleProvider, Provider};

type CfgRef = Rc<RefCell<Config>>;
type Detected = Rc<RefCell<Option<String>>>;

pub fn build_app(app: &gtk::Application) {
    let cfg: CfgRef = Rc::new(RefCell::new(config::load()));
    let detected: Detected = Rc::new(RefCell::new(None));
    let provider: Arc<dyn Provider> = Arc::new(GoogleProvider);

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("Translator")
        // Base size only — window stays resizable.
        .default_width(660)
        .default_height(200)
        .build();

    // ------------------------------------------------------------ header
    let header = gtk::HeaderBar::new();
    let src_btn = gtk::MenuButton::new();
    src_btn.set_label(&source_label(&cfg.borrow(), &detected.borrow()));
    let swap_btn = gtk::Button::from_icon_name("object-flip-horizontal-symbolic");
    swap_btn.set_tooltip_text(Some("Swap languages"));
    let dst_btn = gtk::MenuButton::new();
    dst_btn.set_label(languages::name_for(&cfg.borrow().target));
    let lang_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    lang_box.append(&src_btn);
    lang_box.append(&swap_btn);
    lang_box.append(&dst_btn);
    header.set_title_widget(Some(&lang_box));

    let image_btn = gtk::Button::from_icon_name("image-x-generic-symbolic");
    image_btn.set_tooltip_text(Some("Translate image file (OCR)"));
    header.pack_end(&image_btn);
    window.set_titlebar(Some(&header));

    // -------------------------------------------------------------- body
    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    hbox.set_margin_top(8);
    hbox.set_margin_bottom(8);
    hbox.set_margin_start(8);
    hbox.set_margin_end(8);

    let src_view = gtk::TextView::new();
    src_view.set_wrap_mode(gtk::WrapMode::WordChar);
    text_margins(&src_view);
    let src_scroll = gtk::ScrolledWindow::builder()
        .child(&src_view)
        .hexpand(true)
        .vexpand(true)
        .build();
    src_scroll.add_css_class("frame");

    let arrow = gtk::Label::new(None);
    arrow.set_markup("<span size='x-large'>→</span>");
    arrow.set_valign(gtk::Align::Center);
    arrow.set_margin_start(4);
    arrow.set_margin_end(4);

    let dst_view = gtk::TextView::new();
    dst_view.set_wrap_mode(gtk::WrapMode::WordChar);
    dst_view.set_editable(false);
    dst_view.set_cursor_visible(false);
    text_margins(&dst_view);
    let dst_scroll = gtk::ScrolledWindow::builder()
        .child(&dst_view)
        .hexpand(true)
        .vexpand(true)
        .build();
    dst_scroll.add_css_class("frame");

    hbox.append(&src_scroll);
    hbox.append(&arrow);
    hbox.append(&dst_scroll);
    window.set_child(Some(&hbox));

    // -------------------------------------------------------- translate
    let generation = Rc::new(Cell::new(0u64));

    let do_translate: Rc<dyn Fn()> = Rc::new({
        let cfg = cfg.clone();
        let detected = detected.clone();
        let provider = provider.clone();
        let src_view = src_view.clone();
        let dst_view = dst_view.clone();
        let src_btn = src_btn.clone();
        let generation = generation.clone();
        move || {
            let buf = src_view.buffer();
            let text = buf
                .text(&buf.start_iter(), &buf.end_iter(), false)
                .trim()
                .to_string();
            if text.is_empty() {
                dst_view.buffer().set_text("");
                return;
            }
            let (from, to) = {
                let c = cfg.borrow();
                (c.source.clone(), c.target.clone())
            };
            generation.set(generation.get() + 1);
            let gen = generation.get();
            let handle = gio::spawn_blocking({
                let provider = provider.clone();
                let from = from.clone();
                move || provider.translate(&text, &from, &to)
            });
            glib::spawn_future_local({
                let cfg = cfg.clone();
                let detected = detected.clone();
                let dst_view = dst_view.clone();
                let src_btn = src_btn.clone();
                let generation = generation.clone();
                async move {
                    let res = handle.await;
                    if generation.get() != gen {
                        return;
                    }
                    match res {
                        Ok(Ok(tr)) => {
                            dst_view.buffer().set_text(&tr.text);
                            if from == AUTO_CODE {
                                if let Some(d) = tr.detected {
                                    *detected.borrow_mut() = Some(d);
                                    src_btn.set_label(&source_label(
                                        &cfg.borrow(),
                                        &detected.borrow(),
                                    ));
                                }
                            }
                        }
                        Ok(Err(e)) => dst_view.buffer().set_text(&format!("⚠ {e}")),
                        Err(_) => dst_view.buffer().set_text("⚠ Internal error"),
                    }
                }
            });
        }
    });

    // Instant translation: debounce typing by 350 ms.
    let pending: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));
    src_view.buffer().connect_changed({
        let pending = pending.clone();
        let do_translate = do_translate.clone();
        move |_| {
            if let Some(id) = pending.borrow_mut().take() {
                id.remove();
            }
            let id = glib::timeout_add_local_once(Duration::from_millis(350), {
                let pending = pending.clone();
                let do_translate = do_translate.clone();
                move || {
                    *pending.borrow_mut() = None;
                    do_translate();
                }
            });
            *pending.borrow_mut() = Some(id);
        }
    });

    // -------------------------------------------------- language pickers
    {
        let on_pick: Rc<dyn Fn(&str)> = Rc::new({
            let cfg = cfg.clone();
            let detected = detected.clone();
            let src_btn = src_btn.clone();
            let do_translate = do_translate.clone();
            move |code: &str| {
                {
                    let mut c = cfg.borrow_mut();
                    c.source = code.to_string();
                    config::save(&c);
                }
                src_btn.set_label(&source_label(&cfg.borrow(), &detected.borrow()));
                do_translate();
            }
        });
        let pop = build_lang_popover(cfg.clone(), true, on_pick);
        src_btn.set_popover(Some(&pop));
    }
    {
        let on_pick: Rc<dyn Fn(&str)> = Rc::new({
            let cfg = cfg.clone();
            let dst_btn = dst_btn.clone();
            let do_translate = do_translate.clone();
            move |code: &str| {
                {
                    let mut c = cfg.borrow_mut();
                    c.target = code.to_string();
                    config::save(&c);
                }
                dst_btn.set_label(languages::name_for(code));
                do_translate();
            }
        });
        let pop = build_lang_popover(cfg.clone(), false, on_pick);
        dst_btn.set_popover(Some(&pop));
    }

    let do_swap: Rc<dyn Fn()> = Rc::new({
        let cfg = cfg.clone();
        let detected = detected.clone();
        let src_btn = src_btn.clone();
        let dst_btn = dst_btn.clone();
        let src_view = src_view.clone();
        let dst_view = dst_view.clone();
        let do_translate = do_translate.clone();
        move || {
            {
                let mut c = cfg.borrow_mut();
                let old_src = c.source.clone();
                let old_dst = c.target.clone();
                c.source = old_dst;
                c.target = if old_src == AUTO_CODE {
                    detected
                        .borrow()
                        .clone()
                        .unwrap_or_else(|| "en".to_string())
                } else {
                    old_src
                };
                config::save(&c);
            }
            {
                let c = cfg.borrow();
                src_btn.set_label(languages::name_for(&c.source));
                dst_btn.set_label(languages::name_for(&c.target));
            }
            let dbuf = dst_view.buffer();
            let translated = dbuf
                .text(&dbuf.start_iter(), &dbuf.end_iter(), false)
                .to_string();
            if !translated.trim().is_empty() {
                src_view.buffer().set_text(&translated);
            }
            do_translate();
        }
    });
    swap_btn.connect_clicked({
        let do_swap = do_swap.clone();
        move |_| do_swap()
    });

    // -------------------------------------------------------------- OCR
    let run_ocr: Rc<dyn Fn(PathBuf)> = Rc::new({
        let cfg = cfg.clone();
        let src_view = src_view.clone();
        let dst_view = dst_view.clone();
        move |path: PathBuf| {
            let hint = {
                let c = cfg.borrow();
                if c.source == AUTO_CODE {
                    None
                } else {
                    languages::tesseract_code(&c.source).map(str::to_string)
                }
            };
            dst_view.buffer().set_text("Recognizing text…");
            let handle = gio::spawn_blocking(move || ocr::ocr_image(&path, hint.as_deref()));
            glib::spawn_future_local({
                let src_view = src_view.clone();
                let dst_view = dst_view.clone();
                async move {
                    match handle.await {
                        // Buffer change triggers the debounced translation.
                        Ok(Ok(text)) => src_view.buffer().set_text(&text),
                        Ok(Err(e)) => dst_view.buffer().set_text(&format!("⚠ {e}")),
                        Err(_) => dst_view.buffer().set_text("⚠ Internal error"),
                    }
                }
            });
        }
    });

    image_btn.connect_clicked({
        let window = window.clone();
        let run_ocr = run_ocr.clone();
        move |_| {
            let filter = gtk::FileFilter::new();
            filter.set_name(Some("Images"));
            filter.add_mime_type("image/*");
            let filters = gio::ListStore::new::<gtk::FileFilter>();
            filters.append(&filter);
            let dialog = gtk::FileDialog::builder()
                .title("Choose image to translate")
                .filters(&filters)
                .build();
            dialog.open(Some(&window), gio::Cancellable::NONE, {
                let run_ocr = run_ocr.clone();
                move |res| {
                    if let Ok(file) = res {
                        if let Some(path) = file.path() {
                            run_ocr(path);
                        }
                    }
                }
            });
        }
    });

    // --------------------------------------------------------------- keys
    // Capture phase: intercept before the TextView handles the event.
    let key_ctl = gtk::EventControllerKey::new();
    key_ctl.set_propagation_phase(gtk::PropagationPhase::Capture);
    key_ctl.connect_key_pressed({
        let window = window.clone();
        let dst_view = dst_view.clone();
        let src_btn = src_btn.clone();
        let dst_btn = dst_btn.clone();
        let run_ocr = run_ocr.clone();
        let do_swap = do_swap.clone();
        move |_, key, _, modifiers| {
            // A language popover is open — let it handle keys itself.
            if src_btn.is_active() || dst_btn.is_active() {
                return glib::Propagation::Proceed;
            }
            if key == gdk::Key::Escape {
                window.close();
                return glib::Propagation::Stop;
            }
            let ctrl = modifiers.contains(gdk::ModifierType::CONTROL_MASK);
            let shift = modifiers.contains(gdk::ModifierType::SHIFT_MASK);
            // Ctrl+R — swap languages (Cyrillic_ka is the same key on RU layout).
            let lower = key.to_lower();
            if ctrl && (lower == gdk::Key::r || lower == gdk::Key::Cyrillic_ka) {
                do_swap();
                return glib::Propagation::Stop;
            }
            if (key == gdk::Key::Return || key == gdk::Key::KP_Enter) && !ctrl && !shift {
                let buf = dst_view.buffer();
                let text = buf
                    .text(&buf.start_iter(), &buf.end_iter(), false)
                    .to_string();
                copy_and_close(&window, text);
                return glib::Propagation::Stop;
            }
            // Ctrl+V with an image in the clipboard → OCR it.
            // Cyrillic_em is the same physical key on a RU layout.
            if ctrl && (lower == gdk::Key::v || lower == gdk::Key::Cyrillic_em) {
                let clipboard = window.clipboard();
                let formats = clipboard.formats();
                let has_image = [
                    "image/png",
                    "image/jpeg",
                    "image/bmp",
                    "image/webp",
                    "image/tiff",
                ]
                .iter()
                .any(|m| formats.contain_mime_type(m));
                if has_image {
                    clipboard.read_texture_async(gio::Cancellable::NONE, {
                        let run_ocr = run_ocr.clone();
                        let dst_view = dst_view.clone();
                        move |res| match res {
                            Ok(Some(texture)) => {
                                let bytes = texture.save_to_png_bytes();
                                let path = std::env::temp_dir().join("translator-clipboard.png");
                                match std::fs::write(&path, &bytes) {
                                    Ok(()) => run_ocr(path),
                                    Err(e) => dst_view
                                        .buffer()
                                        .set_text(&format!("⚠ Failed to save image: {e}")),
                                }
                            }
                            _ => dst_view.buffer().set_text("⚠ Clipboard has no image"),
                        }
                    });
                    return glib::Propagation::Stop;
                }
                // No image → fall through to the normal text paste.
            }
            glib::Propagation::Proceed
        }
    });
    window.add_controller(key_ctl);

    window.present();
    src_view.grab_focus();
}

/// Label for the source button: the language name, or "Auto (Detected)"
/// once auto-detect has resolved a language.
fn source_label(cfg: &Config, detected: &Option<String>) -> String {
    if cfg.source == AUTO_CODE {
        match detected {
            Some(d) => format!("Auto ({})", languages::name_for(d)),
            None => "Auto detect".to_string(),
        }
    } else {
        languages::name_for(&cfg.source).to_string()
    }
}

/// Copy `text` to the clipboard and close the window. On Wayland the
/// clipboard dies with the client, so prefer a detached wl-copy process.
fn copy_and_close(window: &gtk::ApplicationWindow, text: String) {
    let text = text.trim().to_string();
    if text.is_empty() || text.starts_with('⚠') {
        window.close();
        return;
    }
    let wl_copy = std::process::Command::new("wl-copy").arg(&text).spawn();
    if wl_copy.is_ok() {
        window.close();
    } else {
        window.clipboard().set_text(&text);
        let w = window.clone();
        glib::timeout_add_local_once(Duration::from_millis(300), move || w.close());
    }
}

fn text_margins(view: &gtk::TextView) {
    view.set_top_margin(8);
    view.set_bottom_margin(8);
    view.set_left_margin(8);
    view.set_right_margin(8);
}

/// Build a language-picker popover: a searchable list with favorite stars.
/// It autohides once a language is selected. `on_pick` receives the chosen code.
fn build_lang_popover(cfg: CfgRef, include_auto: bool, on_pick: Rc<dyn Fn(&str)>) -> gtk::Popover {
    let popover = gtk::Popover::new();
    popover.set_size_request(300, 420);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 6);
    let search = gtk::SearchEntry::new();
    search.set_placeholder_text(Some("Search language…"));
    vbox.append(&search);

    let list = gtk::ListBox::new();
    list.set_selection_mode(gtk::SelectionMode::None);
    let scroll = gtk::ScrolledWindow::builder()
        .child(&list)
        .vexpand(true)
        .min_content_height(340)
        .build();
    scroll.add_css_class("frame");
    vbox.append(&scroll);
    popover.set_child(Some(&vbox));

    if include_auto {
        let row = gtk::ListBoxRow::new();
        row.set_widget_name(AUTO_CODE);
        let lbl = gtk::Label::new(Some("Auto detect"));
        lbl.set_halign(gtk::Align::Start);
        lbl.set_margin_top(6);
        lbl.set_margin_bottom(6);
        lbl.set_margin_start(8);
        row.set_child(Some(&lbl));
        list.append(&row);
    }

    for lang in LANGS {
        let row = gtk::ListBoxRow::new();
        row.set_widget_name(lang.code);
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        hbox.set_margin_start(8);
        hbox.set_margin_end(4);
        hbox.set_margin_top(2);
        hbox.set_margin_bottom(2);
        let lbl = gtk::Label::new(Some(lang.name));
        lbl.set_halign(gtk::Align::Start);
        lbl.set_hexpand(true);
        let fav = cfg.borrow().favorites.contains(&lang.code.to_string());
        let star = gtk::ToggleButton::new();
        star.set_has_frame(false);
        star.set_active(fav);
        star.set_icon_name(if fav {
            "starred-symbolic"
        } else {
            "non-starred-symbolic"
        });
        star.set_tooltip_text(Some("Favorite"));
        star.connect_toggled({
            let cfg = cfg.clone();
            let list = list.clone();
            let code = lang.code.to_string();
            move |btn| {
                let active = btn.is_active();
                btn.set_icon_name(if active {
                    "starred-symbolic"
                } else {
                    "non-starred-symbolic"
                });
                {
                    let mut c = cfg.borrow_mut();
                    if active {
                        if !c.favorites.contains(&code) {
                            c.favorites.push(code.clone());
                        }
                    } else {
                        c.favorites.retain(|x| x != &code);
                    }
                    config::save(&c);
                }
                list.invalidate_sort();
            }
        });
        hbox.append(&lbl);
        hbox.append(&star);
        row.set_child(Some(&hbox));
        list.append(&row);
    }

    // Auto row first, then favorites, then alphabetical.
    list.set_sort_func({
        let cfg = cfg.clone();
        move |a, b| {
            let rank = |row: &gtk::ListBoxRow| {
                let code = row.widget_name().to_string();
                if code == AUTO_CODE {
                    return (0u8, String::new());
                }
                let fav = cfg.borrow().favorites.contains(&code);
                (
                    if fav { 1 } else { 2 },
                    languages::name_for(&code).to_lowercase(),
                )
            };
            rank(a).cmp(&rank(b)).into()
        }
    });

    list.set_filter_func({
        let search = search.clone();
        move |row| {
            let q = search.text().to_lowercase();
            if q.is_empty() {
                return true;
            }
            let code = row.widget_name().to_string();
            code.to_lowercase().contains(&q)
                || languages::name_for(&code).to_lowercase().contains(&q)
        }
    });

    search.connect_search_changed({
        let list = list.clone();
        move |_| list.invalidate_filter()
    });

    list.connect_row_activated({
        let on_pick = on_pick.clone();
        let popover = popover.clone();
        move |_, row| {
            let code = row.widget_name().to_string();
            popover.popdown();
            on_pick(&code);
        }
    });

    // Reset search and focus it each time the popover opens.
    popover.connect_map({
        let search = search.clone();
        move |_| {
            search.set_text("");
            search.grab_focus();
        }
    });

    popover
}
