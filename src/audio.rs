use windows::Win32::Media::Audio::*;
use windows::Win32::System::Com::*;
use windows::core::{GUID, Interface};

const IID_IAUDIO_METER: GUID = GUID::from_u128(0xC02216F6_8C67_4B5B_9D00_D008E73E0064);

type ActivateFn = unsafe extern "system" fn(*mut std::ffi::c_void, *const GUID, u32, *const std::ffi::c_void, *mut *mut std::ffi::c_void) -> i32;
type ReleaseFn = unsafe extern "system" fn(*mut std::ffi::c_void) -> u32;
type GetPeakFn = unsafe extern "system" fn(*mut std::ffi::c_void, *mut f32) -> i32;

pub fn has_active_audio() -> bool {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let enumerator: IMMDeviceEnumerator = match CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) {
            Ok(e) => e,
            Err(_) => return false,
        };
        let device = match enumerator.GetDefaultAudioEndpoint(eRender, eConsole) {
            Ok(d) => d,
            Err(_) => return false,
        };
        let raw = device.as_raw();
        let vtable = *(raw as *mut *mut *mut std::ffi::c_void);
        let activate: ActivateFn = std::mem::transmute(*vtable.add(3));
        let mut meter_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
        let hr = activate(raw, &IID_IAUDIO_METER, CLSCTX_ALL.0 as u32, std::ptr::null(), &mut meter_ptr);
        if hr < 0 || meter_ptr.is_null() {
            return false;
        }
        let meter_vtable = *(meter_ptr as *mut *mut *mut std::ffi::c_void);
        let get_peak: GetPeakFn = std::mem::transmute(*meter_vtable.add(3));
        let mut peak = 0.0f32;
        let peak_hr = get_peak(meter_ptr, &mut peak);
        let release_fn: ReleaseFn = std::mem::transmute(*meter_vtable.add(2));
        release_fn(meter_ptr);
        peak_hr >= 0 && peak > 0.001
    }
}
