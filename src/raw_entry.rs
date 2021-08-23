use std::{convert::TryInto, ffi::CString, os::raw, path::Path, ptr};
use std::borrow::Borrow;
use std::ops::Deref;
use std::thread::sleep;
use std::time::Duration;

use android_logger::Config;
use ash::{
    Entry,
    extensions::khr::Surface,
    Instance, vk::{self, HANDLE},
};
use ash::vk::{Fence, SubmitInfo};
use log::Level;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use raw_window_handle::android::AndroidHandle;
use skia_safe::{Budgeted, Canvas, gpu, ImageInfo};
use skia_safe::gpu::vk::GetProcOf;
use vk::Handle;

pub type SizeT = raw::c_ulong;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ANativeWindow {
    _unused: [u8; 0],
}

pub struct ANativeWindowWrapper {
    ptr: *mut ANativeWindow,
}

unsafe impl HasRawWindowHandle for ANativeWindowWrapper {
    fn raw_window_handle(&self) -> RawWindowHandle {
        info!("handle1 {:}", self.ptr as u64);
        return RawWindowHandle::Android(raw_window_handle::android::AndroidHandle {
            a_native_window: self.ptr as *mut libc::c_void,
            ..AndroidHandle::empty()
        });
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct AInputQueue {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ARect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

struct Window {
    ash_graphics: AshGraphics,
    render_context: gpu::DirectContext,
    width: i32,
    height: i32,
}

impl Window {
    fn draw(&mut self) {
        let img_info = ImageInfo::new_n32_premul((self.width, self.height), None);
        let mut surface = skia_safe::Surface::new_render_target(
            &mut self.render_context, Budgeted::Yes, &img_info, None, gpu::SurfaceOrigin::TopLeft, None, false,
        ).unwrap();
        let canvas = surface.canvas();
        use skia_safe::{Rect, Color4f, ColorSpace, Paint};
        let rect = Rect {
            left: 300.0,
            right: 700.0,
            top: 500.0,
            bottom: 1000.0,
        };
        let color = Color4f {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        let color_space = ColorSpace::new_srgb();
        let paint = Paint::new(&color, Some(&color_space));
        canvas.draw_rect(rect, &paint);
        surface.flush_and_submit();
        unsafe {
            self.ash_graphics.device.queue_submit(self.ash_graphics.queue_and_index.0, &vec![SubmitInfo::default()], Fence::null());
        }
    }
}

struct App {
    window: Option<Window>,
}

extern "C" {
    pub fn ANativeWindow_getWidth(window: *mut ANativeWindow) -> i32;
}

extern "C" {
    pub fn ANativeWindow_getHeight(window: *mut ANativeWindow) -> i32;
}

impl App {
    fn init(activity: *mut ANativeActivity) {
        let app = Box::new(App {
            window: None
        });
        unsafe { (*activity).instance = (Box::into_raw(app) as *mut std::os::raw::c_void) };
    }

    fn get_app(activity: *mut ANativeActivity) -> &'static mut Self {
        unsafe {
            let instance = (*activity).instance as *mut App;
            Box::leak(Box::from_raw(instance))
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ANativeActivityCallbacks {
    pub on_start: Option<unsafe extern "C" fn(activity: *mut ANativeActivity)>,
    pub on_resume: Option<unsafe extern "C" fn(activity: *mut ANativeActivity)>,
    pub on_save_instance_state: Option<
        unsafe extern "C" fn(
            activity: *mut ANativeActivity,
            out_size: *mut SizeT,
        ) -> *mut raw::c_void,
    >,
    pub on_pause: Option<unsafe extern "C" fn(activity: *mut ANativeActivity)>,
    pub on_stop: Option<unsafe extern "C" fn(activity: *mut ANativeActivity)>,
    pub on_destroy: Option<unsafe extern "C" fn(activity: *mut ANativeActivity)>,
    pub on_window_focus_changed: Option<
        unsafe extern "C" fn(activity: *mut ANativeActivity, has_focus: raw::c_int),
    >,
    pub on_native_window_created: Option<
        unsafe extern "C" fn(activity: *mut ANativeActivity, window: *mut ANativeWindow),
    >,
    pub on_native_window_resized: Option<
        unsafe extern "C" fn(activity: *mut ANativeActivity, window: *mut ANativeWindow),
    >,
    pub on_native_window_redraw_needed: Option<
        unsafe extern "C" fn(activity: *mut ANativeActivity, window: *mut ANativeWindow),
    >,
    pub on_native_window_destroyed: Option<
        unsafe extern "C" fn(activity: *mut ANativeActivity, window: *mut ANativeWindow),
    >,
    pub on_input_queue_created: Option<
        unsafe extern "C" fn(activity: *mut ANativeActivity, queue: *mut AInputQueue),
    >,
    pub on_input_queue_destroyed: Option<
        unsafe extern "C" fn(activity: *mut ANativeActivity, queue: *mut AInputQueue),
    >,
    pub on_content_rect_changed: Option<
        unsafe extern "C" fn(activity: *mut ANativeActivity, rect: *const ARect),
    >,
    pub on_configuration_changed:
    Option<unsafe extern "C" fn(activity: *mut ANativeActivity)>,
    pub on_low_memory: Option<unsafe extern "C" fn(activity: *mut ANativeActivity)>,
}


#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ANativeActivity {
    pub callbacks: *mut ANativeActivityCallbacks,
    pub vm: *mut raw::c_void,
    //JavaVM,
    pub env: *mut raw::c_void,
    //JNIEnv,
    pub clazz: *mut raw::c_void,
    //jobject,
    pub internal_data_path: *const raw::c_char,
    pub external_data_path: *const raw::c_char,
    pub sdk_version: i32,
    pub instance: *mut raw::c_void,
    // pub asset_manager: *mut AAssetManager,
    // pub obb_path: *const raw::c_char,
}

pub extern "C" fn on_window_created(activity: *mut ANativeActivity, window_ptr: *mut ANativeWindow) {
    info!("Native window created, window = {:?}", window_ptr);
    let app = App::get_app(activity);
    match app.window {
        None => {
            // let a_native_window = unsafe { Box::from_raw(window_ptr) };
            info!("handle 0 {:}", window_ptr as u64);
            let ash_graphics = unsafe {
                AshGraphics::new("rust_app", &ANativeWindowWrapper {
                    ptr: window_ptr
                })
            };
            app.window = Some(Window {
                render_context: vk_init(ash_graphics.borrow()),
                ash_graphics,
                width: 0,
                height: 0,
            });
        }
        _ => {}
    }
}

pub extern "C" fn on_window_resized(activity: *mut ANativeActivity, window_ptr: *mut ANativeWindow) {
    info!("Native window resized, window = {:?}", window_ptr);
    let app = App::get_app(activity);
    match &mut app.window {
        Some(window) => {
            unsafe {
                window.width = ANativeWindow_getWidth(window_ptr);
                window.height = ANativeWindow_getHeight(window_ptr);
            }
            info!("width {:} height {:}", window.width, window.height);
        }
        _ => {}
    }
}

pub extern "C" fn on_window_redraw(activity: *mut ANativeActivity, window_ptr: *mut ANativeWindow) {
    info!("Native window redraw, window = {:?}", window_ptr);
    let app = App::get_app(activity);
    app.window.as_mut().unwrap().draw();
}

#[no_mangle]
pub extern "C" fn ANativeActivity_onCreate(
    activity: *mut ANativeActivity,
    saved_state: *mut raw::c_void,
    saved_state_size: SizeT,
) {
    android_logger::init_once(
        Config::default()
            .with_min_level(Level::Trace)
            .with_tag("RustApp")
    );

    info!("Native activity instance = {:?}, savedState = {:?}, size = {}", activity, saved_state, saved_state_size);


    unsafe {
        // initialize all callbacks
        let mut callbacks = (*activity).callbacks.as_mut().unwrap();
        // callbacks.on_start = Some(on_start);
        // callbacks.onResume = Some(on_resume);
        // callbacks.on_pause = Some(on_pause);
        // callbacks.on_stop = Some(on_stop);
        // callbacks.onDestroy = Some(on_destroy);
        // callbacks.onWindowFocusChanged = Some(on_window_focus_changed);
        callbacks.on_native_window_created = Some(on_window_created);
        callbacks.on_native_window_resized = Some(on_window_resized);
        callbacks.on_native_window_redraw_needed = Some(on_window_redraw);
        // callbacks.onInputQueueCreated = Some(on_input_queue_created);
        // callbacks.onInputQueueDestroyed = Some(on_input_queue_destroyed);

        App::init(activity);
    }
}


fn vk_init(ash_graphics: &AshGraphics) -> gpu::DirectContext {
    unsafe {
        let context = {
            let get_proc = |of| unsafe {
                match ash_graphics.get_proc(of) {
                    Some(f) => f as _,
                    None => {
                        println!("resolve of {} failed", of.name().to_str().unwrap());
                        ptr::null()
                    }
                }
            };

            let backend_context = unsafe {
                gpu::vk::BackendContext::new(
                    ash_graphics.instance.handle().as_raw() as _,
                    ash_graphics.physical_device.as_raw() as _,
                    ash_graphics.device.handle().as_raw() as _,
                    (
                        ash_graphics.queue_and_index.0.as_raw() as _,
                        ash_graphics.queue_and_index.1,
                    ),
                    &get_proc,
                )
            };
            gpu::DirectContext::new_vulkan(&backend_context, None).unwrap()
        };
        context
    }
}


pub struct AshGraphics {
    pub entry: Entry,
    pub instance: Instance,
    pub physical_device: vk::PhysicalDevice,
    pub device: ash::Device,
    pub queue_and_index: (vk::Queue, usize),
}

impl Drop for AshGraphics {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().unwrap();
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

impl AshGraphics {
    pub fn vulkan_version() -> Option<(usize, usize, usize)> {
        let entry = unsafe { Entry::new() }.unwrap();

        let detected_version = entry.try_enumerate_instance_version().unwrap_or(None);

        detected_version.map(|ver| {
            (
                vk::api_version_major(ver).try_into().unwrap(),
                vk::api_version_minor(ver).try_into().unwrap(),
                vk::api_version_patch(ver).try_into().unwrap(),
            )
        })
    }

    pub unsafe fn new(app_name: &str, a_native_window: &ANativeWindowWrapper) -> AshGraphics {
        let entry = Entry::new().unwrap();

        let minimum_version = vk::make_api_version(0, 1, 0, 0);

        match a_native_window.raw_window_handle() {
            RawWindowHandle::Android(handle) => {
                info!("handle {:}", handle.a_native_window as u64);
            }
            _ => {}
        }

        let instance: Instance = {
            let api_version = Self::vulkan_version()
                .map(|(major, minor, patch)| {
                    vk::make_api_version(
                        0,
                        major.try_into().unwrap(),
                        minor.try_into().unwrap(),
                        patch.try_into().unwrap(),
                    )
                })
                .unwrap_or(minimum_version);
            info!("api version {:}", api_version);

            let surface_extensions = ash_window::enumerate_required_extensions(a_native_window).unwrap();
            let mut extension_names_raw = surface_extensions
                .iter()
                .map(|ext| ext.as_ptr())
                .collect::<Vec<_>>();


            let app_name = CString::new(app_name).unwrap();
            let layer_names: [&CString; 0] = []; // [CString::new("VK_LAYER_LUNARG_standard_validation").unwrap()];

            let app_info = vk::ApplicationInfo::builder()
                .application_name(&app_name)
                .application_version(0)
                .engine_name(&app_name)
                .engine_version(0)
                .api_version(api_version);

            let layers_names_raw: Vec<*const raw::c_char> = layer_names
                .iter()
                .map(|raw_name| raw_name.as_ptr())
                .collect();

            let create_info = vk::InstanceCreateInfo::builder()
                .application_info(&app_info)
                .enabled_layer_names(&layers_names_raw)
                .enabled_extension_names(&extension_names_raw);

            entry
                .create_instance(&create_info, None)
                .expect("Failed to create a Vulkan instance.")
        };

        let surface = unsafe { ash_window::create_surface(&entry, &instance, a_native_window, None).unwrap() };

        let (physical_device, queue_family_index) = {
            let physical_devices = instance
                .enumerate_physical_devices()
                .expect("Failed to enumerate Vulkan physical devices.");

            let surface_loader = Surface::new(&entry, &instance);

            physical_devices
                .iter()
                .map(|physical_device| {
                    instance
                        .get_physical_device_queue_family_properties(*physical_device)
                        .iter()
                        .enumerate()
                        .find_map(|(index, info)| {
                            let supports_graphic_and_surface =
                                info.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                                    && surface_loader
                                    .get_physical_device_surface_support(
                                        *physical_device,
                                        index as u32,
                                        surface,
                                    )
                                    .unwrap();
                            if supports_graphic_and_surface {
                                Some((*physical_device, index))
                            } else {
                                None
                            }
                        })
                })
                .find_map(|v| v)
                .expect("Failed to find a suitable Vulkan device.")
        };

        let device: ash::Device = {
            let features = vk::PhysicalDeviceFeatures::default();

            let priorities = [1.0];

            let queue_info = [vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(queue_family_index as _)
                .queue_priorities(&priorities)
                .build()];

            let device_extension_names_raw = [];

            let device_create_info = vk::DeviceCreateInfo::builder()
                .queue_create_infos(&queue_info)
                .enabled_extension_names(&device_extension_names_raw)
                .enabled_features(&features);

            instance
                .create_device(physical_device, &device_create_info, None)
                .unwrap()
        };

        let queue_index: usize = 0;
        let queue: vk::Queue = device.get_device_queue(queue_family_index as _, queue_index as _);

        AshGraphics {
            queue_and_index: (queue, queue_index),
            device,
            physical_device,
            instance,
            entry,
        }
    }

    pub unsafe fn get_proc(&self, of: gpu::vk::GetProcOf) -> Option<unsafe extern "system" fn()> {
        match of {
            gpu::vk::GetProcOf::Instance(instance, name) => {
                let ash_instance = vk::Instance::from_raw(instance as _);
                self.entry.get_instance_proc_addr(ash_instance, name)
            }
            gpu::vk::GetProcOf::Device(device, name) => {
                let ash_device = vk::Device::from_raw(device as _);
                self.instance.get_device_proc_addr(ash_device, name)
            }
        }
    }
}