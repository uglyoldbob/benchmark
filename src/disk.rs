use std::io::Read;

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

    pub fn disk_read_all_files(p: &std::path::Path) -> Self {
        let (s, r) = std::sync::mpsc::channel();
        let (s2, r2) = std::sync::mpsc::channel();
        let p = p.to_owned();
        let p2 = p.clone();
        let thread = std::thread::spawn(move || {
            let mut running = false;

            let rd = std::fs::read_dir(&p);
            if let Ok(rd) = rd {
                let mut dirs: Vec<std::fs::ReadDir> = vec![rd];
                let mut curfile: Option<std::fs::File> = None;
                let mut bytes_read: u64 = 0;
                let mut read_buf: [u8; 100000] = [0; 100000];

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
                        if !dirs.is_empty() {
                            let index = dirs.len() - 1;
                            let rdir = &mut dirs[index];
                            if let Some(f) = &mut curfile {
                                let len = f.read(&mut read_buf);
                                if let Ok(length) = len {
                                    bytes_read += length as u64;
                                    if length == 0 {
                                        //println!("Read {} bytes", bytes_read);
                                        curfile = None;
                                    }
                                } else {
                                    //println!("Read {} bytes", bytes_read);
                                    curfile = None;
                                }
                            } else {
                                if let Some(Ok(a)) = rdir.next() {
                                    let path = a.path();
                                    if path.is_dir() && !path.is_symlink() {
                                        //println!(" dir: {}", path.display());
                                        let rd = std::fs::read_dir(path.clone());
                                        if let Ok(rd) = rd {
                                            dirs.push(rd);
                                        }
                                    } else if path.is_file() {
                                        curfile = std::fs::File::open(a.path()).ok();
                                        bytes_read = 0;
                                        //println!("File: {:?}", a.path());
                                    }
                                } else {
                                    dirs.pop();
                                }
                            }
                        } else {
                            //println!("Restarting");
                            dirs.push(std::fs::read_dir(&p).unwrap());
                        }
                    } else {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                }
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
