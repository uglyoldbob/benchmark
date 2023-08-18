use egui_multiwin::egui_glow::EguiGlow;
use egui_multiwin::{
    multi_window::NewWindowRequest,
    tracked_window::{RedrawResponse, TrackedWindow},
};

use crate::AppCommon;

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

        for listener in &mut c.netlisteners {
            listener.process_messages();
        }

        egui.egui_ctx
            .request_repaint_after(std::time::Duration::from_millis(100));

        egui_multiwin::egui::CentralPanel::default().show(&egui.egui_ctx, |ui| {
            egui_multiwin::egui::ScrollArea::vertical().show(ui, |ui| {
                for listener in &mut c.netlisteners {
                    ui.label(format!("Listener {:?}", listener.addr));
                    ui.horizontal(|ui| {
                        if ui.button("Start").clicked() {
                            listener.send.send(crate::MessageToNetworkListener::Start);
                        }
                        if ui.button("Stop").clicked() {
                            listener.send.send(crate::MessageToNetworkListener::Stop);
                        }
                        ui.label(format!("Status: {} {}", listener.listening, listener.done));
                    });
                }
                for net in &c.networks {
                    for addr in &net.addr {
                        match addr {
                            network_interface::Addr::V4(v4) => {
                                ui.label(format!("{} v4: {}", net.name, v4.ip));
                            }
                            network_interface::Addr::V6(v6) => {
                                ui.label(format!("{} v6: {}", net.name, v6.ip));
                            }
                        }
                    }
                }
            });
        });

        RedrawResponse {
            quit: quit,
            new_windows: windows_to_create,
        }
    }
}
