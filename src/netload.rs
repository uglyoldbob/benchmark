use std::{
    net::{SocketAddr, UdpSocket},
    time::Duration,
};

pub struct NetworkLoad {
    thread: std::thread::JoinHandle<()>,
    recv: std::sync::mpsc::Receiver<MessageFromNetworkLoad>,
    pub send: std::sync::mpsc::Sender<MessageToNetworkLoad>,
    pub ready: bool,
    pub running: bool,
    pub done: bool,
    pub addr: network_interface::Addr,
    pub server: Option<SocketAddr>,
}

pub enum MessageFromNetworkLoad {
    Ready(bool),
    Running(bool),
    Server(Option<SocketAddr>),
    Done,
}

pub enum MessageToNetworkLoad {
    Start,
    Stop,
    Exit,
}

impl NetworkLoad {
    pub fn process_messages(&mut self) {
        while let Ok(message) = self.recv.try_recv() {
            match message {
                MessageFromNetworkLoad::Ready(l) => {
                    self.ready = l;
                }
                MessageFromNetworkLoad::Running(r) => {
                    self.running = r;
                }
                MessageFromNetworkLoad::Done => {
                    self.done = true;
                }
                MessageFromNetworkLoad::Server(s) => {
                    self.server = s;
                }
            }
        }
    }

    pub fn new(addr: &network_interface::Addr) -> Self {
        let addr = addr.to_owned();
        let (s, r) = std::sync::mpsc::channel();
        let (s2, r2) = std::sync::mpsc::channel();
        let thread = std::thread::spawn(move || {
            let mut running = false;
            let mut socket: Option<UdpSocket> = None;
            let mut buf_broad: [u8; 1000] = [0; 1000];
            let mut buf: [u8; 10000] = [0; 10000];
            let mut server_address = None;

            let s = match addr {
                network_interface::Addr::V4(a) => UdpSocket::bind((a.ip, 5002)),
                network_interface::Addr::V6(a) => UdpSocket::bind((a.ip, 5002)),
            };
            if let Ok(r) = s {
                let broad = r.set_broadcast(true);
                if broad.is_ok() {
                    buf_broad[0] = b'A';
                    match addr {
                        network_interface::Addr::V4(a) => {
                            if let Some(addr) = a.broadcast {
                                if let Err(e) = r.send_to(&buf_broad, (addr, 5003)) {
                                    println!("Error broadcast {}", e);
                                } else {
                                    println!("Broadcast to {}", addr);
                                }
                            }
                        }
                        network_interface::Addr::V6(a) => {
                            if let Some(addr) = a.broadcast {
                                if let Err(e) = r.send_to(&buf_broad, (addr, 5003)) {
                                    println!("Error broadcast {}", e);
                                } else {
                                    println!("Broadcast to {}", addr);
                                }
                            }
                        }
                    };
                    if let Ok((size, addr)) = r.recv_from(&mut buf) {
                        println!("Received a response from {}", addr);
                        server_address = Some(addr);
                        s2.send(MessageFromNetworkLoad::Server(server_address));
                    }
                    socket = Some(r);
                }
            }
            if s2.send(MessageFromNetworkLoad::Ready(true)).is_err() {
                socket = None;
            }
            if server_address.is_none() {
                socket = None;
            }
            if let Some(sock) = socket {
                'main: loop {
                    while let Ok(message) = r.try_recv() {
                        match message {
                            MessageToNetworkLoad::Start => {
                                running = true;
                                if s2.send(MessageFromNetworkLoad::Running(running)).is_err() {
                                    break 'main;
                                }
                            }
                            MessageToNetworkLoad::Stop => {
                                running = false;
                                if s2.send(MessageFromNetworkLoad::Running(running)).is_err() {
                                    break 'main;
                                }
                            }
                            MessageToNetworkLoad::Exit => break 'main,
                        }
                    }
                    if running {
                        if let Some(a) = server_address {
                            sock.send_to(&buf_broad, a);
                        }
                    } else {
                        std::thread::sleep(Duration::from_millis(100));
                    }
                }
            }
            let _e = s2.send(MessageFromNetworkLoad::Done);
        });
        Self {
            thread,
            recv: r2,
            send: s,
            ready: false,
            running: false,
            done: false,
            addr,
            server: None,
        }
    }
}
