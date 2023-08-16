use egui_multiwin::egui_glow::EguiGlow;
use egui_multiwin::{
    multi_window::NewWindowRequest,
    tracked_window::{RedrawResponse, TrackedWindow},
};

#[cfg(target_os = "linux")]
use lm_sensors::prelude::*;

use crate::{AppCommon, MessageToGui};

pub struct RootWindow {}

impl RootWindow {
    pub fn new() -> NewWindowRequest<AppCommon> {
        NewWindowRequest {
            window_state: Box::new(RootWindow {}),
            builder: egui_multiwin::winit::window::WindowBuilder::new()
                .with_resizable(true)
                .with_transparent(true)
                .with_inner_size(egui_multiwin::winit::dpi::LogicalSize {
                    width: 800.0,
                    height: 600.0,
                })
                .with_title("egui-multiwin root window"),
            options: egui_multiwin::tracked_window::TrackedWindowOptions {
                vsync: false,
                shader: None,
            },
        }
    }
}

impl TrackedWindow<AppCommon> for RootWindow {
    fn is_root(&self) -> bool {
        true
    }

    fn set_root(&mut self, _root: bool) {}

    fn redraw(
        &mut self,
        c: &mut AppCommon,
        egui: &mut EguiGlow,
        window: &egui_multiwin::winit::window::Window,
    ) -> RedrawResponse<AppCommon> {
        let mut quit = false;

        let mut windows_to_create = vec![];

        egui.egui_ctx.request_repaint_after(std::time::Duration::from_millis(100));
        for thread in &mut c.cpu_threads {
            thread.process_messages();
        }
        while let Ok(message) = c.gui_recv.try_recv() {
            match message {
                MessageToGui::StopAllCpu => {
                    for t in &mut c.cpu_threads {
                        let _e = t.send.send(crate::cpu::MessageToCpuLoad::Stop);
                    }
                }
            }
        }

        egui_multiwin::egui::SidePanel::left("my_side_panel").show(&egui.egui_ctx, |ui| {
            ui.heading("Hello World!");
            if ui.button("Quit").clicked() {
                quit = true;
            }
        });

        egui_multiwin::egui::CentralPanel::default().show(&egui.egui_ctx, |ui| {
            ui.label("I am groot".to_string());
            egui_multiwin::egui::ScrollArea::vertical().show(ui, |ui| {
                #[cfg(target_os = "linux")]
                if let Some(sensors) = &c.sensors {
                    for chip in sensors.chip_iter(None) {
                        if let Some(p) = chip.path() {
                            ui.label(format!("chip {}", p.display()));
                        }

                        for feature in chip.feature_iter() {
                            let name = feature.name().transpose();
                            if let Ok(Some(name)) = name {
                                ui.label(format!("    {}: {}", name, feature));

                                // Print all sub-features of the current chip feature.
                                for sub_feature in feature.sub_feature_iter() {
                                    if let Ok(value) = sub_feature.value() {
                                        ui.label(format!("        {}: {}", sub_feature, value));
                                    } else {
                                        ui.label(format!("        {}: N/A", sub_feature));
                                    }
                                }
                            }
                        }
                    }
                }
                for thread in &mut c.cpu_threads {
                    ui.label(format!(
                        "CPU running {} {}",
                        thread.running, thread.associated
                    ));
                    ui.label(format!("Performance: {}", thread.performance));
                    ui.horizontal(|ui| {
                        if ui.button("Start").clicked() {
                            thread.send.send(crate::cpu::MessageToCpuLoad::Start);
                        }
                        if ui.button("Stop").clicked() {
                            thread.send.send(crate::cpu::MessageToCpuLoad::Stop);
                        }
                    });
                }
                if ui.button("Timed cpu load").clicked() {
                    let send = c.gui_send.clone();
                    for t in &mut c.cpu_threads {
                        let _e = t.send.send(crate::cpu::MessageToCpuLoad::Start);
                    }
                    c.timer
                        .schedule_with_delay(chrono::Duration::milliseconds(5000), move || {
                            println!("Stopping all cpu threads");
                            send.send(MessageToGui::StopAllCpu);
                        }).ignore();
                }
            });
        });

        RedrawResponse {
            quit: quit,
            new_windows: windows_to_create,
        }
    }
}
