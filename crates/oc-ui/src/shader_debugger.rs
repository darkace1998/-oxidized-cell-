//! Shader debugger panel for inspecting and debugging RSX shaders

use eframe::egui;

/// Shader type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderType {
    Vertex,
    Fragment,
}

impl ShaderType {
    fn label(&self) -> &'static str {
        match self {
            ShaderType::Vertex => "Vertex",
            ShaderType::Fragment => "Fragment",
        }
    }
}

/// Information about a compiled shader
#[derive(Debug, Clone)]
pub struct ShaderInfo {
    /// Shader ID
    pub id: u32,
    /// Shader type
    pub shader_type: ShaderType,
    /// Original RSX microcode size
    pub microcode_size: usize,
    /// SPIR-V binary size (after translation)
    pub spirv_size: usize,
    /// Compilation time in milliseconds
    pub compile_time_ms: f64,
    /// Number of instructions
    pub instruction_count: usize,
    /// Number of registers used
    pub registers_used: usize,
    /// Is cached on disk
    pub cached: bool,
    /// Original microcode (first N bytes for preview)
    pub microcode_preview: Vec<u8>,
    /// Disassembled source (if available)
    pub disassembly: Option<String>,
}

/// Shader debugger panel state
pub struct ShaderDebugger {
    /// Currently selected shader type filter
    shader_type_filter: Option<ShaderType>,
    /// Search query for shader list
    search_query: String,
    /// Selected shader ID
    selected_shader: Option<u32>,
    /// Shader list (cached from shader cache)
    shaders: Vec<ShaderInfo>,
    /// Show disassembly panel
    show_disassembly: bool,
    /// Show microcode panel
    show_microcode: bool,
    /// Status message
    status_message: String,
    /// Auto-refresh shader list
    auto_refresh: bool,
    /// Shader statistics
    stats: ShaderStats,
}

/// Shader cache statistics
#[derive(Debug, Clone, Default)]
pub struct ShaderStats {
    /// Total vertex shaders
    pub vertex_shader_count: usize,
    /// Total fragment shaders
    pub fragment_shader_count: usize,
    /// Cache hits
    pub cache_hits: usize,
    /// Cache misses
    pub cache_misses: usize,
    /// Total compilation time
    pub total_compile_time_ms: f64,
    /// Average compilation time
    pub avg_compile_time_ms: f64,
}

impl ShaderDebugger {
    /// Create a new shader debugger
    pub fn new() -> Self {
        Self {
            shader_type_filter: None,
            search_query: String::new(),
            selected_shader: None,
            shaders: Vec::new(),
            show_disassembly: true,
            show_microcode: false,
            status_message: String::from("Shader debugger ready"),
            auto_refresh: false,
            stats: ShaderStats::default(),
        }
    }

    /// Add a shader to the list (called when a shader is compiled)
    pub fn add_shader(&mut self, info: ShaderInfo) {
        // Update stats
        match info.shader_type {
            ShaderType::Vertex => self.stats.vertex_shader_count += 1,
            ShaderType::Fragment => self.stats.fragment_shader_count += 1,
        }
        self.stats.total_compile_time_ms += info.compile_time_ms;
        
        // Recalculate average
        let total = self.stats.vertex_shader_count + self.stats.fragment_shader_count;
        if total > 0 {
            self.stats.avg_compile_time_ms = self.stats.total_compile_time_ms / total as f64;
        }
        
        self.shaders.push(info);
    }

    /// Record a cache hit
    pub fn record_cache_hit(&mut self) {
        self.stats.cache_hits += 1;
    }

    /// Record a cache miss
    pub fn record_cache_miss(&mut self) {
        self.stats.cache_misses += 1;
    }

    /// Clear all shaders
    pub fn clear(&mut self) {
        self.shaders.clear();
        self.selected_shader = None;
        self.stats = ShaderStats::default();
        self.status_message = String::from("Shader list cleared");
    }

    /// Get filtered shaders based on current filters
    fn filtered_shaders(&self) -> Vec<&ShaderInfo> {
        self.shaders
            .iter()
            .filter(|s| {
                // Type filter
                if let Some(filter_type) = self.shader_type_filter {
                    if s.shader_type != filter_type {
                        return false;
                    }
                }
                // Search filter
                if !self.search_query.is_empty() {
                    let query = self.search_query.to_lowercase();
                    let id_str = format!("{}", s.id);
                    if !id_str.contains(&query) {
                        return false;
                    }
                }
                true
            })
            .collect()
    }

    /// Show the shader debugger panel
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("Shader Debugger");
        ui.add_space(5.0);

        // Statistics bar
        ui.horizontal(|ui| {
            ui.label(format!("Vertex: {}", self.stats.vertex_shader_count));
            ui.separator();
            ui.label(format!("Fragment: {}", self.stats.fragment_shader_count));
            ui.separator();
            ui.label(format!("Cache Hits: {}", self.stats.cache_hits));
            ui.separator();
            ui.label(format!("Cache Misses: {}", self.stats.cache_misses));
            ui.separator();
            ui.label(format!("Avg Compile: {:.2}ms", self.stats.avg_compile_time_ms));
        });

        ui.separator();

        // Toolbar
        ui.horizontal(|ui| {
            // Type filter
            ui.label("Filter:");
            egui::ComboBox::from_id_salt("shader_type_filter")
                .selected_text(match self.shader_type_filter {
                    None => "All",
                    Some(ShaderType::Vertex) => "Vertex",
                    Some(ShaderType::Fragment) => "Fragment",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.shader_type_filter, None, "All");
                    ui.selectable_value(&mut self.shader_type_filter, Some(ShaderType::Vertex), "Vertex");
                    ui.selectable_value(&mut self.shader_type_filter, Some(ShaderType::Fragment), "Fragment");
                });

            ui.separator();

            ui.label("Search:");
            ui.add(egui::TextEdit::singleline(&mut self.search_query)
                .desired_width(100.0)
                .hint_text("Shader ID..."));

            ui.separator();

            ui.checkbox(&mut self.show_disassembly, "Disassembly");
            ui.checkbox(&mut self.show_microcode, "Microcode");

            ui.separator();

            if ui.button("🔄 Refresh").clicked() {
                self.status_message = String::from("Refreshed shader list");
            }

            ui.checkbox(&mut self.auto_refresh, "Auto");

            ui.separator();

            if ui.button("🗑 Clear").clicked() {
                self.clear();
            }
        });

        ui.separator();

        // Main content - split view
        let available_height = ui.available_height() - 30.0;
        
        ui.horizontal(|ui| {
            // Shader list (left panel)
            egui::Frame::default()
                .inner_margin(4.0)
                .show(ui, |ui| {
                    ui.set_min_width(250.0);
                    ui.set_max_height(available_height);
                    
                    ui.label(egui::RichText::new("Shaders").strong());
                    ui.separator();

                    // Collect shader info outside the borrow
                    let shader_list: Vec<_> = self.filtered_shaders()
                        .iter()
                        .map(|s| (s.id, s.shader_type, s.instruction_count))
                        .collect();

                    let mut new_selection = None;

                    egui::ScrollArea::vertical()
                        .id_salt("shader_list")
                        .max_height(available_height - 30.0)
                        .show(ui, |ui| {
                            if shader_list.is_empty() {
                                ui.label("No shaders loaded.");
                                ui.label("Shaders will appear here when");
                                ui.label("compiled during emulation.");
                            } else {
                                for (id, shader_type, instruction_count) in &shader_list {
                                    let is_selected = self.selected_shader == Some(*id);
                                    let type_icon = match shader_type {
                                        ShaderType::Vertex => "🔺",
                                        ShaderType::Fragment => "🔷",
                                    };
                                    
                                    let label = format!(
                                        "{} #{} ({} inst)",
                                        type_icon,
                                        id,
                                        instruction_count
                                    );
                                    
                                    if ui.selectable_label(is_selected, label).clicked() {
                                        new_selection = Some(*id);
                                    }
                                }
                            }
                        });

                    // Update selection outside the loop
                    if let Some(id) = new_selection {
                        self.selected_shader = Some(id);
                        self.status_message = format!("Selected shader #{}", id);
                    }
                });

            ui.separator();

            // Shader details (right panel)
            egui::Frame::default()
                .inner_margin(4.0)
                .show(ui, |ui| {
                    ui.set_min_width(400.0);
                    ui.set_max_height(available_height);

                    if let Some(shader_id) = self.selected_shader {
                        if let Some(shader) = self.shaders.iter().find(|s| s.id == shader_id) {
                            self.show_shader_details(ui, shader);
                        } else {
                            ui.label("Shader not found.");
                        }
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.label("Select a shader to view details");
                        });
                    }
                });
        });

        // Status bar
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(&self.status_message).small());
        });
    }

    /// Show shader details panel
    fn show_shader_details(&self, ui: &mut egui::Ui, shader: &ShaderInfo) {
        ui.label(egui::RichText::new(format!("Shader #{} - {}", shader.id, shader.shader_type.label())).strong());
        ui.separator();

        // Properties grid
        egui::Grid::new("shader_props")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .show(ui, |ui| {
                ui.label("Type:");
                ui.label(shader.shader_type.label());
                ui.end_row();

                ui.label("Instructions:");
                ui.label(format!("{}", shader.instruction_count));
                ui.end_row();

                ui.label("Registers:");
                ui.label(format!("{}", shader.registers_used));
                ui.end_row();

                ui.label("Microcode Size:");
                ui.label(format!("{} bytes", shader.microcode_size));
                ui.end_row();

                ui.label("SPIR-V Size:");
                ui.label(format!("{} bytes", shader.spirv_size));
                ui.end_row();

                ui.label("Compile Time:");
                ui.label(format!("{:.2} ms", shader.compile_time_ms));
                ui.end_row();

                ui.label("Cached:");
                ui.label(if shader.cached { "Yes" } else { "No" });
                ui.end_row();
            });

        ui.add_space(10.0);

        // Disassembly section
        if self.show_disassembly {
            ui.collapsing("Disassembly", |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("shader_disasm")
                    .max_height(200.0)
                    .show(ui, |ui| {
                        if let Some(ref disasm) = shader.disassembly {
                            ui.monospace(disasm);
                        } else {
                            ui.label("Disassembly not available.");
                            ui.label("Enable 'Dump Shaders' in Debug settings.");
                        }
                    });
            });
        }

        // Microcode section
        if self.show_microcode {
            ui.collapsing("Microcode (First 64 bytes)", |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("shader_microcode")
                    .max_height(150.0)
                    .show(ui, |ui| {
                        if shader.microcode_preview.is_empty() {
                            ui.label("Microcode not available.");
                        } else {
                            // Display as hex dump
                            for (i, chunk) in shader.microcode_preview.chunks(16).enumerate() {
                                let addr = i * 16;
                                let hex: String = chunk
                                    .iter()
                                    .map(|b| format!("{:02X}", b))
                                    .collect::<Vec<_>>()
                                    .join(" ");
                                ui.monospace(format!("{:04X}: {}", addr, hex));
                            }
                        }
                    });
            });
        }
    }
}

impl Default for ShaderDebugger {
    fn default() -> Self {
        Self::new()
    }
}
