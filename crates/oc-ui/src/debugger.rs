//! Debugger UI

use eframe::egui;
use oc_debug::{PpuDebugger, SpuDebugger, RsxDebugger, Profiler, PpuDisassembler};
use oc_debug::ppu_debugger::DebugState;

/// Debugger view state
pub struct DebuggerView {
    /// Current debugger tab
    current_tab: DebuggerTab,
    /// Memory viewer address
    memory_address: String,
    /// Memory viewer data
    memory_data: Vec<u8>,
    /// Disassembly address
    disasm_address: String,
    /// Disassembled instructions
    disassembled: Vec<DisasmLine>,
    /// Breakpoints with enabled state
    breakpoints: Vec<(u32, bool)>,
    /// New breakpoint address input
    breakpoint_input: String,
    /// PPU Debugger
    ppu_debugger: PpuDebugger,
    /// SPU Debugger
    spu_debugger: SpuDebugger,
    /// RSX Debugger
    rsx_debugger: RsxDebugger,
    /// Profiler
    profiler: Profiler,
    /// Status message
    status_message: String,
}

/// Debugger tabs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DebuggerTab {
    Registers,
    Memory,
    Disassembly,
    Breakpoints,
    Profiler,
}

/// Disassembly line for display
struct DisasmLine {
    address: u64,
    bytes: String,
    instruction: String,
}

impl DebuggerView {
    /// Create a new debugger view
    pub fn new() -> Self {
        Self {
            current_tab: DebuggerTab::Registers,
            memory_address: String::from("0x00000000"),
            memory_data: vec![0; 256],
            disasm_address: String::from("0x00000000"),
            disassembled: Vec::new(),
            breakpoints: Vec::new(),
            breakpoint_input: String::new(),
            ppu_debugger: PpuDebugger::new(),
            spu_debugger: SpuDebugger::new(),
            rsx_debugger: RsxDebugger::new(),
            profiler: Profiler::new(),
            status_message: String::from("Ready"),
        }
    }

    /// Get reference to PPU debugger
    pub fn ppu_debugger(&self) -> &PpuDebugger {
        &self.ppu_debugger
    }

    /// Get mutable reference to PPU debugger
    pub fn ppu_debugger_mut(&mut self) -> &mut PpuDebugger {
        &mut self.ppu_debugger
    }

    /// Get reference to SPU debugger
    pub fn spu_debugger(&self) -> &SpuDebugger {
        &self.spu_debugger
    }

    /// Get mutable reference to SPU debugger
    pub fn spu_debugger_mut(&mut self) -> &mut SpuDebugger {
        &mut self.spu_debugger
    }

    /// Get reference to RSX debugger
    pub fn rsx_debugger(&self) -> &RsxDebugger {
        &self.rsx_debugger
    }

    /// Get mutable reference to RSX debugger
    pub fn rsx_debugger_mut(&mut self) -> &mut RsxDebugger {
        &mut self.rsx_debugger
    }

    /// Get reference to profiler
    pub fn profiler(&self) -> &Profiler {
        &self.profiler
    }

    /// Get mutable reference to profiler
    pub fn profiler_mut(&mut self) -> &mut Profiler {
        &mut self.profiler
    }

    /// Show the debugger view
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.current_tab, DebuggerTab::Registers, "Registers");
            ui.selectable_value(&mut self.current_tab, DebuggerTab::Memory, "Memory");
            ui.selectable_value(&mut self.current_tab, DebuggerTab::Disassembly, "Disassembly");
            ui.selectable_value(&mut self.current_tab, DebuggerTab::Breakpoints, "Breakpoints");
            ui.selectable_value(&mut self.current_tab, DebuggerTab::Profiler, "Profiler");
        });

        ui.separator();

        // Control buttons with debug state integration
        let is_paused = self.ppu_debugger.is_paused();
        ui.horizontal(|ui| {
            let continue_btn = ui.add_enabled(is_paused, egui::Button::new("▶ Continue"));
            if continue_btn.clicked() {
                self.ppu_debugger.resume();
                self.status_message = String::from("Resumed execution");
                tracing::info!("Debugger: Resume execution");
            }
            
            let pause_btn = ui.add_enabled(!is_paused, egui::Button::new("⏸ Pause"));
            if pause_btn.clicked() {
                self.ppu_debugger.pause();
                self.status_message = String::from("Paused execution");
                tracing::info!("Debugger: Pause execution");
            }
            
            let step_btn = ui.add_enabled(is_paused, egui::Button::new("⏭ Step"));
            if step_btn.clicked() {
                self.ppu_debugger.step();
                self.status_message = String::from("Single step");
                tracing::info!("Debugger: Single step");
            }
            
            let step_over_btn = ui.add_enabled(is_paused, egui::Button::new("⏩ Step Over"));
            if step_over_btn.clicked() {
                // Note: In a real implementation, we'd get the current PC from the thread
                self.ppu_debugger.step_over(0, 0);
                self.status_message = String::from("Step over");
                tracing::info!("Debugger: Step over");
            }

            // Debug state indicator
            ui.separator();
            let state_text = match self.ppu_debugger.state {
                DebugState::Running => egui::RichText::new("● Running").color(egui::Color32::GREEN),
                DebugState::Paused => egui::RichText::new("● Paused").color(egui::Color32::YELLOW),
                DebugState::Stepping => egui::RichText::new("● Stepping").color(egui::Color32::BLUE),
                DebugState::SteppingOver => egui::RichText::new("● Step Over").color(egui::Color32::BLUE),
            };
            ui.label(state_text);
        });

        // Status bar
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(&self.status_message).small());
        });

        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            match self.current_tab {
                DebuggerTab::Registers => self.show_registers(ui),
                DebuggerTab::Memory => self.show_memory(ui),
                DebuggerTab::Disassembly => self.show_disassembly(ui),
                DebuggerTab::Breakpoints => self.show_breakpoints(ui),
                DebuggerTab::Profiler => self.show_profiler(ui),
            }
        });
    }

    fn show_registers(&self, ui: &mut egui::Ui) {
        ui.heading("PPU Registers");
        ui.add_space(10.0);

        // General Purpose Registers
        ui.label(egui::RichText::new("General Purpose Registers (GPRs)").strong());
        egui::Grid::new("gpr_grid")
            .striped(true)
            .num_columns(4)
            .show(ui, |ui| {
                for i in 0..32 {
                    if i % 4 == 0 && i > 0 {
                        ui.end_row();
                    }
                    ui.label(format!("R{:02}:", i));
                    ui.label(egui::RichText::new("0x0000000000000000").monospace());
                }
            });

        ui.add_space(10.0);

        // Floating Point Registers
        ui.label(egui::RichText::new("Floating Point Registers (FPRs)").strong());
        egui::Grid::new("fpr_grid")
            .striped(true)
            .num_columns(4)
            .show(ui, |ui| {
                for i in 0..32 {
                    if i % 4 == 0 && i > 0 {
                        ui.end_row();
                    }
                    ui.label(format!("F{:02}:", i));
                    ui.label(egui::RichText::new("0.0").monospace());
                }
            });

        ui.add_space(10.0);

        // Special Registers
        ui.label(egui::RichText::new("Special Registers").strong());
        egui::Grid::new("special_regs")
            .striped(true)
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("PC:");
                ui.label(egui::RichText::new("0x00000000").monospace());
                ui.end_row();

                ui.label("LR:");
                ui.label(egui::RichText::new("0x00000000").monospace());
                ui.end_row();

                ui.label("CTR:");
                ui.label(egui::RichText::new("0x00000000").monospace());
                ui.end_row();

                ui.label("CR:");
                ui.label(egui::RichText::new("0x00000000").monospace());
                ui.end_row();

                ui.label("XER:");
                ui.label(egui::RichText::new("0x00000000").monospace());
                ui.end_row();

                ui.label("FPSCR:");
                ui.label(egui::RichText::new("0x00000000").monospace());
                ui.end_row();
            });
    }

    fn show_memory(&mut self, ui: &mut egui::Ui) {
        ui.heading("Memory Viewer");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("Address:");
            ui.text_edit_singleline(&mut self.memory_address);
            if ui.button("Go").clicked() {
                if let Ok(addr) = self.parse_address(&self.memory_address) {
                    // Try to read memory from the debugger
                    if let Some(data) = self.ppu_debugger.read_memory(addr, 256) {
                        self.memory_data = data;
                        self.status_message = format!("Loaded memory at 0x{:08X}", addr);
                    } else {
                        // Use placeholder data if memory not available
                        self.memory_data = vec![0; 256];
                        self.status_message = format!("Memory at 0x{:08X} (no data available)", addr);
                    }
                } else {
                    self.status_message = String::from("Invalid address format");
                }
            }
        });

        ui.add_space(10.0);

        // Hex dump display
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.monospace("Address    00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F  ASCII");
            ui.separator();

            let base_addr = self.parse_address(&self.memory_address).unwrap_or(0);
            for (i, chunk) in self.memory_data.chunks(16).enumerate() {
                let addr = base_addr + (i as u32 * 16);
                let hex: String = chunk
                    .iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");
                
                let ascii: String = chunk
                    .iter()
                    .map(|&b| {
                        if b >= 0x20 && b <= 0x7E {
                            b as char
                        } else {
                            '.'
                        }
                    })
                    .collect();

                ui.monospace(format!("0x{:08X}  {:48}  {}", addr, hex, ascii));
            }
        });
    }

    fn show_disassembly(&mut self, ui: &mut egui::Ui) {
        ui.heading("Disassembly");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("Address:");
            ui.text_edit_singleline(&mut self.disasm_address);
            if ui.button("Go").clicked() {
                if let Ok(addr) = self.parse_address(&self.disasm_address) {
                    // Disassemble instructions using the debugger's disassembler
                    let instructions = self.ppu_debugger.disassemble_at(addr as u64, 20);
                    self.disassembled = instructions.iter().map(|inst| {
                        DisasmLine {
                            address: inst.address,
                            bytes: inst.opcode_hex(),
                            instruction: inst.to_string(),
                        }
                    }).collect();
                    
                    if self.disassembled.is_empty() {
                        // Use mock data if no memory available
                        self.generate_mock_disassembly(addr as u64);
                    }
                    self.status_message = format!("Disassembled at 0x{:08X}", addr);
                } else {
                    self.status_message = String::from("Invalid address format");
                }
            }
        });

        ui.add_space(10.0);

        // Disassembly display
        egui::Grid::new("disasm_grid")
            .striped(true)
            .num_columns(3)
            .show(ui, |ui| {
                ui.strong("Address");
                ui.strong("Bytes");
                ui.strong("Instruction");
                ui.end_row();

                if self.disassembled.is_empty() {
                    // Show placeholder data
                    let mock_instructions = [
                        ("0x00000000", "7C 08 02 A6", "mflr    r0"),
                        ("0x00000004", "FB E1 FF F8", "std     r31, -8(r1)"),
                        ("0x00000008", "F8 21 FF 91", "stdu    r1, -112(r1)"),
                        ("0x0000000C", "7C 3F 0B 78", "mr      r31, r1"),
                        ("0x00000010", "F8 01 00 80", "std     r0, 128(r1)"),
                        ("0x00000014", "38 60 00 00", "li      r3, 0"),
                        ("0x00000018", "48 00 00 01", "bl      0x0000001C"),
                    ];

                    for (addr, bytes, inst) in mock_instructions {
                        ui.label(egui::RichText::new(addr).monospace());
                        ui.label(egui::RichText::new(bytes).monospace());
                        ui.label(egui::RichText::new(inst).monospace());
                        ui.end_row();
                    }
                } else {
                    for line in &self.disassembled {
                        ui.label(egui::RichText::new(format!("0x{:08X}", line.address)).monospace());
                        ui.label(egui::RichText::new(&line.bytes).monospace());
                        ui.label(egui::RichText::new(&line.instruction).monospace());
                        ui.end_row();
                    }
                }
            });
    }

    fn generate_mock_disassembly(&mut self, start_addr: u64) {
        // Generate some mock disassembly for display
        let mock_opcodes: [(u32, &str); 7] = [
            (0x7C0802A6, "mflr    r0"),
            (0xFBE1FFF8, "std     r31, -8(r1)"),
            (0xF821FF91, "stdu    r1, -112(r1)"),
            (0x7C3F0B78, "mr      r31, r1"),
            (0xF8010080, "std     r0, 128(r1)"),
            (0x38600000, "li      r3, 0"),
            (0x4E800020, "blr"),
        ];

        self.disassembled = mock_opcodes.iter().enumerate().map(|(i, (opcode, _))| {
            let addr = start_addr + (i as u64 * 4);
            let disasm = PpuDisassembler::disassemble(addr, *opcode);
            DisasmLine {
                address: addr,
                bytes: disasm.opcode_hex(),
                instruction: disasm.to_string(),
            }
        }).collect();
    }

    fn show_breakpoints(&mut self, ui: &mut egui::Ui) {
        ui.heading("Breakpoints");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("Add Breakpoint:");
            ui.text_edit_singleline(&mut self.breakpoint_input);
            if ui.button("Add").clicked() {
                if let Ok(addr) = self.parse_address(&self.breakpoint_input) {
                    // Add to both UI list and debugger
                    self.breakpoints.push((addr, true));
                    self.ppu_debugger.breakpoints.add_execution_breakpoint(addr as u64);
                    self.breakpoint_input.clear();
                    self.status_message = format!("Added breakpoint at 0x{:08X}", addr);
                } else {
                    self.status_message = String::from("Invalid address format");
                }
            }
        });

        ui.add_space(10.0);

        // Show breakpoint count
        let bp_count = self.ppu_debugger.breakpoints.count();
        ui.label(format!("Total breakpoints: {}", bp_count));
        ui.add_space(5.0);

        if self.breakpoints.is_empty() {
            ui.label("No breakpoints set.");
        } else {
            egui::Grid::new("breakpoints_grid")
                .striped(true)
                .num_columns(3)
                .show(ui, |ui| {
                    ui.strong("Address");
                    ui.strong("Enabled");
                    ui.strong("Actions");
                    ui.end_row();

                    let mut to_remove = None;

                    for (i, (addr, enabled)) in self.breakpoints.iter_mut().enumerate() {
                        ui.label(egui::RichText::new(format!("0x{:08X}", addr)).monospace());
                        if ui.checkbox(enabled, "").changed() {
                            // Update debugger breakpoint state
                            // Note: This would need the breakpoint ID in a real implementation
                            self.status_message = format!(
                                "Breakpoint at 0x{:08X} {}",
                                addr,
                                if *enabled { "enabled" } else { "disabled" }
                            );
                        }
                        if ui.button("Remove").clicked() {
                            to_remove = Some(i);
                        }
                        ui.end_row();
                    }

                    if let Some(idx) = to_remove {
                        let (addr, _) = self.breakpoints.remove(idx);
                        self.status_message = format!("Removed breakpoint at 0x{:08X}", addr);
                    }
                });
        }
    }

    fn show_profiler(&mut self, ui: &mut egui::Ui) {
        ui.heading("Performance Profiler");
        ui.add_space(10.0);

        // Profiler controls
        ui.horizontal(|ui| {
            if self.profiler.enabled {
                if ui.button("⏹ Stop Profiling").clicked() {
                    self.profiler.disable();
                    self.status_message = String::from("Profiling stopped");
                }
            } else {
                if ui.button("▶ Start Profiling").clicked() {
                    self.profiler.enable();
                    self.status_message = String::from("Profiling started");
                }
            }

            if ui.button("🔄 Reset").clicked() {
                self.profiler.reset();
                self.status_message = String::from("Profiler reset");
            }

            ui.separator();
            let status = if self.profiler.enabled {
                egui::RichText::new("● Profiling").color(egui::Color32::RED)
            } else {
                egui::RichText::new("○ Stopped").color(egui::Color32::GRAY)
            };
            ui.label(status);
        });

        ui.add_space(10.0);

        // Performance summary
        ui.label(egui::RichText::new("Performance Summary").strong());
        egui::Grid::new("perf_summary")
            .striped(true)
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Average FPS:");
                ui.label(format!("{:.1}", self.profiler.get_average_fps()));
                ui.end_row();

                ui.label("Frame Time:");
                ui.label(format!("{:.2} ms", self.profiler.get_average_frame_time_ms()));
                ui.end_row();

                ui.label("Session Duration:");
                ui.label(format!("{:.1} s", self.profiler.session_duration().as_secs_f64()));
                ui.end_row();
            });

        ui.add_space(10.0);

        // Top profile sections
        ui.label(egui::RichText::new("Top Sections by Time").strong());
        let entries = self.profiler.get_entries();
        if entries.is_empty() {
            ui.label("No profiling data yet.");
        } else {
            egui::Grid::new("profile_entries")
                .striped(true)
                .num_columns(4)
                .show(ui, |ui| {
                    ui.strong("Section");
                    ui.strong("Total Time");
                    ui.strong("Calls");
                    ui.strong("Avg Time");
                    ui.end_row();

                    for entry in entries.iter().take(10) {
                        ui.label(&entry.name);
                        ui.label(format!("{:.2} ms", entry.total_time.as_secs_f64() * 1000.0));
                        ui.label(format!("{}", entry.call_count));
                        ui.label(format!("{:.3} ms", entry.average().as_secs_f64() * 1000.0));
                        ui.end_row();
                    }
                });
        }

        ui.add_space(10.0);

        // Hotspots
        ui.label(egui::RichText::new("PPU Hotspots").strong());
        let hotspots = self.profiler.get_ppu_hotspots(5);
        if hotspots.is_empty() {
            ui.label("No hotspot data yet.");
        } else {
            egui::Grid::new("ppu_hotspots")
                .striped(true)
                .num_columns(3)
                .show(ui, |ui| {
                    ui.strong("Address");
                    ui.strong("Hits");
                    ui.strong("Percentage");
                    ui.end_row();

                    for hotspot in &hotspots {
                        ui.label(egui::RichText::new(format!("0x{:016X}", hotspot.address)).monospace());
                        ui.label(format!("{}", hotspot.hit_count));
                        ui.label(format!("{:.2}%", hotspot.percentage));
                        ui.end_row();
                    }
                });
        }
    }

    fn parse_address(&self, s: &str) -> Result<u32, ()> {
        let s = s.trim();
        if s.starts_with("0x") || s.starts_with("0X") {
            u32::from_str_radix(&s[2..], 16).map_err(|_| ())
        } else {
            s.parse().map_err(|_| ())
        }
    }
}

impl Default for DebuggerView {
    fn default() -> Self {
        Self::new()
    }
}
