use std::path::PathBuf;
use windows::Win32::Foundation::HANDLE;

pub struct DiskLoad
{
    h: HANDLE,
}

impl DiskLoad {
    pub fn new(p: &PathBuf) -> Result<Self, i32> {
        let mut path = p.as_os_str().to_string_lossy().to_string();
        path.pop();
        let name = format!("\\\\.\\{}", path);
        println!("Disk name is {}", name);
        let h = unsafe { open_disk(name.as_ptr()) };
        if h != windows::Win32::Foundation::INVALID_HANDLE_VALUE {
            Ok(Self {
                h,
            })
        }
        else {
            Err(unsafe { get_last_error()})
        }
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<(), i32> {
        let b = buf.as_mut_ptr();
        let code = unsafe { read_from_disk(self.h, b, buf.len() as i64) };
        if code != 0 {
            Ok(())
        }
        else {
            Err(unsafe { get_last_error()})
        }
    }
}

extern "C" {
    pub fn open_disk(disk: *const u8) -> HANDLE;
    pub fn read_from_disk(h: HANDLE, buf: *mut u8, size: i64) -> i64;
    pub fn close_disk(h: HANDLE);
    pub fn get_last_error() -> i32;
}

