#![feature(portable_simd)]
#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)] // hide console window on Windows in release

use std::{net::UdpSocket, time::Duration};

use cpu::MessageToCpuLoad;
use egui_multiwin::multi_window::MultiWindow;

mod cpu;
mod windows_network;

use network_interface::NetworkInterfaceConfig;
use windows_network::root::{self};

pub enum MessageToGui {
    StopAllCpu,
}

enum MessageFromNetworkListener {
    Listening(bool),
    Done,
}

enum MessageToNetworkListener {
    Start,
    Stop,
    Exit,
}

struct NetworkListener {
    thread: std::thread::JoinHandle<()>,
    recv: std::sync::mpsc::Receiver<MessageFromNetworkListener>,
    pub send: std::sync::mpsc::Sender<MessageToNetworkListener>,
    listening: bool,
    done: bool,
    pub addr: network_interface::Addr,
}

impl NetworkListener {
    pub fn process_messages(&mut self) {
        while let Ok(message) = self.recv.try_recv() {
            match message {
                MessageFromNetworkListener::Listening(l) => {
                    self.listening = l;
                }
                MessageFromNetworkListener::Done => {
                    self.done = true;
                }
            }
        }
    }

    fn new(addr: &network_interface::Addr) -> Self {
        let addr = addr.to_owned();
        let (s, r) = std::sync::mpsc::channel();
        let (s2, r2) = std::sync::mpsc::channel();
        let thread = std::thread::spawn(move || {
            let mut running = false;
            let mut socket: Option<UdpSocket> = None;
            let mut broadcast_socket: Option<UdpSocket> = None;
            let mut buf: [u8; 10000] = [0; 10000];
            'main: loop {
                while let Ok(message) = r.try_recv() {
                    match message {
                        MessageToNetworkListener::Start => {
                            running = true;
                            if s2
                                .send(MessageFromNetworkListener::Listening(running))
                                .is_err()
                            {
                                break 'main;
                            }
                        }
                        MessageToNetworkListener::Stop => {
                            running = false;
                            socket = None;
                            if s2
                                .send(MessageFromNetworkListener::Listening(running))
                                .is_err()
                            {
                                break 'main;
                            }
                        }
                        MessageToNetworkListener::Exit => break 'main,
                    }
                }
                if running {
                    if socket.is_none() {
                        let s = match addr {
                            network_interface::Addr::V4(a) => {
                                UdpSocket::bind((a.ip, 5003))
                            }
                            network_interface::Addr::V6(a) => {
                                UdpSocket::bind((a.ip, 5003))
                            }
                        };
                        let broad = match addr {
                            network_interface::Addr::V4(a) => {
                                if let Some(b) = a.broadcast {
                                    Some(UdpSocket::bind((b, 5003)))
                                }
                                else {
                                    None
                                }
                            }
                            network_interface::Addr::V6(a) => {
                                if let Some(b) = a.broadcast {
                                    Some(UdpSocket::bind((b, 5003)))
                                }
                                else {
                                    None
                                }
                            }
                        };
                        if let Some(Ok(broad)) = broad {
                            if broad.set_nonblocking(true).is_ok() {
                                broadcast_socket = Some(broad);
                            }
                        }
                        if let Ok(r) = s {
                            if r.set_nonblocking(true).is_ok() {
                                socket = Some(r);
                            }
                        }
                    } else {
                        if let Some(s) = &mut broadcast_socket {
                            if let Ok((_size, addr)) = s.recv_from(&mut buf[..]) {
                                println!("Received broadcast from {:?} {}", addr, buf[0]);
                                if buf[0] == b'A' {
                                    s.send_to(&buf[..], addr);
                                }
                            }
                        }
                        if let Some(s) = &mut socket {
                            if let Ok((_size, addr)) = s.recv_from(&mut buf[..]) {
                                println!("Received from {:?} {}", addr, buf[0]);
                                s.send_to(&buf[..], addr);
                            }
                        }
                    }
                } else {
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
            let _e = s2.send(MessageFromNetworkListener::Done);
        });
        Self {
            thread,
            recv: r2,
            send: s,
            listening: false,
            done: false,
            addr,
        }
    }
}

pub struct AppCommon {
    networks: Vec<network_interface::NetworkInterface>,
    netlisteners: Vec<NetworkListener>,
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

    let netlisteners: Vec<NetworkListener> = networks
        .iter()
        .flat_map(|net| net.addr.iter().map(|addr| NetworkListener::new(addr)))
        .collect();

    let ac = AppCommon {
        networks,
        netlisteners,
    };

    let _e = multi_window.add(root_window, &event_loop);
    multi_window.run(event_loop, ac);
}
