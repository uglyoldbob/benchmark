fn main() {
    println!("Running test program");
    let p: std::path::PathBuf = std::path::PathBuf::from("C:\\");
    let disk = rawdisk::DiskLoad::new(&p);
    if let Ok(mut disk) = disk {
        println!("Success");
        let mut buf = [0; 512000];
        println!("Address of buf is {:p}", &buf);
        loop {
            if let Err(err) = disk.read(&mut buf) {
                println!("Error reading {}", err);
                std::thread::sleep(std::time::Duration::from_millis(1000));
            }
        }
    }
}
