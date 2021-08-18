use ndk_sys::{ALooper,AInputQueue,AInputQueue_attachLooper,AInputQueue_detachLooper};

const LOOPER_ID_EVENT: i32 = 1;

pub struct Looper {
    ptr: *mut ALooper,
}

impl Looper {
    pub fn create<'a>(ptr: *mut ALooper) -> Self {
        Looper {
            ptr: ptr
        }
    }

    pub fn bind_queue(&self, queue: *mut AInputQueue) {
        unsafe {
            AInputQueue_attachLooper(queue, self.ptr, LOOPER_ID_EVENT, None, std::ptr::null_mut())
        }
    }

    pub fn unbind_queue(&self, queue: *mut AInputQueue) {
        unsafe {
            AInputQueue_detachLooper(queue);
        }
    }
}