use std::io::{BufRead, SeekFrom, Seek};

pub struct DiskLoad {
    thread: std::thread::JoinHandle<()>,
    recv: std::sync::mpsc::Receiver<MessageFromDiskLoad>,
    pub send: std::sync::mpsc::Sender<MessageToDiskLoad>,
    pub performance: u64,
    pub running: bool,
    pub done: bool,
    pub path: std::path::PathBuf,
}

pub enum MessageToDiskLoad {
    Start,
    Stop,
    Exit,
}

pub enum MessageFromDiskLoad {
    Performance(u64),
    Running(bool),
    Done,
}

impl DiskLoad {
    pub fn process_messages(&mut self) {
        while let Ok(message) = self.recv.try_recv() {
            match message {
                MessageFromDiskLoad::Performance(bps) => {
                    self.performance = bps;
                }
                MessageFromDiskLoad::Running(r) => {
                    self.running = r;
                }
                MessageFromDiskLoad::Done => {
                    self.done = true;
                }
            }
        }
    }

    pub fn disk_read_all_files(p: &std::path::PathBuf) -> Self {
        let (s, r) = std::sync::mpsc::channel();
        let (s2, r2) = std::sync::mpsc::channel();
        let p = p.to_owned();
        let p2 = p.clone();
        let thread = std::thread::spawn(move || {
            let mut running = false;

            let clock = quanta::Clock::new();
            let mut buf = Box::new([0; 512000]);
            #[cfg(target_os = "windows")]
            let mut disk = rawdisk::DiskLoad::new(&p);
            #[cfg(target_os = "linux")]
            let mut disk = std::fs::File::open(&p);
            #[cfg(target_os = "windows")]
            while disk.is_err() {
                std::thread::sleep(std::time::Duration::from_millis(100));
                disk = rawdisk::DiskLoad::new(&p);
            }
            #[cfg(target_os = "linux")]
            while disk.is_err() {
                std::thread::sleep(std::time::Duration::from_millis(100));
                disk = std::fs::File::open(&p);
            }
            if let Ok(mut disk) = disk {
                #[cfg(target_os = "linux")]
                let mut br = std::io::BufReader::new(disk);
                println!("Successfully opened {}", p.display());
                'load: loop {
                    while let Ok(message) = r.try_recv() {
                        match message {
                            MessageToDiskLoad::Start => {
                                running = true;
                                if s2.send(MessageFromDiskLoad::Running(running)).is_err() {
                                    break 'load;
                                }
                            }
                            MessageToDiskLoad::Stop => {
                                running = false;
                                if s2.send(MessageFromDiskLoad::Running(running)).is_err() {
                                    break 'load;
                                }
                            }
                            MessageToDiskLoad::Exit => {
                                break 'load;
                            }
                        }
                    }
                    if running {
                        #[cfg(target_os = "windows")]
                        disk.read(buf.as_mut_slice());
                        #[cfg(target_os = "linux")]
                        let amt = if let Ok(b) = br.fill_buf() {
                            b.len()
                        } else {
                            0
                        };
                        #[cfg(target_os = "linux")]
                        if amt == 0 {
                            let _e = br.seek(SeekFrom::Start(0));
                        }
                        #[cfg(target_os = "linux")]
                        br.consume(amt);
                    } else {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                }
            } else {
                println!("Failed to open {}", p.display());
            }
            let _e = s2.send(MessageFromDiskLoad::Done);
        });
        Self {
            thread,
            recv: r2,
            send: s,
            performance: 0,
            running: false,
            done: false,
            path: p2,
        }
    }
}
