use std::collections::HashMap;
use eframe::egui;
use unibrowser_engine::{
    CommandRegistry, DocumentSession, KeybindingMap, ModeController, ToolbarController,
    keybinding::COMMAND_PALETTE_TOGGLE,
    palette::search_commands,
    KeyChord,
};
use unibrowser_mod_host::{ModHost, ModRegisterFn, default_mods_dir};

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 720.0])
            .with_title("Unibrowser"),
        ..Default::default()
    };
    eframe::run_native(
        "Unibrowser",
        native_options,
        Box::new(|_cc| Box::new(UnibrowserApp::new())),
    )
}

struct UnibrowserApp {
    registry: CommandRegistry,
    _mod_host: ModHost,
    documents: Vec<DocumentSession>,
    active_doc: usize,
    mode_ctrl: ModeController,
    toolbar: ToolbarController,
    palette_open: bool,
    palette_query: String,
    keybindings: KeybindingMap,
    status: String,
    mods_loaded: String,
    demo_selection: Vec<bool>,
}

impl UnibrowserApp {
    fn new() -> Self {
        let mut registry = CommandRegistry::with_builtins();
        let bundled: HashMap<&str, ModRegisterFn> = [
            ("base-text", base_text::register as ModRegisterFn),
            ("base-png", base_png::register as ModRegisterFn),
            ("toy-markdown", toy_markdown::register as ModRegisterFn),
        ]
        .into_iter()
        .collect();
        let mod_roots = vec![default_mods_dir()];
        let _mod_host =
            ModHost::discover_and_load(&mod_roots, &mut registry, &bundled).expect("load mods");
        let mods_loaded = _mod_host.load_order.join(", ");

        let demo_selection = vec![true, false, true];
        let mut documents = vec![
            DocumentSession::new("notes.txt", base_text::open_sample("notes.txt")),
            DocumentSession::new("photo.png", base_png::open_sample("photo.png")),
            DocumentSession::new("readme.md", toy_markdown::open_sample("readme.md")),
        ];
        for (doc, sel) in documents.iter_mut().zip(demo_selection.iter()) {
            doc.backend.set_demo_text_selection(*sel);
        }

        let mut toolbar = ToolbarController::default();
        if let Some(doc) = documents.first() {
            toolbar.select_tab_for_mode(doc.backend.mode());
        }

        let keybindings = KeybindingMap::load_merged(None);
        let status = format!(
            "{} for command palette",
            keybindings.palette_chord_display()
        );

        Self {
            registry,
            _mod_host,
            documents,
            active_doc: 0,
            mode_ctrl: ModeController::default(),
            toolbar,
            palette_open: false,
            palette_query: String::new(),
            keybindings,
            status,
            mods_loaded,
            demo_selection,
        }
    }

    fn egui_key_to_chord_key(key: egui::Key) -> Option<String> {
        match key {
            egui::Key::P => Some("p".into()),
            egui::Key::Equals | egui::Key::Plus => Some("=".into()),
            egui::Key::Minus => Some("-".into()),
            _ => None,
        }
    }

    fn handle_keybindings(&mut self, ctx: &egui::Context) {
        let chords: Vec<KeyChord> = ctx.input(|i| {
            let mut out = Vec::new();
            for event in &i.events {
                let egui::Event::Key {
                    key,
                    pressed: true,
                    repeat: false,
                    ..
                } = event
                else {
                    continue;
                };
                let Some(key_str) = Self::egui_key_to_chord_key(*key) else {
                    continue;
                };
                out.push(KeyChord {
                    ctrl: i.modifiers.ctrl,
                    alt: i.modifiers.alt,
                    shift: i.modifiers.shift,
                    meta: i.modifiers.command,
                    key: key_str,
                });
            }
            out
        });
        for chord in chords {
            let command_id = self
                .keybindings
                .command_for_chord(&chord)
                .map(|id| id.to_string());
            let Some(command_id) = command_id else {
                continue;
            };
            if command_id == COMMAND_PALETTE_TOGGLE {
                self.palette_open = !self.palette_open;
            } else {
                self.run_command(&command_id);
            }
        }
    }

    fn refresh_toolbar_tab(&mut self) {
        let doc = &self.documents[self.active_doc];
        let mode = self.mode_ctrl.active_mode(doc);
        self.toolbar.select_tab_for_mode(mode);
    }

    fn run_command(&mut self, command_id: &str) {
        let doc = &mut self.documents[self.active_doc];
        match self.registry.execute(command_id, doc) {
            Ok(msg) => self.status = msg,
            Err(e) => self.status = e,
        }
    }
}

impl eframe::App for UnibrowserApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !ctx.wants_keyboard_input() {
            self.handle_keybindings(ctx);
        }

        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Mods: {}", self.mods_loaded));
                ui.separator();
                ui.label(&self.status);
            });
        });

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            let mut switch_to = None;
            ui.horizontal(|ui| {
                for (idx, doc) in self.documents.iter().enumerate() {
                    let label = doc.path.clone();
                    if ui.selectable_label(self.active_doc == idx, label).clicked() {
                        switch_to = Some(idx);
                    }
                }
            });
            if let Some(idx) = switch_to {
                self.active_doc = idx;
                self.refresh_toolbar_tab();
            }

            ui.separator();
            ui.horizontal(|ui| {
                for tab in &self.toolbar.config.toolbar_tabs {
                    let selected = self.toolbar.config.selected_tab == tab.id;
                    if ui.selectable_label(selected, &tab.label).clicked() {
                        self.toolbar.config.selected_tab = tab.id.clone();
                    }
                }
            });
            ui.separator();

            let commands: Vec<String> = self
                .toolbar
                .active_tab()
                .map(|t| t.commands.clone())
                .unwrap_or_default();
            let host_ctx = self.documents[self.active_doc].host_context();
            ui.horizontal(|ui| {
                for cmd_id in &commands {
                    let meta = self.registry.command_meta(cmd_id);
                    let title = meta.map(|m| m.title).unwrap_or(cmd_id.as_str());
                    let en = self.registry.enablement_ctx(cmd_id, &host_ctx);
                    let enabled = en.is_enabled();
                    let mut btn = egui::Button::new(title);
                    if !enabled {
                        btn = btn.fill(egui::Color32::from_gray(55));
                    }
                    let response = ui.add_enabled(enabled, btn);
                    if !enabled {
                        if response.hovered() {
                            egui::show_tooltip_at_pointer(
                                ui.ctx(),
                                egui::Id::new(("tip", cmd_id)),
                                |ui| {
                                    ui.label(en.disabled_reason().unwrap_or("Unavailable"));
                                },
                            );
                        }
                    } else if response.clicked() {
                        self.run_command(cmd_id);
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let active = self.active_doc;
            let mut sel = self.demo_selection[active];
            if ui.checkbox(&mut sel, "Simulate text selection").changed() {
                self.demo_selection[active] = sel;
                self.documents[active]
                    .backend
                    .set_demo_text_selection(sel);
            }
            let doc = &self.documents[active];
            ui.heading(doc.backend.display_name());
            ui.label(format!("Backend: {}", doc.backend.backend_id()));
            ui.label(doc.backend.status_line());
            ui.separator();
            ui.label("Document surface placeholder (Tauri WebView in Phase 1).");
        });

        if self.palette_open {
            egui::Window::new(format!(
                "Command palette ({})",
                self.keybindings.palette_chord_display()
            ))
                .collapsible(false)
                .resizable(true)
                .default_width(420.0)
                .show(ctx, |ui| {
                    ui.label("Fuzzy search — disabled commands stay visible.");
                    ui.text_edit_singleline(&mut self.palette_query);
                    let doc = &self.documents[self.active_doc];
                    let ctx_host = doc.host_context();
                    let entries =
                        search_commands(&self.registry, &ctx_host, &self.palette_query);
                    egui::ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
                        for entry in entries {
                            let enabled = entry.enablement.is_enabled();
                            let label = if enabled {
                                entry.title.clone()
                            } else {
                                format!("{} (greyed)", entry.title)
                            };
                            if ui
                                .add_enabled(enabled, egui::Button::new(label))
                                .clicked()
                            {
                                self.run_command(&entry.command_id);
                                self.palette_open = false;
                            }
                            if !enabled {
                                ui.label(
                                    egui::RichText::new(
                                        entry
                                            .enablement
                                            .disabled_reason()
                                            .unwrap_or(""),
                                    )
                                        .small()
                                        .weak(),
                                );
                            }
                        }
                    });
                    if ui.button("Close").clicked() {
                        self.palette_open = false;
                    }
                });
        }
    }
}
