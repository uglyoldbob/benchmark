#![feature(portable_simd)]
#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)] // hide console window on Windows in release

use std::time::Instant;

use egui_multiwin::multi_window::MultiWindow;

mod cpu;
mod windows;

use windows::root::{self};

pub struct AppCommon {
    #[cfg(target_os = "linux")]
    sensors: Option<lm_sensors::LMSensors>,
}

impl egui_multiwin::multi_window::CommonEventHandler<AppCommon, u32> for AppCommon {
    fn process_event(
        &mut self,
        event: u32,
    ) -> Vec<egui_multiwin::multi_window::NewWindowRequest<AppCommon>> {
        let mut windows_to_create = vec![];
        println!("Received an event {}", event);
        match event {
            _ => {}
        }
        windows_to_create
    }
}

fn main() {
    let event_loop = egui_multiwin::winit::event_loop::EventLoopBuilder::with_user_event().build();
    let mut multi_window: MultiWindow<AppCommon, u32> = MultiWindow::new();
    let root_window = root::RootWindow::new();

    #[cfg(target_os = "linux")]
    let ms = lm_sensors::Initializer::default().initialize();

    let ac = AppCommon {
        #[cfg(target_os = "linux")]
        sensors: ms.ok(),
    };

    let thread = cpu::CpuLoadThread::new();

    let _e = multi_window.add(root_window, &event_loop);
    multi_window.run(event_loop, ac);
}
