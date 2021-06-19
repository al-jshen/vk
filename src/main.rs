use std::ffi::{c_void, CStr, CString};

use ash::extensions::{ext::DebugUtils, khr};
use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::vk;
use vka::vk_to_str;
use winit::{
    event::{ElementState, KeyboardInput, VirtualKeyCode},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
    window::WindowBuilder,
};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

const VALIDATION_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];

#[cfg(debug_assertions)]
const ENABLE_VALIDATION_LAYERS: bool = true;
#[cfg(not(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = false;

struct VkApp {
    _entry: ash::Entry,
    instance: ash::Instance,
    debug_utils: DebugUtils,
    debug_messenger: vk::DebugUtilsMessengerEXT,
    device: ash::Device,
    surface: vk::SurfaceKHR,
    surface_loader: khr::Surface,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
}

struct QueueFamilyIndices {
    graphics_family: Option<u32>,
    present_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

unsafe extern "system" fn debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
    let message = CStr::from_ptr((*p_callback_data).p_message);

    println!(
        "[DEBUG] [{:?}] [{:?}] {:?}",
        message_severity, message_type, message
    );

    ash::vk::FALSE
}

fn populate_debug_messenger_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
    vk::DebugUtilsMessengerCreateInfoEXT {
        s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
        p_next: std::ptr::null(),
        flags: vk::DebugUtilsMessengerCreateFlagsEXT::empty(),
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
        | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
        // | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
        | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
            | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
        pfn_user_callback: Some(debug_callback),
        p_user_data: std::ptr::null_mut(),
    }
}

impl VkApp {
    fn init_vulkan(window: &Window) -> Self {
        let entry = unsafe { ash::Entry::new().unwrap() };
        let instance = Self::create_instance(&entry, window);
        let (debug_utils, debug_messenger) = Self::setup_debug_messenger(&entry, &instance);
        let (surface, surface_loader) = Self::create_surface(&entry, &instance, &window);
        let physical_device = Self::pick_physical_device(&instance, &surface_loader, &surface);
        let (logical_device, graphics_queue, present_queue) =
            Self::create_logical_device(&instance, physical_device, &surface_loader, &surface);

        VkApp {
            _entry: entry,
            instance,
            debug_utils,
            debug_messenger,
            device: logical_device,
            surface,
            surface_loader,
            graphics_queue,
            present_queue,
        }
    }

    fn init_window(event_loop: &EventLoop<()>) -> Window {
        WindowBuilder::new()
            .with_inner_size(winit::dpi::LogicalSize::new(WIDTH, HEIGHT))
            .with_title("Vulkan")
            .build(event_loop)
            .expect("failed to create window")
    }

    fn setup_debug_messenger(
        entry: &ash::Entry,
        instance: &ash::Instance,
    ) -> (DebugUtils, vk::DebugUtilsMessengerEXT) {
        let debug_utils = DebugUtils::new(entry, instance);

        if !ENABLE_VALIDATION_LAYERS {
            return (debug_utils, vk::DebugUtilsMessengerEXT::null());
        }

        let messenger_create_info = populate_debug_messenger_create_info();

        let debug_utils_messenger = unsafe {
            debug_utils
                .create_debug_utils_messenger(&messenger_create_info, None)
                .expect("could not create debug messenger")
        };

        (debug_utils, debug_utils_messenger)
    }

    fn check_validation_layer_support(entry: &ash::Entry) -> bool {
        let available_layers = entry
            .enumerate_instance_layer_properties()
            .expect("could not enumerate instance layer properties");

        let available_layers = available_layers
            .iter()
            .map(|x| vk_to_str(&x.layer_name))
            .collect::<Vec<_>>();

        println!("Available layers");
        for l in &available_layers {
            println!("\t{}", l);
        }

        for val_layer in VALIDATION_LAYERS {
            if !available_layers.contains(&val_layer) {
                return false;
            }
        }

        true
    }

    pub fn create_surface(
        entry: &ash::Entry,
        instance: &ash::Instance,
        window: &Window,
    ) -> (vk::SurfaceKHR, khr::Surface) {
        let surface = unsafe {
            ash_window::create_surface(entry, instance, window, None)
                .expect("failed to create window surface!")
        };

        let surface_loader = khr::Surface::new(entry, instance);

        (surface, surface_loader)
    }

    pub fn pick_physical_device(
        instance: &ash::Instance,
        surface_loader: &khr::Surface,
        surface: &vk::SurfaceKHR,
    ) -> vk::PhysicalDevice {
        let devices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("could not enumerate physical devices")
        };

        if devices.is_empty() {
            panic!("failed to find GPUs with Vulkan support!");
        }

        for device in devices {
            if Self::is_device_suitable(instance, device, surface_loader, surface) {
                return device;
            }
        }

        panic!("failed to find GPUs with Vulkan support!");
    }

    pub fn is_device_suitable(
        instance: &ash::Instance,
        device: vk::PhysicalDevice,
        surface_loader: &khr::Surface,
        surface: &vk::SurfaceKHR,
    ) -> bool {
        // let mut device_properties = unsafe { instance.get_physical_device_properties(device) };
        // let mut device_features = unsafe { instance.get_physical_device_features(device) };

        let indices = Self::find_queue_family(instance, device, surface_loader, surface);

        indices.is_complete()
    }

    pub fn find_queue_family(
        instance: &ash::Instance,
        device: vk::PhysicalDevice,
        surface_loader: &khr::Surface,
        surface: &vk::SurfaceKHR,
    ) -> QueueFamilyIndices {
        let mut indices = QueueFamilyIndices {
            graphics_family: None,
            present_family: None,
        };

        let queue_families_properties =
            unsafe { instance.get_physical_device_queue_family_properties(device) };

        let mut i = 0;
        for qf in queue_families_properties.iter() {
            if qf.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                indices.graphics_family = Some(i as u32);
            }

            if unsafe {
                surface_loader
                    .get_physical_device_surface_support(device, i, *surface)
                    .expect("failed to get physical device surface support!")
            } {
                indices.present_family = Some(i);
            }

            if indices.is_complete() {
                break;
            }

            i += 1;
        }

        indices
    }

    pub fn create_logical_device(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface_loader: &khr::Surface,
        surface: &vk::SurfaceKHR,
    ) -> (ash::Device, vk::Queue, vk::Queue) {
        let indices = Self::find_queue_family(instance, physical_device, surface_loader, surface);

        let mut unique_queue_families = std::collections::HashSet::new();
        unique_queue_families.insert(indices.graphics_family.unwrap());
        unique_queue_families.insert(indices.present_family.unwrap());

        let queue_priority = &1_f32 as *const f32;

        let queue_create_infos = unique_queue_families
            .iter()
            .map(|qf| vk::DeviceQueueCreateInfo {
                s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                p_next: std::ptr::null(),
                flags: vk::DeviceQueueCreateFlags::empty(),
                queue_family_index: *qf,
                queue_count: 1,
                p_queue_priorities: queue_priority,
            })
            .collect::<Vec<_>>();

        let device_features = vk::PhysicalDeviceFeatures::default(); // defaults to all 0 (false)

        // let layer_names = get_validation_layer_names_as_ptrs();

        let layer_names = VALIDATION_LAYERS
            .iter()
            .map(|x| CString::new(*x).unwrap())
            .collect::<Vec<_>>();
        let layer_names = layer_names.iter().map(|x| x.as_ptr()).collect::<Vec<_>>();

        let create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::DeviceCreateFlags::empty(),
            queue_create_info_count: queue_create_infos.len() as u32,
            p_queue_create_infos: queue_create_infos.as_ptr(),
            enabled_layer_count: if ENABLE_VALIDATION_LAYERS {
                layer_names.len() as u32
            } else {
                0
            },
            pp_enabled_layer_names: if ENABLE_VALIDATION_LAYERS {
                layer_names.as_ptr()
            } else {
                std::ptr::null()
            },
            enabled_extension_count: 0,
            pp_enabled_extension_names: std::ptr::null(),
            p_enabled_features: &device_features,
        };

        let logical_device = unsafe {
            instance
                .create_device(physical_device, &create_info, None)
                .expect("failed to create logical device!")
        };

        let graphics_queue =
            unsafe { logical_device.get_device_queue(indices.graphics_family.unwrap(), 0) };

        let present_queue =
            unsafe { logical_device.get_device_queue(indices.present_family.unwrap(), 0) };

        (logical_device, graphics_queue, present_queue)
    }

    fn get_required_extensions(window: &Window) -> Vec<*const i8> {
        let mut extension_names = ash_window::enumerate_required_extensions(window).unwrap();

        if ENABLE_VALIDATION_LAYERS {
            extension_names.push(DebugUtils::name());
        }

        let extension_names_ptrs = extension_names
            .iter()
            .map(|x| {
                println!("\t{}", x.to_str().unwrap());
                x.as_ptr()
            })
            .collect::<Vec<*const i8>>();

        extension_names_ptrs
    }

    fn create_instance(entry: &ash::Entry, window: &Window) -> ash::Instance {
        if ENABLE_VALIDATION_LAYERS && !Self::check_validation_layer_support(entry) {
            panic!("validation layers requested but not available!");
        }

        let appname = CString::new("Hello triangle!").unwrap();
        let enginename = CString::new("No Engine.").unwrap();
        let appinfo = vk::ApplicationInfo {
            s_type: vk::StructureType::APPLICATION_INFO,
            p_next: std::ptr::null(),
            p_application_name: appname.as_ptr(),
            application_version: vk::make_version(1, 2, 0),
            p_engine_name: enginename.as_ptr(),
            engine_version: vk::make_version(1, 2, 0),
            api_version: vk::API_VERSION_1_2,
        };

        let debug_utils_create_info = populate_debug_messenger_create_info();

        let layer_names = VALIDATION_LAYERS
            .iter()
            .map(|x| CString::new(*x).unwrap())
            .collect::<Vec<_>>();
        let layer_names = layer_names.iter().map(|x| x.as_ptr()).collect::<Vec<_>>();

        println!("Available extensions");
        let extension_names = Self::get_required_extensions(window);

        let createinfo = vk::InstanceCreateInfo {
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
            p_next: if ENABLE_VALIDATION_LAYERS {
                &debug_utils_create_info as *const vk::DebugUtilsMessengerCreateInfoEXT
                    as *const c_void
            } else {
                std::ptr::null()
            },
            flags: vk::InstanceCreateFlags::empty(),
            p_application_info: &appinfo,
            enabled_layer_count: if ENABLE_VALIDATION_LAYERS {
                layer_names.len() as u32
            } else {
                0
            },
            pp_enabled_layer_names: if ENABLE_VALIDATION_LAYERS {
                layer_names.as_ptr()
            } else {
                std::ptr::null()
            },
            enabled_extension_count: extension_names.len() as u32,
            pp_enabled_extension_names: extension_names.as_ptr(),
        };

        let instance = unsafe {
            entry
                .create_instance(&createinfo, None)
                .expect("failed to create instance!")
        };

        instance
    }

    fn main_loop(self, event_loop: EventLoop<()>) {
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => match input {
                        KeyboardInput {
                            virtual_keycode,
                            state,
                            ..
                        } => match (virtual_keycode, state) {
                            (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                                *control_flow = ControlFlow::Exit
                            }
                            _ => {}
                        },
                    },
                    _ => {}
                },
                _ => {}
            }
        })
    }
}

impl Drop for VkApp {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
            // this doesn't work??? doesn't complain when disabled.
            if ENABLE_VALIDATION_LAYERS {
                self.debug_utils
                    .destroy_debug_utils_messenger(self.debug_messenger, None);
            }
            self.surface_loader.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}

fn main() {
    let el = EventLoop::new();
    let win = VkApp::init_window(&el);
    let app = VkApp::init_vulkan(&win);
    app.main_loop(el);
}
