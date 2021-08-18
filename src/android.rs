use crate::window::Window;
use crate::looper::Looper;
use ndk_sys::{ANativeActivity, ALooper_prepare};
use std::boxed::Box;

pub struct App {
    pub window: Option<Window>,
    pub looper: Looper,
}

impl App {
    pub unsafe fn init(activity: *mut ANativeActivity) {
        let alooper = ALooper_prepare(0);
        let app = Box::new(App {
            window: None,
            looper: Looper::create(alooper),
        });
        (*activity).instance = Box::into_raw(app) as *mut std::os::raw::c_void;
    }

    // activity & activity's instance must not be null here!!
    pub fn from_ptr(activity: *mut ANativeActivity) -> &'static mut Self {
        unsafe {
            let instance = (*activity).instance as *mut App;
            Box::leak(Box::from_raw(instance))
        }
    }
}