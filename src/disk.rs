use std::io::{BufRead, Read};

trait ReadEof: BufRead {
    fn read_eof(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        let mut read = 0;
        loop {
            let (done, used) = {
                let available = match self.fill_buf() {
                    Ok(n) => n,
                    Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(e) => return Err(e),
                };
                buf.extend_from_slice(available);
                (false, available.len())
            };
            self.consume(used);
            read += used;
            if done || used == 0 {
                return Ok(read);
            }
        }
    }
}

struct EofRead<T> {
    buf: std::io::BufReader<T>,
}

impl<T: Read> EofRead<T> {
    fn new(b: T) -> Self {
        Self {
            buf: std::io::BufReader::new(b),
        }
    }
}

impl<T: Read> Read for EofRead<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.buf.read(buf)
    }
}

impl<T: Read> BufRead for EofRead<T> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        self.buf.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.buf.consume(amt)
    }
}

impl<T: Read> ReadEof for EofRead<T> {}

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

            let clock = quanta::Clock::new();
            let mut buf = Box::new([0; 512000]);
            let mut disk = rawdisk::DiskLoad::new(&p);
            if let Ok(mut disk) = disk {
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
                        disk.read(buf.as_mut_slice());
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
