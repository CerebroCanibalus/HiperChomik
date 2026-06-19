use std::path::Path;

mod ffi {
    #[repr(C)] pub struct SHFILEOPSTRUCTW {
        pub hwnd: *mut std::ffi::c_void, pub wFunc: u32,
        pub pFrom: *const u16, pub pTo: *const u16,
        pub fFlags: u16, pub fAnyOperationsAborted: i32,
        pub hNameMappings: *mut std::ffi::c_void,
        pub lpszProgressTitle: *const u16,
    }
    extern "system" {
        pub fn SHFileOperationW(lpFileOp: *mut SHFILEOPSTRUCTW) -> i32;
    }
}

pub fn send_to_bin(paths: &[String]) {
    if paths.is_empty() { return; }
    let mut s = String::new();
    for p in paths { s.push_str(p); s.push('\0'); }
    s.push('\0');
    let from: Vec<u16> = s.encode_utf16().collect();
    unsafe {
        let mut op = ffi::SHFILEOPSTRUCTW {
            hwnd: std::ptr::null_mut(), wFunc: 3, pFrom: from.as_ptr(), pTo: std::ptr::null(),
            fFlags: 0x0040 | 0x0200 | 0x0004, fAnyOperationsAborted: 0,
            hNameMappings: std::ptr::null_mut(), lpszProgressTitle: std::ptr::null(),
        };
        ffi::SHFileOperationW(&mut op);
    }
}

fn strip_readonly_recursive(path: &Path) {
    let mut perms = match std::fs::metadata(path) {
        Ok(m) => m.permissions(),
        Err(_) => return,
    };
    perms.set_readonly(false);
    let _ = std::fs::set_permissions(path, perms);
    if let Ok(rd) = std::fs::read_dir(path) {
        for e in rd.flatten() {
            strip_readonly_recursive(&e.path());
        }
    }
}

pub fn turbo_delete(paths: &[String]) -> Vec<String> {
    let mut errors = Vec::new();
    for p in paths {
        let path = Path::new(p);
        strip_readonly_recursive(path);
        let res = if path.is_dir() {
            std::fs::remove_dir_all(path)
        } else {
            std::fs::remove_file(path)
        };
        if let Err(e) = res {
            errors.push(format!("{}: {}", p, e));
        }
    }
    errors
}
