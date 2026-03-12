#![no_std]
#![no_main]

use kos::fs::{FileInfo, fs_read};

static FILENAME: &[u8] = b"/rd/1/readme.txt\0";
static mut BUFFER: [u8; 256] = [0u8; 256];

#[no_mangle]
pub extern "C" fn _start() -> ! {
    unsafe {
        let info = FileInfo {
            func:   0,
            param1: 0,
            param2: 0,
            param3: 256,
            buf:    BUFFER.as_mut_ptr(),
            name:   FILENAME.as_ptr(),
        };

        let (_result, _bytes_read) = fs_read(&info as *const FileInfo);
    }

    loop {}
}
