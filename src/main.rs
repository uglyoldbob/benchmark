#![feature(portable_simd)]

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

    let ms = lm_sensors::Initializer::default().initialize();

    let ac = AppCommon { sensors: ms.ok() };

    let thread = std::thread::spawn(|| {
        println!("running long calc on cpu now");
        let mut num_cycles = 1000000;
        let mut sum = 0.0;
        loop {
            let clock = quanta::Clock::new();
            let start = clock.raw();
            let (each, r) = cpu::load_select(num_cycles);
            sum += r;
            let end = clock.raw();
            let d = clock.delta(start, end);
            if d.as_millis() < 1 {
                num_cycles *= 10;
            } else {
                let ratio = 1000.0 / d.as_millis() as f64;
                num_cycles = (num_cycles as f64 * ratio) as usize;
            }
            println!("Iterations is {} Number is {}", num_cycles * each, sum);
        }
    });

    let _e = multi_window.add(root_window, &event_loop);
    multi_window.run(event_loop, ac);
}
