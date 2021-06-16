use std::ffi::CStr;
use std::os::raw::c_char;

pub fn vk_to_str(c: &[c_char]) -> &str {
    unsafe { CStr::from_ptr(c.as_ptr()) }
        .to_str()
        .expect("failed to convert vulkan string")
}
