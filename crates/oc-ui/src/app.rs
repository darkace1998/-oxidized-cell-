//! Main application

use eframe::egui;
use oc_core::config::Config;

/// Main application state
pub struct OxidizedCellApp {
    /// Configuration
    config: Config,
    /// Current view
    current_view: View,
    /// Show settings window
    show_settings: bool,
    /// Show about window
    show_about: bool,
}

/// Application views
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    GameList,
    Emulation,
    Debugger,
}

impl OxidizedCellApp {
    /// Create a new application
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = Config::load().unwrap_or_default();
        
        Self {
            config,
            current_view: View::GameList,
            show_settings: false,
            show_about: false,
        }
    }
}

impl eframe::App for OxidizedCellApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open Game").clicked() {
                        // TODO: Open file dialog
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                
                ui.menu_button("Emulation", |ui| {
                    if ui.button("Start").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Pause").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Stop").clicked() {
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Reset").clicked() {
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("View", |ui| {
                    if ui.button("Game List").clicked() {
                        self.current_view = View::GameList;
                        ui.close_menu();
                    }
                    if ui.button("Debugger").clicked() {
                        self.current_view = View::Debugger;
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("Settings", |ui| {
                    if ui.button("Configuration").clicked() {
                        self.show_settings = true;
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.show_about = true;
                        ui.close_menu();
                    }
                });
            });
        });
        
        // Status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Ready");
                ui.separator();
                ui.label("FPS: --");
            });
        });
        
        // Main content
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_view {
                View::GameList => {
                    ui.heading("Game List");
                    ui.separator();
                    ui.label("No games found. Use File > Open Game to load a game.");
                }
                View::Emulation => {
                    ui.heading("Emulation");
                    ui.label("Game display would appear here.");
                }
                View::Debugger => {
                    ui.heading("Debugger");
                    ui.separator();
                    ui.label("Debugger interface would appear here.");
                }
            }
        });
        
        // Settings window
        if self.show_settings {
            egui::Window::new("Settings")
                .open(&mut self.show_settings)
                .show(ctx, |ui| {
                    ui.heading("CPU Settings");
                    ui.checkbox(&mut self.config.cpu.accurate_dfma, "Accurate DFMA");
                    ui.checkbox(&mut self.config.cpu.spu_loop_detection, "SPU Loop Detection");
                    
                    ui.separator();
                    
                    ui.heading("GPU Settings");
                    ui.checkbox(&mut self.config.gpu.vsync, "VSync");
                    ui.checkbox(&mut self.config.gpu.shader_cache, "Shader Cache");
                    
                    ui.separator();
                    
                    ui.heading("Audio Settings");
                    ui.checkbox(&mut self.config.audio.enable, "Enable Audio");
                    
                    ui.separator();
                    
                    if ui.button("Save").clicked() {
                        let _ = self.config.save();
                    }
                });
        }
        
        // About window
        if self.show_about {
            egui::Window::new("About")
                .open(&mut self.show_about)
                .show(ctx, |ui| {
                    ui.heading("oxidized-cell");
                    ui.label("PS3 Emulator");
                    ui.label("Version 0.1.0");
                    ui.separator();
                    ui.label("A Rust/C++ hybrid PS3 emulator.");
                    ui.label("Licensed under GPL-3.0");
                });
        }
    }
}

/// Run the application
pub fn run() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "oxidized-cell",
        options,
        Box::new(|cc| Ok(Box::new(OxidizedCellApp::new(cc)))),
    )
}
