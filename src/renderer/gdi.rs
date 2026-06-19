use std::ffi::c_void;
use std::ptr;

use crate::ffi;

struct DibCache {
    hb: ffi::HBITMAP,
    bits: *mut u8,
    w: i32,
    h: i32,
}

pub struct GDIRenderer {
    dib: Option<DibCache>,
    sdc: Option<ffi::HDC>,
    mdc: Option<ffi::HDC>,
}

impl GDIRenderer {
    pub fn new() -> Self {
        Self { dib: None, sdc: None, mdc: None }
    }

    fn ensure_dc(&mut self) {
        unsafe {
            if self.sdc.is_none() {
                self.sdc = Some(ffi::GetDC(ptr::null_mut()));
            }
            if self.mdc.is_none() {
                if let Some(sdc) = self.sdc {
                    self.mdc = Some(ffi::CreateCompatibleDC(sdc));
                }
            }
        }
    }

    pub unsafe fn render(&mut self, hwnd: *mut c_void, data: &[u8], w: i32, h: i32) {
        self.ensure_dc();

        let dib_changed = self.dib.as_ref().map_or(true, |d| d.w != w || d.h != h);
        if dib_changed {
            if let Some(old) = self.dib.take() {
                ffi::DeleteObject(old.hb as ffi::HANDLE);
            }
            if let Some(sdc) = self.sdc {
                let mut bmi: ffi::BITMAPINFO = std::mem::zeroed();
                bmi.h.biSize = std::mem::size_of::<ffi::BITMAPINFOHEADER>() as u32;
                bmi.h.biWidth = w;
                bmi.h.biHeight = -h;
                bmi.h.biPlanes = 1;
                bmi.h.biBitCount = 32;
                let mut bits: *mut u8 = ptr::null_mut();
                let hb = ffi::CreateDIBSection(sdc, &bmi, 0, &mut bits, ptr::null_mut(), 0);
                if !hb.is_null() && !bits.is_null() {
                    self.dib = Some(DibCache { hb, bits, w, h });
                    if let Some(mdc) = self.mdc {
                        ffi::SelectObject(mdc, hb as ffi::HANDLE);
                    }
                }
            }
        }

        if let Some(ref dib) = self.dib {
            ptr::copy_nonoverlapping(data.as_ptr(), dib.bits, data.len());

            if let (Some(sdc), Some(mdc)) = (self.sdc, self.mdc) {
                let bl = ffi::BLENDFUNCTION {
                    BlendOp: ffi::AC_SRC_OVER,
                    BlendFlags: 0,
                    SourceConstantAlpha: 255,
                    AlphaFormat: ffi::AC_SRC_ALPHA,
                };
                let sz = ffi::SIZE { cx: w, cy: h };
                let p0 = ffi::POINT { x: 0, y: 0 };
                let ret = ffi::UpdateLayeredWindow(
                    hwnd as ffi::HWND,
                    sdc,
                    ptr::null(),
                    &sz,
                    mdc,
                    &p0,
                    0,
                    &bl,
                    ffi::ULW_ALPHA,
                );
                if ret == 0 { eprintln!("ULW failed, GLE={}", std::io::Error::last_os_error().raw_os_error().unwrap_or(0)); }
            }
        }
    }
}

impl Drop for GDIRenderer {
    fn drop(&mut self) {
        unsafe {
            if let Some(dib) = self.dib.take() {
                ffi::DeleteObject(dib.hb as ffi::HANDLE);
            }
            if let Some(mdc) = self.mdc.take() {
                ffi::DeleteDC(mdc);
            }
            if let Some(sdc) = self.sdc.take() {
                ffi::ReleaseDC(ptr::null_mut(), sdc);
            }
        }
    }
}
