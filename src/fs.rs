use core::arch::asm;

/// KolibriOS Function 70 — filesystem operations
/// Passes a pointer to a FileInfo structure via ebx
/// int 0x40 with eax=70 is the KolibriOS FS syscall

#[repr(C, packed)]
pub struct FileInfo {
    pub func:       u32,   // 0 = read, 1 = read folder, 2 = create/write
    pub param1:     u32,   // offset (low)
    pub param2:     u32,   // offset (high) or flags
    pub param3:     u32,   // bytes to read/write
    pub buf:        *mut u8, // pointer to buffer
    pub name:       *const u8, // pointer to filename (ASCIIZ)
}

/// Opens/reads a file on KolibriOS using syscall eax=70
/// Returns (eax_result, ebx_bytes_read)
pub unsafe fn fs_read(info: *const FileInfo) -> (u32, u32) {
    let eax_out: u32;
    let ebx_out: u32;
    unsafe {
        asm!(
            "int 0x40",
            inlateout("eax") 70u32 => eax_out,
            inlateout("ebx") info as u32 => ebx_out,
            options(nostack)
        );
    }
    (eax_out, ebx_out)
}
