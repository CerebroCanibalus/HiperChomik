mod gdi;

use std::ffi::c_void;

pub struct Renderer(gdi::GDIRenderer);

impl Renderer {
    pub fn new() -> Self {
        Self(gdi::GDIRenderer::new())
    }

    pub unsafe fn render(&mut self, hwnd: *mut c_void, data: &[u8], w: i32, h: i32) {
        self.0.render(hwnd, data, w, h)
    }
}
