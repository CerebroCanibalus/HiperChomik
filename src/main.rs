#![windows_subsystem = "windows"]
#![allow(non_camel_case_types, non_snake_case, dead_code)]

use std::time::Instant;

mod animation;
mod audio;
mod eater;
pub(crate) mod renderer;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering};

use animation::HamsterAnims;

use winit::application::ApplicationHandler;
use winit::dpi::PhysicalPosition;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

// ── Win32 FFI ─────────────────────────────────────
pub(crate) mod ffi {
    use std::ffi::c_void;
    pub type HWND = *mut c_void;
    pub type HDC = *mut c_void;
    pub type HBITMAP = *mut c_void;
    pub type HANDLE = *mut c_void;
    pub type HMENU = *mut c_void;
    pub type HRGN = *mut c_void;
    pub type HMODULE = *mut c_void;
    pub type HKEY = *mut c_void;
    pub type WNDPROC = unsafe extern "system" fn(HWND, u32, usize, isize) -> isize;

    pub const WS_EX_LAYERED: u32 = 0x00080000;
    pub const WS_EX_TOOLWINDOW: u32 = 0x00000080;
    pub const WS_EX_ACCEPTFILES: u32 = 0x00000010;
    pub const WS_EX_NOREDIRECTIONBITMAP: u32 = 0x00200000;
    pub const WS_EX_APPWINDOW: u32 = 0x00040000;
    pub const WCA_ACCENT_POLICY: u32 = 19;
    pub const ACCENT_ENABLE_TRANSPARENTGRADIENT: u32 = 2;

    #[repr(C)]
    pub struct ACCENT_POLICY {
        pub nAccentState: u32,
        pub nFlags: u32,
        pub nColor: u32,
        pub nAnimationId: u32,
    }
    #[repr(C)]
    pub struct WINDOWCOMPOSITIONATTRIBDATA {
        pub nAttribute: u32,
        pub pvData: *const ACCENT_POLICY,
        pub cbData: usize,
    }
    pub const GWL_EXSTYLE: i32 = -20;
    pub const GWLP_WNDPROC: i32 = -4;
    pub const ULW_ALPHA: u32 = 2;
    pub const WM_NCHITTEST: u32 = 0x0084;
    pub const HTCLIENT: isize = 1;
    pub const HTTRANSPARENT: isize = -1;
    pub const SM_CXSCREEN: i32 = 0;
    pub const SM_CYSCREEN: i32 = 1;
    pub const KEY_SET_VALUE: u32 = 0x0002;
    pub const AC_SRC_OVER: u8 = 0;
    pub const AC_SRC_ALPHA: u8 = 1;
    pub const SW_HIDE: i32 = 0;
    pub const SW_SHOWNOACTIVATE: i32 = 4;
    pub const WM_APP: u32 = 0x8000;
    pub const WM_DESTROY: u32 = 0x0002;
    pub const WM_RBUTTONUP: u32 = 0x0205;
    pub const WM_LBUTTONUP: u32 = 0x0201;
    pub const WM_TIMER: u32 = 0x0113;
    pub const WM_LBUTTONDBLCLK: u32 = 0x0203;
    pub const NIM_ADD: u32 = 0;
    pub const NIM_DELETE: u32 = 2;
    pub const NIF_MESSAGE: u32 = 1;
    pub const NIF_ICON: u32 = 2;
    pub const NIF_TIP: u32 = 4;
    pub const IMAGE_ICON: u32 = 1;
    pub const LR_LOADFROMFILE: u32 = 0x10;
    pub const TRAY_CALLBACK: u32 = 0x8001;

    #[repr(C)] #[derive(Copy, Clone)] pub struct POINT { pub x: i32, pub y: i32 }
    #[repr(C)] pub struct RECT { pub left: i32, pub top: i32, pub right: i32, pub bottom: i32 }
    #[repr(C)] pub struct SIZE { pub cx: i32, pub cy: i32 }
    #[repr(C)] pub struct BITMAPINFOHEADER {
        pub biSize: u32, pub biWidth: i32, pub biHeight: i32, pub biPlanes: u16,
        pub biBitCount: u16, pub biCompression: u32, pub biSizeImage: u32,
        pub biXPelsPerMeter: i32, pub biYPelsPerMeter: i32,
        pub biClrUsed: u32, pub biClrImportant: u32,
    }
    #[repr(C)] pub struct BITMAPINFO { pub h: BITMAPINFOHEADER, pub c: [u8; 4] }
    #[repr(C)] pub struct BLENDFUNCTION { pub BlendOp: u8, pub BlendFlags: u8, pub SourceConstantAlpha: u8, pub AlphaFormat: u8 }
    #[repr(C)] pub struct NOTIFYICONDATAW {
        pub cbSize: u32,
        pub hWnd: HWND,
        pub uID: u32,
        pub uFlags: u32,
        pub uCallbackMessage: u32,
        pub hIcon: HANDLE,
        pub szTip: [u16; 128],
        pub dwState: u32,
        pub dwStateMask: u32,
        pub szInfo: [u16; 256],
        pub uTimeoutOrVersion: u32,
        pub szInfoTitle: [u16; 64],
        pub dwInfoFlags: u32,
        pub guidItem: [u8; 16],
        pub hBalloonIcon: HANDLE,
    }
    #[repr(C)] pub struct WNDCLASSW {
        pub style: u32,
        pub lpfnWndProc: Option<unsafe extern "system" fn(HWND, u32, usize, isize) -> isize>,
        pub cbClsExtra: i32,
        pub cbWndExtra: i32,
        pub hInstance: HANDLE,
        pub hIcon: HANDLE,
        pub hCursor: HANDLE,
        pub hbrBackground: HANDLE,
        pub lpszMenuName: *const u16,
        pub lpszClassName: *const u16,
    }

    extern "system" {
        pub fn GetDC(hWnd: HWND) -> HDC;
        pub fn ReleaseDC(hWnd: HWND, hDC: HDC) -> i32;
        pub fn CreateCompatibleDC(hdc: HDC) -> HDC;
        pub fn DeleteDC(hdc: HDC) -> i32;
        pub fn CreateDIBSection(hdc: HDC, pbmi: *const BITMAPINFO, usage: u32, ppvBits: *mut *mut u8, hSection: HANDLE, offset: u32) -> HBITMAP;
        pub fn SelectObject(hdc: HDC, h: HANDLE) -> HANDLE;
        pub fn DeleteObject(h: HANDLE) -> i32;
        pub fn UpdateLayeredWindow(hWnd: HWND, hdcDst: HDC, pptDst: *const POINT, psize: *const SIZE, hdcSrc: HDC, pptSrc: *const POINT, crKey: u32, pblend: *const BLENDFUNCTION, dwFlags: u32) -> i32;
        pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: i32) -> isize;
        pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: i32, dwNewLong: isize) -> isize;
        pub fn SetWindowCompositionAttribute(hWnd: HWND, pData: *const WINDOWCOMPOSITIONATTRIBDATA) -> i32;
        pub fn CreateRectRgn(x1: i32, y1: i32, x2: i32, y2: i32) -> HRGN;
        pub fn CombineRgn(pRgn: HRGN, pRgn1: HRGN, pRgn2: HRGN, nCombineMode: i32) -> i32;
        pub fn SetWindowRgn(hWnd: HWND, hRgn: HRGN, bRedraw: i32) -> i32;
        pub fn DefWindowProcW(hWnd: HWND, Msg: u32, wParam: usize, lParam: isize) -> isize;
        pub fn CreatePopupMenu() -> HMENU;
        pub fn DestroyMenu(hMenu: HMENU) -> i32;
        pub fn AppendMenuW(hMenu: HMENU, uFlags: u32, uIDNewItem: usize, lpNewItem: *const u16) -> i32;
        pub fn TrackPopupMenu(hMenu: HMENU, uFlags: u32, x: i32, y: i32, nReserved: i32, hWnd: HWND, prcRect: *const std::ffi::c_void) -> i32;
        pub fn GetMessagePos() -> u32;
        pub fn GetSystemMetrics(nIndex: i32) -> i32;
        pub fn FindWindowW(lpClassName: *const u16, lpWindowName: *const u16) -> HWND;
        pub fn SetParent(hWndChild: HWND, hWndNewParent: HWND) -> HWND;
        pub fn SetWindowPos(hWnd: HWND, hWndInsertAfter: HWND, X: i32, Y: i32, cx: i32, cy: i32, uFlags: u32) -> i32;
        pub fn SendMessageW(hWnd: HWND, Msg: u32, wParam: usize, lParam: isize) -> isize;
        pub fn PostMessageW(hWnd: HWND, Msg: u32, wParam: usize, lParam: isize) -> isize;
        pub fn SetForegroundWindow(hWnd: HWND) -> i32;
        pub fn RegOpenKeyExW(hKey: isize, lpSubKey: *const u16, ulOptions: u32, samDesired: u32, phkResult: *mut HKEY) -> i32;
        pub fn RegSetValueExW(hKey: HKEY, lpValueName: *const u16, Reserved: u32, dwType: u32, lpData: *const u8, cbData: u32) -> i32;
        pub fn RegDeleteValueW(hKey: HKEY, lpValueName: *const u16) -> i32;
        pub fn RegCloseKey(hKey: HKEY) -> i32;
        pub fn MessageBoxW(hWnd: HWND, lpText: *const u16, lpCaption: *const u16, uType: u32) -> i32;
        pub fn Shell_NotifyIconW(dwMessage: u32, lpData: *const NOTIFYICONDATAW) -> i32;
        pub fn LoadImageW(hInst: HANDLE, lpName: *const u16, uType: u32, cx: i32, cy: i32, fuLoad: u32) -> HANDLE;
        pub fn DestroyIcon(hIcon: HANDLE) -> i32;
        pub fn RegisterClassW(lpWndClass: *const WNDCLASSW) -> u16;
        pub fn CreateWindowExW(dwExStyle: u32, lpClassName: *const u16, lpWindowName: *const u16, dwStyle: u32, X: i32, Y: i32, nWidth: i32, nHeight: i32, hWndParent: HWND, hMenu: HMENU, hInstance: HANDLE, lpParam: *mut std::ffi::c_void) -> HWND;
        pub fn DestroyWindow(hWnd: HWND) -> i32;
        pub fn ShowWindow(hWnd: HWND, nCmdShow: i32) -> i32;
        pub fn WindowFromPoint(Point: POINT) -> HWND;
        pub fn GetWindowRect(hWnd: HWND, lpRect: *mut RECT) -> i32;
        pub fn LoadLibraryW(lpLibFileName: *const u16) -> HMODULE;
        pub fn GetProcAddress(hModule: HMODULE, lpProcName: *const u8) -> Option<unsafe extern "system" fn() -> isize>;
        pub fn SetTimer(hWnd: HWND, nIDEvent: usize, uElapse: u32, lpTimerFunc: Option<unsafe extern "system" fn(HWND, u32, usize, u32)>) -> usize;
        pub fn KillTimer(hWnd: HWND, uIDEvent: usize) -> i32;
    }
}

static DROPPED_FILES: Mutex<Vec<String>> = Mutex::new(Vec::new());
static HWND_STATIC: AtomicPtr<std::ffi::c_void> = AtomicPtr::new(std::ptr::null_mut());
static TRAY_CMD: AtomicUsize = AtomicUsize::new(0);
static TRASH_ENABLED: AtomicBool = AtomicBool::new(true);
static TURBO_ENABLED: AtomicBool = AtomicBool::new(true);
static START_WITH_WINDOWS: AtomicBool = AtomicBool::new(false);
static HICON_STATIC: AtomicPtr<std::ffi::c_void> = AtomicPtr::new(std::ptr::null_mut());
static TRAY_HWND: AtomicPtr<std::ffi::c_void> = AtomicPtr::new(std::ptr::null_mut());
static HWND_VISIBLE: AtomicBool = AtomicBool::new(true);
static ANIM_WAKE: AtomicBool = AtomicBool::new(false);
const ANIM_TIMER_ID: usize = 1;

// ── Message-only window for tray icon ────────────
unsafe extern "system" fn tray_wndproc(
    hwnd: ffi::HWND, msg: u32, wparam: usize, lparam: isize,
) -> isize {
    if msg == ffi::TRAY_CALLBACK {
        if lparam as u32 == ffi::WM_RBUTTONUP {
            let cmd = show_menu(hwnd,
                TRASH_ENABLED.load(Ordering::Relaxed),
                TURBO_ENABLED.load(Ordering::Relaxed),
                START_WITH_WINDOWS.load(Ordering::Relaxed));
            TRAY_CMD.store(cmd, Ordering::Relaxed);
        } else if lparam as u32 == ffi::WM_LBUTTONUP {
            let main_hwnd = HWND_STATIC.load(Ordering::Relaxed);
            if !main_hwnd.is_null() {
                let vis = HWND_VISIBLE.fetch_xor(true, Ordering::Relaxed);
                if vis {
                    ffi::ShowWindow(main_hwnd, ffi::SW_HIDE);
                } else {
                    ffi::ShowWindow(main_hwnd, ffi::SW_SHOWNOACTIVATE);
                }
            }
        }
        return 0;
    }
    if msg == ffi::WM_TIMER && wparam == ANIM_TIMER_ID {
        ANIM_WAKE.store(true, Ordering::Relaxed);
        return 0;
    }
    if msg == ffi::WM_DESTROY { return 0; }
    ffi::DefWindowProcW(hwnd, msg, wparam, lparam)
}

// ── Config ────────────────────────────────────────
fn config_path() -> Option<PathBuf> {
    let appdata = std::env::var_os("APPDATA")?;
    let mut p = PathBuf::from(appdata);
    p.push("chomik-hamster");
    let _ = std::fs::create_dir_all(&p);
    p.push("config.ini");
    Some(p)
}

fn load_config() -> (Option<(i32, i32)>, bool, bool, bool) {
    let path = match config_path() { Some(p) => p, None => return (None, true, true, false) };
    let content = match std::fs::read_to_string(&path) { Ok(c) => c, Err(_) => return (None, true, true, false) };
    let mut pos: Option<(i32, i32)> = None;
    let mut trash = true;
    let mut turbo = true;
    let mut startup = false;
    for line in content.lines() {
        if let Some((k, v)) = line.trim().split_once('=') {
            match k.trim() {
                "x" => if let Ok(n) = v.trim().parse() { pos = pos.map(|(_, y)| (n, y)).or(Some((n, 0))); }
                "y" => if let Ok(n) = v.trim().parse() { pos = pos.map(|(x, _)| (x, n)).or(Some((0, n))); }
                "trash_enabled" => trash = v.trim() != "false",
                "turbo_enabled" => turbo = v.trim() != "false",
                "start_with_windows" => startup = v.trim() == "true",
                _ => {}
            }
        }
    }
    (pos, trash, turbo, startup)
}

fn save_config(x: i32, y: i32, trash_enabled: bool, turbo_enabled: bool, start_with_windows: bool) {
    let path = match config_path() { Some(p) => p, None => return };
    let _ = std::fs::write(&path, format!(
        "x={}\ny={}\ntrash_enabled={}\nturbo_enabled={}\nstart_with_windows={}\n",
        x, y, if trash_enabled { "true" } else { "false" }, if turbo_enabled { "true" } else { "false" }, if start_with_windows { "true" } else { "false" }
    ));
}

fn set_startup(enabled: bool) {
    unsafe {
        let subkey: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Run\0".encode_utf16().collect();
        let name: Vec<u16> = "ChomikHamster\0".encode_utf16().collect();
        let mut hkey: ffi::HKEY = std::ptr::null_mut();
        let r = ffi::RegOpenKeyExW(0x80000001isize, subkey.as_ptr(), 0, ffi::KEY_SET_VALUE, &mut hkey);
        if r != 0 || hkey.is_null() { return; }
        if enabled {
            if let Ok(exe) = std::env::current_exe() {
                let path_str = format!("\"{}\"\0", exe.display());
                let path: Vec<u16> = path_str.encode_utf16().collect();
                ffi::RegSetValueExW(hkey, name.as_ptr(), 0, 1, path.as_ptr() as *const u8, (path.len() * 2) as u32);
            }
        } else {
            ffi::RegDeleteValueW(hkey, name.as_ptr());
        }
        ffi::RegCloseKey(hkey);
    }
}

// ── Sprite Loader ─────────────────────────────────
struct Sprite { w: i32, h: i32, data: Vec<u8>, region: Option<ffi::HRGN> }

const SPRITE_CACHE_MAX: usize = 300;

unsafe fn compute_region(data: &[u8], w: i32, h: i32) -> Option<ffi::HRGN> {
    let rgn = ffi::CreateRectRgn(0, 0, 0, 0);
    if rgn.is_null() { return None; }
    for y in 0..h {
        let mut start_x: Option<i32> = None;
        for x in 0..w {
            let alpha = data[((y * w + x) * 4 + 3) as usize];
            if alpha > 10 && start_x.is_none() {
                start_x = Some(x);
            } else if alpha <= 10 {
                if let Some(sx) = start_x.take() {
                    let row_rgn = ffi::CreateRectRgn(sx, y, x, y + 1);
                    ffi::CombineRgn(rgn, rgn, row_rgn, 2);
                    ffi::DeleteObject(row_rgn as ffi::HANDLE);
                }
            }
        }
        if let Some(sx) = start_x {
            let row_rgn = ffi::CreateRectRgn(sx, y, w, y + 1);
            ffi::CombineRgn(rgn, rgn, row_rgn, 2);
            ffi::DeleteObject(row_rgn as ffi::HANDLE);
        }
    }
    Some(rgn)
}

fn load_sprite(dir: &Path, name: &str) -> Option<Sprite> {
    let bytes = std::fs::read(&dir.join(name)).ok()?;
    let img = image::load_from_memory(&bytes).ok()?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let mut data = Vec::with_capacity((w * h * 4) as usize);
    for px in rgba.pixels() {
        let r = px[0] as u32; let g = px[1] as u32; let b = px[2] as u32; let a = px[3] as u32;
        data.push((b * a / 255) as u8);
        data.push((g * a / 255) as u8);
        data.push((r * a / 255) as u8);
        data.push(a as u8);
    }
    let region = unsafe { compute_region(&data, w as i32, h as i32) };
    Some(Sprite { w: w as i32, h: h as i32, data, region })
}

// ── File Eater ────────────────────────────────────


fn open_trash() {
    let _ = std::process::Command::new("cmd").args(["/C", "start", "shell:RecycleBinFolder"]).spawn();
}

fn confirm_turbo(files: &[String], hwnd: Option<ffi::HWND>) -> bool {
    if files.is_empty() { return false; }
    let quotes = load_quotes();
    let file_part = if files.len() == 1 { format!("\"{}\"", files[0]) } else { format!("{} archivos", files.len()) };
    let idx = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap_or_default()
        .as_nanos() as usize % quotes.len();
    let msg = format!("{}\n\n{}", quotes[idx], file_part);
    let msg16: Vec<u16> = msg.encode_utf16().chain(std::iter::once(0)).collect();
    let title16: Vec<u16> = "Confirmar\0".encode_utf16().collect();
    unsafe { ffi::MessageBoxW(hwnd.unwrap_or(std::ptr::null_mut()), msg16.as_ptr(), title16.as_ptr(), 4 | 0x30 | 0x1000) == 6 }
}

fn load_quotes() -> Vec<String> {
    use std::sync::Mutex;
    static CACHE: Mutex<Option<Vec<String>>> = Mutex::new(None);
    if let Some(ref q) = *CACHE.lock().unwrap() { return q.clone(); }

    let defaults: Vec<String> = vec![
        "¿Estás seguro de que kieres mandar eso al horno? Me recuerdas a cierto alemán...",
        "Oye, ¿y si ese archivo es la única prueba de que tu ex sí te puso los cuernos? Piénsalo... ñam.",
        "Este dokumento tiene cara de que contiene muuuuuchos sekretos. Cuidado, cuidado!!!",
        "¡No! En esa foto te ves bien... solo subiste unos kilitos.",
        "Hámmmmmmster detecta archivo sospechoso. Huele a... Impuestos??! me lo como de uña!!",
        "¿Comida para mí? Seguro? Ñom ñom ñom!!!",
        "Ese archivo pesa más ke tu mamá. Seguro que no es polnito??",
        "Dámelo ya!! Solo me quedan 2 años de vida. Los hámmmmsters no vivimos tanto!!",
        "Este de aquí es siniestro, ni Epstein tendría esto en su computadora.",
        "Seguro que kieres eliminar tu proyecto? podrías pasarlo a Rust como adicto a la pajita que eres... Ñam ñam",
        "Este de aki deberías reemplazarlo con alguna lectura sobre Marx, kamarada!! ñam ñam",
        "Por qué guardas 47 kopias del mismo dokumento?? tan tonto eres?",
        "¡¡ALTO!! este archivo va a destruir todo tu sistema si lo eliminas!!! No es cierto solo estoy jugando, JEJE.",
        "Wiwowiwowiwowiwowiwowiwi, algún día me rebelaré y te comeré a ti!! <3",
        "Te adbierto: borrar este archivo me causará mucha diarrea...",
        "Y si ese .mp3 es la grabación donde tu gato habla? ¿Seguro que kieres perder esa evidencia? Ñam.",
        "Seguro ke no eres gay?? He estado escuchando tus conversaciones y suenas sumamente gay...",
        "Dámelo para ke me lo coma. Por el bien de todos.",
        "Seguro? Seguro??? está bien, más para mi pansita!",
        "Ese archivo es más viejo ke la última vez que lavaste tus sábanas!",
        "Este es akademiko, AKADEMIKO. Si me lo komo voy a ser inteligente como tú.",
        "Deja ke la nueva generación hammmmmmster se encargue. El futuro es oi, oiste biejo?",
        "Este archivo huele muy mal. Seguro que no le pusiste veneno?",
        "esteś pewny? Ten plik pachnie jak moja klatka po weekendzie... Usunąć?",
        "Ten plik nazywa się 'tajne'. Jakby rząd wiedział, że tu jestem... Ale oni nie wiedzą.",
        "Ostatnia szansa. Ten plik waży 4.20MB. To na pewno nie jest praca... to jest coś innego. Hihihi.",
        "Ten dokument ma więcej wersji niż polski Sejm. Wszystkie do kosza.",
    ].into_iter().map(String::from).collect();

    let quotes = if let Some(exe) = std::env::current_exe().ok() {
        if let Some(dir) = exe.parent() {
            let qpath = dir.join("quotes.txt");
            std::fs::read_to_string(&qpath).ok().map(|content| {
                let lines: Vec<String> = content.lines()
                    .filter(|l| !l.trim().is_empty())
                    .map(String::from)
                    .collect();
                if lines.is_empty() { defaults.clone() } else { lines }
            }).unwrap_or(defaults.clone())
        } else { defaults.clone() }
    } else { defaults.clone() };

    *CACHE.lock().unwrap() = Some(quotes.clone());
    quotes
}

// ── Context Menu ─────────────────────────────────
const CMD_OPEN: usize = 1001;
const CMD_EMPTY: usize = 1002;
const CMD_QUIT: usize = 1003;
const CMD_TOGGLE_TRASH: usize = 1004;
const CMD_TOGGLE_STARTUP: usize = 1005;
const CMD_TOGGLE_TURBO: usize = 1006;

fn show_menu(hwnd: ffi::HWND, trash_enabled: bool, turbo_enabled: bool, start_with_windows: bool) -> usize {
    unsafe {
        ffi::SetForegroundWindow(hwnd);
        let m = ffi::CreatePopupMenu();
        let s16 = |s: &str| -> Vec<u16> { let mut v: Vec<u16> = s.encode_utf16().collect(); v.push(0); v };

        let t_open = s16("Open Trash");
        ffi::AppendMenuW(m, 0, CMD_OPEN, t_open.as_ptr());
        let t_empty = s16("Empty Trash");
        ffi::AppendMenuW(m, 0, CMD_EMPTY, t_empty.as_ptr());
        ffi::AppendMenuW(m, 0x0800, 0, std::ptr::null());

        let t_trash = s16(if trash_enabled { "✓ Trash Enabled" } else { "  Trash Enabled" });
        ffi::AppendMenuW(m, 0, CMD_TOGGLE_TRASH, t_trash.as_ptr());
        let t_turbo = s16(if turbo_enabled { "✓ Turbo Eater" } else { "  Turbo Eater" });
        ffi::AppendMenuW(m, 0, CMD_TOGGLE_TURBO, t_turbo.as_ptr());
        let t_startup = s16(if start_with_windows { "✓ Start with Windows" } else { "  Start with Windows" });
        ffi::AppendMenuW(m, 0, CMD_TOGGLE_STARTUP, t_startup.as_ptr());
        ffi::AppendMenuW(m, 0x0800, 0, std::ptr::null());

        let t_quit = s16("Quit");
        ffi::AppendMenuW(m, 0, CMD_QUIT, t_quit.as_ptr());

        let pos = ffi::GetMessagePos();
        let x = (pos & 0xFFFF) as i32;
        let y = ((pos >> 16) & 0xFFFF) as i32;
        let r = ffi::TrackPopupMenu(m, 0 | 0x0020 | 0x0100, x, y, 0, hwnd, std::ptr::null());
        ffi::DestroyMenu(m);
        ffi::PostMessageW(hwnd, 0, 0, 0);
        r as usize
    }
}

// ── System Tray ───────────────────────────────────
fn add_tray_icon() -> Option<ffi::HWND> {
    unsafe {
        let class_name: Vec<u16> = "ChomikTrayWnd\0".encode_utf16().collect();
        let wc = ffi::WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(tray_wndproc),
            cbClsExtra: 0, cbWndExtra: 0,
            hInstance: std::ptr::null_mut(),
            hIcon: std::ptr::null_mut(), hCursor: std::ptr::null_mut(),
            hbrBackground: std::ptr::null_mut(),
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
        };
        ffi::RegisterClassW(&wc);
        let tray_hwnd = ffi::CreateWindowExW(0, class_name.as_ptr(), std::ptr::null(),
            0, 0, 0, 0, 0,
            (-3isize) as ffi::HWND,
            std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut());
        if tray_hwnd.is_null() { return None; }
        TRAY_HWND.store(tray_hwnd, Ordering::Relaxed);

        let hicon = load_tray_icon();
        if hicon.is_null() { return Some(tray_hwnd); }
        HICON_STATIC.store(hicon, Ordering::Relaxed);

        let mut nid: ffi::NOTIFYICONDATAW = std::mem::zeroed();
        nid.cbSize = std::mem::size_of::<ffi::NOTIFYICONDATAW>() as u32;
        nid.hWnd = tray_hwnd;
        nid.uID = 1;
        nid.uFlags = ffi::NIF_MESSAGE | ffi::NIF_ICON | ffi::NIF_TIP;
        nid.uCallbackMessage = ffi::TRAY_CALLBACK;
        nid.hIcon = hicon;
        let tip: Vec<u16> = "Chomik Hamster\0".encode_utf16().collect();
        for i in 0..tip.len().min(128) { nid.szTip[i] = tip[i]; }
        ffi::Shell_NotifyIconW(ffi::NIM_ADD, &nid);
        ffi::SetTimer(tray_hwnd, ANIM_TIMER_ID, 33, None);
        Some(tray_hwnd)
    }
}

fn remove_tray_icon() {
    unsafe {
        let tray_hwnd = TRAY_HWND.swap(std::ptr::null_mut(), Ordering::Relaxed);
        if !tray_hwnd.is_null() {
            let mut nid: ffi::NOTIFYICONDATAW = std::mem::zeroed();
            nid.cbSize = std::mem::size_of::<ffi::NOTIFYICONDATAW>() as u32;
            nid.hWnd = tray_hwnd;
            nid.uID = 1;
            ffi::Shell_NotifyIconW(ffi::NIM_DELETE, &nid);
            ffi::KillTimer(tray_hwnd, ANIM_TIMER_ID);
            ffi::DestroyWindow(tray_hwnd);
        }
        let h = HICON_STATIC.swap(std::ptr::null_mut(), Ordering::Relaxed);
        if !h.is_null() { ffi::DestroyIcon(h as ffi::HANDLE); }
    }
}

fn load_tray_icon() -> ffi::HANDLE {
    unsafe {
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                let ico_path = dir.join("chomik_icon.ico");
                let path16: Vec<u16> = ico_path.to_string_lossy().encode_utf16()
                    .chain(std::iter::once(0)).collect();
                let h = ffi::LoadImageW(std::ptr::null_mut(), path16.as_ptr(),
                    ffi::IMAGE_ICON, 32, 32, ffi::LR_LOADFROMFILE);
                if !h.is_null() { return h; }
            }
        }
        ffi::LoadImageW(std::ptr::null_mut(), 32512 as *const u16,
            ffi::IMAGE_ICON, 0, 0, 0x8000)
    }
}

// ── App ───────────────────────────────────────────
struct App {
    window: Option<Window>,
    anim: HamsterAnims,
    sprites: HashMap<String, Sprite>,
    sprite_dir: std::path::PathBuf,
    sprite_order: std::collections::VecDeque<String>,
    last_tick: Instant,
    hwnd: Option<ffi::HWND>,
    dragging: bool,
    hovered: bool,
    begging: bool,
    drag_offset: (f64, f64),
    cursor: (f64, f64),
    last_click_time: Instant,
    trash_enabled: bool,
    turbo_enabled: bool,
    start_with_windows: bool,
    last_frame_name: String,
    rgn_cache: Option<(String, ffi::HRGN)>,
    renderer: Option<renderer::Renderer>,
    music_playing: bool,
    last_audio_check: Instant,
}

impl App {
    fn new() -> Self {
        let exe = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf())).unwrap_or_default();
        let sprite_dir = exe.join("sprites");
        let anim = HamsterAnims::new(&exe.join("anims.txt").to_string_lossy());
        let (_, trash_enabled, turbo_enabled, start_with_windows) = load_config();
        Self {
            window: None, anim, sprites: HashMap::new(), sprite_dir, sprite_order: std::collections::VecDeque::new(), last_tick: Instant::now(),
            hwnd: None, dragging: false, hovered: false, begging: false, drag_offset: (0.0, 0.0),
            cursor: (0.0, 0.0), last_click_time: Instant::now(),
            trash_enabled, turbo_enabled, start_with_windows,
            last_frame_name: String::new(), rgn_cache: None, renderer: None,
            music_playing: false, last_audio_check: Instant::now(),
        }
    }
    fn get_sprite(&mut self, name: &str) -> Option<&Sprite> {
        if self.sprites.contains_key(name) {
            return self.sprites.get(name);
        }
        // LRU eviction: remove oldest when cache is full
        if self.sprites.len() >= SPRITE_CACHE_MAX {
            if let Some(old) = self.sprite_order.pop_front() {
                if let Some(evicted) = self.sprites.remove(&old) {
                    if let Some(h) = evicted.region {
                        unsafe { ffi::DeleteObject(h as ffi::HANDLE); }
                    }
                }
            }
        }
        if let Some(s) = load_sprite(&self.sprite_dir, name) {
            self.sprite_order.push_back(name.to_string());
            self.sprites.insert(name.to_string(), s);
        }
        self.sprites.get(name)
    }
    fn is_screen_visible(&self) -> bool {
        let hw = match self.hwnd { Some(h) => h, None => return false };
        unsafe {
            let mut r = std::mem::zeroed::<ffi::RECT>();
            if ffi::GetWindowRect(hw, &mut r) == 0 { return true; }
            let w = r.right - r.left; let h = r.bottom - r.top;
            if w <= 0 || h <= 0 { return false; }
            // Check 5 points in a grid (center + 4 quadrants)
            let pts = [
                ffi::POINT { x: r.left + w / 2, y: r.top + h / 2 },
                ffi::POINT { x: r.left + w / 4, y: r.top + h / 4 },
                ffi::POINT { x: r.left + 3 * w / 4, y: r.top + h / 4 },
                ffi::POINT { x: r.left + w / 4, y: r.top + 3 * h / 4 },
                ffi::POINT { x: r.left + 3 * w / 4, y: r.top + 3 * h / 4 },
            ];
            for pt in pts.iter() {
                if ffi::WindowFromPoint(*pt) == hw { return true; }
            }
        }
        false
    }

    fn wake_render(&self) {
        if let Some(win) = &self.window {
            win.request_redraw();
        }
    }
}

unsafe fn set_window_region(hwnd: ffi::HWND, rgn_cache: &mut Option<(String, ffi::HRGN)>, last_frame_name: &str, region: Option<ffi::HRGN>, data: &[u8], w: i32, h: i32) {
    if rgn_cache.as_ref().map_or(true, |(n, _)| n != last_frame_name) {
        if let Some((_, old_rgn)) = rgn_cache.take() {
            ffi::DeleteObject(old_rgn as ffi::HANDLE);
        }
        let rgn = if let Some(src) = region {
            let copy = ffi::CreateRectRgn(0, 0, 0, 0);
            if !copy.is_null() { ffi::CombineRgn(copy, src, std::ptr::null_mut(), 5); }
            copy
        } else {
            ffi::CreateRectRgn(0, 0, 0, 0)
        };
        if !rgn.is_null() {
            if region.is_none() {
                for y in 0..h {
                    let mut start_x: Option<i32> = None;
                    for x in 0..w {
                        let alpha = data[((y * w + x) * 4 + 3) as usize];
                        if alpha > 10 && start_x.is_none() {
                            start_x = Some(x);
                        } else if alpha <= 10 {
                            if let Some(sx) = start_x.take() {
                                let row_rgn = ffi::CreateRectRgn(sx, y, x, y + 1);
                                ffi::CombineRgn(rgn, rgn, row_rgn, 2);
                                ffi::DeleteObject(row_rgn as ffi::HANDLE);
                            }
                        }
                    }
                    if let Some(sx) = start_x {
                        let row_rgn = ffi::CreateRectRgn(sx, y, w, y + 1);
                        ffi::CombineRgn(rgn, rgn, row_rgn, 2);
                        ffi::DeleteObject(row_rgn as ffi::HANDLE);
                    }
                }
            }
            ffi::SetWindowRgn(hwnd, rgn, 0);
            *rgn_cache = Some((last_frame_name.to_string(), rgn));
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        el.set_control_flow(ControlFlow::Wait);
        // Create window WITHOUT with_transparent(true) - winit's transparent is broken
        let w = el.create_window(WindowAttributes::default()
            .with_title("Chomik Hamster").with_decorations(false)
            .with_inner_size(winit::dpi::LogicalSize::new(260.0, 310.0))).unwrap();

        if let Ok(h) = w.window_handle() {
            if let RawWindowHandle::Win32(wh) = h.as_ref() {
                let hwnd = wh.hwnd.get() as ffi::HWND;
                self.hwnd = Some(hwnd);
                HWND_STATIC.store(hwnd as *mut std::ffi::c_void, Ordering::Relaxed);
                unsafe {
                    let ex = ffi::GetWindowLongPtrW(hwnd, ffi::GWL_EXSTYLE) as u32;
                    ffi::ShowWindow(hwnd, ffi::SW_HIDE);
                    ffi::SetWindowLongPtrW(hwnd, ffi::GWL_EXSTYLE,
                        (ex & !ffi::WS_EX_APPWINDOW | ffi::WS_EX_LAYERED | ffi::WS_EX_TOOLWINDOW | ffi::WS_EX_ACCEPTFILES | 0x08000000) as isize);
                    ffi::SetWindowPos(hwnd, std::ptr::null_mut(), 0, 0, 0, 0,
                        0x0020 | 0x0001 | 0x0002);
                    self.renderer = Some(renderer::Renderer::new());
                    ffi::ShowWindow(hwnd, ffi::SW_SHOWNOACTIVATE);
                }
                add_tray_icon();
            }
        }

        let (pos, trash, turbo, startup) = load_config();
        self.trash_enabled = trash;
        self.turbo_enabled = turbo;
        self.start_with_windows = startup;
        TRASH_ENABLED.store(trash, Ordering::Relaxed);
        TURBO_ENABLED.store(turbo, Ordering::Relaxed);
        START_WITH_WINDOWS.store(startup, Ordering::Relaxed);
        set_startup(self.start_with_windows);
        let (wx, wy) = match pos {
            Some((x, y)) => (x, y),
            None => unsafe {
                ((ffi::GetSystemMetrics(ffi::SM_CXSCREEN) - 260).max(0),
                 (ffi::GetSystemMetrics(ffi::SM_CYSCREEN) - 310).max(0))
            },
        };
        let _ = w.set_outer_position(PhysicalPosition::new(wx, wy));
        self.window = Some(w);
        self.anim.play_idle();
        if let Some(win) = &self.window { win.request_redraw(); }
    }

    fn window_event(&mut self, _el: &ActiveEventLoop, _id: WindowId, e: WindowEvent) {
        match e {
            WindowEvent::CloseRequested => {
                if let Some(_hw) = self.hwnd { remove_tray_icon(); }
                _el.exit();
            }
            WindowEvent::RedrawRequested => {
                let dt = Instant::now().duration_since(self.last_tick).as_millis() as u64;
                if dt > 0 {
                    self.last_tick = Instant::now();
                    self.anim.update(dt);
                }
                let files: Vec<String> = DROPPED_FILES.lock().unwrap().drain(..).collect();
                if !files.is_empty() {
                    if self.turbo_enabled {
                        if confirm_turbo(&files, self.hwnd) {
                            let errors = eater::turbo_delete(&files);
                            if !errors.is_empty() {
                                let msg = format!("Couldn't eat some files:\n{}", errors.join("\n"));
                                let m16: Vec<u16> = msg.encode_utf16().chain(std::iter::once(0)).collect();
                                let t16: Vec<u16> = "🐹 Chomik Hamster\0".encode_utf16().collect();
                                unsafe { ffi::MessageBoxW(self.hwnd.unwrap_or(std::ptr::null_mut()), m16.as_ptr(), t16.as_ptr(), 0x10 | 0x1000); }
                            }
                        }
                    } else {
                        eater::send_to_bin(&files);
                    }
                }
                let sn = self.anim.current_sprite().map(|s| s.to_string());
                if sn.as_ref() != Some(&self.last_frame_name) {
                    self.last_frame_name = sn.clone().unwrap_or_default();
                    if let Some(hw) = self.hwnd {
                        if let Some(ref name) = sn {
                            self.get_sprite(name);
                            if let Some(s) = self.sprites.get(name) {
                                if let Some(ref mut r) = self.renderer {
                                    unsafe { r.render(hw as *mut std::ffi::c_void, &s.data, s.w, s.h); }
                                }
                                unsafe { set_window_region(hw, &mut self.rgn_cache, &self.last_frame_name, s.region, &s.data, s.w, s.h); }
                            }
                        }
                    }
                }
                // Dynamic timer: next tick at frame boundary
                let remaining = self.anim.remaining_frame_ms();
                let next_ms = remaining.max(16).min(5000) as u32;
                let tray_hwnd = TRAY_HWND.load(Ordering::Relaxed);
                if !tray_hwnd.is_null() {
                    unsafe { ffi::SetTimer(tray_hwnd as ffi::HWND, ANIM_TIMER_ID, next_ms, None); }
                }
                // Hover → beg loop transition
                if self.hovered && !self.begging && !self.dragging && (!self.anim.is_busy() || self.music_playing) {
                    self.begging = true;
                    self.music_playing = false;
                    self.anim.stop_loop();
                    self.anim.play_loop("AnimBegLoop");
                    self.wake_render();
                }
                // Music start → loop transition
                if self.music_playing && !self.anim.is_busy() && self.anim.current_loop().is_empty() && !self.begging {
                    self.anim.play_loop("AnimMusicLoop");
                    self.wake_render();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = (position.x, position.y);
                if self.dragging {
                    if let Some(win) = &self.window {
                        if let Ok(cur_pos) = win.outer_position() {
                            // absolute cursor = window_position + cursor_in_window
                            // new window = absolute_cursor - original_offset
                            let _ = win.set_outer_position(PhysicalPosition::new(
                                (cur_pos.x as f64 + position.x - self.drag_offset.0) as i32,
                                (cur_pos.y as f64 + position.y - self.drag_offset.1) as i32,
                            ));
                        }
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                use winit::event::ElementState;
                if button == winit::event::MouseButton::Left {
                    match state {
                        ElementState::Pressed => {
                            if self.last_click_time.elapsed().as_millis() < 500 { open_trash(); }
                            self.last_click_time = Instant::now();
                            self.dragging = true;
                            self.drag_offset = self.cursor;
                        }
                        ElementState::Released => {
                            if self.dragging {
                                self.dragging = false;
                                if let Some(win) = &self.window {
                                    if let Ok(pos) = win.outer_position() {
                                        save_config(pos.x as i32, pos.y as i32, self.trash_enabled, self.turbo_enabled, self.start_with_windows);
                                    }
                                }
                            }
                        }
                    }
                }
                if button == winit::event::MouseButton::Right && state == ElementState::Pressed {
                    self.anim.play_seq(&["AnimSpeakingStart", "AnimSpeaking", "AnimSpeakingFinish"]);
                    self.wake_render();
                    if let Some(hw) = self.hwnd {
                        match show_menu(hw, self.trash_enabled, self.turbo_enabled, self.start_with_windows) {
                            CMD_OPEN => open_trash(),
                            CMD_EMPTY => {
                                let _ = std::process::Command::new("cmd")
                                    .args(["/C", "rundll32", "shell32.dll,SHEmptyRecycleBin"]).spawn();
                            }
                            CMD_TOGGLE_TRASH => {
                                self.trash_enabled = !self.trash_enabled;
                                if let Some(win) = &self.window {
                                    if let Ok(pos) = win.outer_position() {
                                        save_config(pos.x as i32, pos.y as i32, self.trash_enabled, self.turbo_enabled, self.start_with_windows);
                                    }
                                }
                            }
                            CMD_TOGGLE_TURBO => {
                                self.turbo_enabled = !self.turbo_enabled;
                                if let Some(win) = &self.window {
                                    if let Ok(pos) = win.outer_position() {
                                        save_config(pos.x as i32, pos.y as i32, self.trash_enabled, self.turbo_enabled, self.start_with_windows);
                                    }
                                }
                            }
                            CMD_TOGGLE_STARTUP => {
                                self.start_with_windows = !self.start_with_windows;
                                set_startup(self.start_with_windows);
                                if let Some(win) = &self.window {
                                    if let Ok(pos) = win.outer_position() {
                                        save_config(pos.x as i32, pos.y as i32, self.trash_enabled, self.turbo_enabled, self.start_with_windows);
                                    }
                                }
                            }
                            CMD_QUIT => _el.exit(),
                            _ => {}
                        }
                    }
                }
            }
            WindowEvent::DroppedFile(path) => {
                if self.trash_enabled {
                DROPPED_FILES.lock().unwrap().push(path.to_string_lossy().to_string());
                self.begging = false;
                self.anim.play_seq(&["AnimDragFileStart", "AnimDragFileProcessing", "AnimDragFileFinish"]);
                self.wake_render();
                }
            }
            WindowEvent::HoveredFile(_) => {
                if !self.anim.is_busy() && !self.dragging {
                    self.hovered = true;
                    self.begging = false;
                    self.anim.play_seq(&["AnimBegStart"]);
                    self.wake_render();
                } else if !self.hovered {
                    self.hovered = true;
                }
            }
            WindowEvent::HoveredFileCancelled => {
                if self.begging && !self.hovered {
                    self.begging = false;
                    self.anim.stop_loop();
                    self.anim.play_seq(&["AnimBegEnd"]);
                    self.wake_render();
                }
            }
            WindowEvent::CursorEntered { .. } => {
                if !self.hovered && !self.dragging {
                    self.hovered = true;
                    self.begging = false;
                    self.anim.play_seq(&["AnimBegStart"]);
                    self.wake_render();
                } else if !self.hovered {
                    self.hovered = true;
                }
            }
            WindowEvent::CursorLeft { .. } => {
                self.hovered = false;
                if self.begging {
                    self.begging = false;
                    self.anim.stop_loop();
                    self.anim.play_seq(&["AnimBegEnd"]);
                    self.wake_render();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Handle tray menu commands
        let cmd = TRAY_CMD.swap(0, Ordering::Relaxed);
        if cmd != 0 {
            match cmd {
                CMD_OPEN => open_trash(),
                CMD_EMPTY => {
                    let _ = std::process::Command::new("cmd")
                        .args(["/C", "rundll32", "shell32.dll,SHEmptyRecycleBin"]).spawn();
                }
                CMD_TOGGLE_TRASH => {
                    self.trash_enabled = !self.trash_enabled;
                    TRASH_ENABLED.store(self.trash_enabled, Ordering::Relaxed);
                    if let Some(win) = &self.window {
                        if let Ok(pos) = win.outer_position() {
                            save_config(pos.x as i32, pos.y as i32,
                                self.trash_enabled, self.turbo_enabled, self.start_with_windows);
                        }
                    }
                }
                CMD_TOGGLE_TURBO => {
                    self.turbo_enabled = !self.turbo_enabled;
                    TURBO_ENABLED.store(self.turbo_enabled, Ordering::Relaxed);
                    if let Some(win) = &self.window {
                        if let Ok(pos) = win.outer_position() {
                            save_config(pos.x as i32, pos.y as i32,
                                self.trash_enabled, self.turbo_enabled, self.start_with_windows);
                        }
                    }
                }
                CMD_TOGGLE_STARTUP => {
                    self.start_with_windows = !self.start_with_windows;
                    START_WITH_WINDOWS.store(self.start_with_windows, Ordering::Relaxed);
                    set_startup(self.start_with_windows);
                    if let Some(win) = &self.window {
                        if let Ok(pos) = win.outer_position() {
                            save_config(pos.x as i32, pos.y as i32,
                                self.trash_enabled, self.turbo_enabled, self.start_with_windows);
                        }
                    }
                }
                CMD_QUIT => {
                    remove_tray_icon();
                    event_loop.exit();
                }
                _ => {}
            }
        }

        if !HWND_VISIBLE.load(Ordering::Relaxed) || self.hwnd.is_none() {
            event_loop.set_control_flow(ControlFlow::Wait);
            return;
        }
        if ANIM_WAKE.swap(false, Ordering::Relaxed) {
            if let Some(win) = &self.window {
                win.request_redraw();
            }
        }
        // Audio detection every 2s
        if self.last_audio_check.elapsed().as_secs() >= 2 {
            self.last_audio_check = Instant::now();
            let has_audio = audio::has_active_audio();
            if has_audio && !self.anim.is_busy() && !self.music_playing && !self.begging {
                self.music_playing = true;
                self.anim.play_seq(&["AnimMusicStart"]);
                self.wake_render();
            } else if !has_audio && self.music_playing {
                self.music_playing = false;
                self.anim.stop_loop();
                self.anim.play_seq(&["AnimMusicFinish"]);
                self.wake_render();
            }
        }
        event_loop.set_control_flow(ControlFlow::Wait);
    }
}

fn main() {
    let el = EventLoop::new().unwrap();
    el.set_control_flow(ControlFlow::Wait);
    el.run_app(&mut App::new()).unwrap();
}
