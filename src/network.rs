#![feature(portable_simd)]
#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)] // hide console window on Windows in release

use cpu::MessageToCpuLoad;
use egui_multiwin::multi_window::MultiWindow;

mod cpu;
mod windows_network;

use network_interface::NetworkInterfaceConfig;
use windows_network::root::{self};

pub enum MessageToGui {
    StopAllCpu,
}

pub struct AppCommon {
    networks: Vec<network_interface::NetworkInterface>,
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

    println!("Starting application");

    #[cfg(target_os = "linux")]
    let ms = lm_sensors::Initializer::default().initialize();

    let mut threads = vec![];
    #[cfg(feature = "hwlocality")]
    let topology = hwlocality::Topology::new();
    if let Err(e) = &topology {
        println!("Error obtaining topology {}", e);
    }
    let mut topology = topology.ok();
    if let Some(topology) = &mut topology {
        let root = topology.root_object();
        let cpuset = root.cpuset();
        if let Some(cpuset) = cpuset {
            for index in cpuset.iter_set() {
                let thread = cpu::CpuLoadThread::new();
                thread
                    .send
                    .send(MessageToCpuLoad::Associate(topology.clone(), index.into()));
                threads.push(thread);
            }
        }
    }

    let mut networks = vec![];
    if let Ok(mut n) = network_interface::NetworkInterface::show() {
        networks.append(&mut n);
    }

    let ac = AppCommon {
        networks,
    };

    let _e = multi_window.add(root_window, &event_loop);
    multi_window.run(event_loop, ac);
}
