use std::ffi::{c_void, CStr, CString};

use ash::extensions::ext::DebugUtils;
use ash::version::InstanceV1_0;
use ash::version::{DeviceV1_0, EntryV1_0};
use ash::vk::{make_version, DebugUtilsMessengerCreateFlagsEXT};
use ash::vk::{ApplicationInfo, DebugUtilsMessengerCreateInfoEXT};
use ash::vk::{Bool32, QueueFlags};
use ash::vk::{
    DebugUtilsMessageSeverityFlagsEXT, DebugUtilsMessageTypeFlagsEXT,
    DebugUtilsMessengerCallbackDataEXT, DebugUtilsMessengerEXT,
};
use ash::vk::{DeviceCreateFlags, StructureType};
use ash::vk::{DeviceCreateInfo, InstanceCreateInfo};
use ash::vk::{DeviceQueueCreateFlags, DeviceQueueCreateInfo, InstanceCreateFlags};
use ash::vk::{PhysicalDevice, PhysicalDeviceFeatures};
use ash::vk::{Queue, API_VERSION_1_0};
use ash::Device;
use ash::Entry;
use ash::Instance;
use vk::vk_to_str;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};
use winit::window::Window;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
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
    _entry: Entry,
    instance: Instance,
    debug_utils: DebugUtils,
    debug_messenger: DebugUtilsMessengerEXT,
    device: Device,
}

struct QueueFamilyIndices {
    graphics_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some()
    }
}

unsafe extern "system" fn debug_callback(
    message_severity: DebugUtilsMessageSeverityFlagsEXT,
    message_type: DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> Bool32 {
    let message = CStr::from_ptr((*p_callback_data).p_message);

    println!(
        "[DEBUG] [{:?}] [{:?}] {:?}",
        message_severity, message_type, message
    );

    ash::vk::FALSE
}

fn populate_debug_messenger_create_info() -> DebugUtilsMessengerCreateInfoEXT {
    DebugUtilsMessengerCreateInfoEXT {
        s_type: StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
        p_next: std::ptr::null(),
        flags: DebugUtilsMessengerCreateFlagsEXT::empty(),
        message_severity: DebugUtilsMessageSeverityFlagsEXT::ERROR
            | DebugUtilsMessageSeverityFlagsEXT::WARNING
            | DebugUtilsMessageSeverityFlagsEXT::VERBOSE
            | DebugUtilsMessageSeverityFlagsEXT::INFO,
        message_type: DebugUtilsMessageTypeFlagsEXT::GENERAL
            | DebugUtilsMessageTypeFlagsEXT::VALIDATION
            | DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
        pfn_user_callback: Some(debug_callback),
        p_user_data: std::ptr::null_mut(),
    }
}

// does not like this. just retype the function every time.
// fn get_validation_layer_names_as_ptrs() -> Vec<*const i8> {

//     // don't combine the maps. does not like that.
//     let layer_names = VALIDATION_LAYERS
//         .iter()
//         .map(|x| CString::new(*x).unwrap())
//         .collect::<Vec<_>>();
//     let layer_names = layer_names.iter().map(|x| x.as_ptr()).collect::<Vec<_>>();

//     layer_names
// }

impl VkApp {
    fn init_vulkan() -> Self {
        let entry = unsafe { Entry::new().unwrap() };
        let instance = Self::create_instance(&entry);
        let (debug_utils, debug_messenger) = Self::setup_debug_messenger(&entry, &instance);
        let physical_device = Self::pick_physical_device(&instance);
        let (logical_device, graphics_queue) =
            Self::create_logical_device(&instance, physical_device);

        VkApp {
            _entry: entry,
            instance,
            debug_utils,
            debug_messenger,
            device: logical_device,
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
        entry: &Entry,
        instance: &Instance,
    ) -> (DebugUtils, DebugUtilsMessengerEXT) {
        let debug_utils = DebugUtils::new(entry, instance);

        if !ENABLE_VALIDATION_LAYERS {
            return (debug_utils, DebugUtilsMessengerEXT::null());
        }

        let messenger_create_info = populate_debug_messenger_create_info();

        let debug_utils_messenger = unsafe {
            debug_utils
                .create_debug_utils_messenger(&messenger_create_info, None)
                .expect("could not create debug messenger")
        };

        (debug_utils, debug_utils_messenger)
    }

    fn check_validation_layer_support(entry: &Entry) -> bool {
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

    pub fn pick_physical_device(instance: &Instance) -> PhysicalDevice {
        let devices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("could not enumerate physical devices")
        };

        if devices.len() == 0 {
            panic!("failed to find GPUs with Vulkan support!");
        }

        for device in devices {
            if Self::is_device_suitable(instance, device) {
                return device;
            }
        }

        panic!("failed to find GPUs with Vulkan support!");
    }

    pub fn is_device_suitable(instance: &Instance, device: PhysicalDevice) -> bool {
        // let mut device_properties = unsafe { instance.get_physical_device_properties(device) };
        // let mut device_features = unsafe { instance.get_physical_device_features(device) };

        let indices = Self::find_queue_family(instance, device);

        indices.is_complete()
    }

    pub fn find_queue_family(instance: &Instance, device: PhysicalDevice) -> QueueFamilyIndices {
        let mut indices = QueueFamilyIndices {
            graphics_family: None,
        };
        let queue_families_properties =
            unsafe { instance.get_physical_device_queue_family_properties(device) };

        for (i, qf) in queue_families_properties.iter().enumerate() {
            if qf.queue_flags.contains(QueueFlags::GRAPHICS) {
                indices.graphics_family = Some(i as u32);
                break;
            }
        }

        indices
    }

    pub fn create_logical_device(
        instance: &Instance,
        physical_device: PhysicalDevice,
    ) -> (Device, Queue) {
        let indices = Self::find_queue_family(instance, physical_device);

        let queue_priority = &1_f32 as *const f32;

        let queue_create_info = DeviceQueueCreateInfo {
            s_type: StructureType::DEVICE_QUEUE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: DeviceQueueCreateFlags::empty(),
            queue_family_index: indices.graphics_family.unwrap(),
            queue_count: 1,
            p_queue_priorities: queue_priority,
        };

        let device_features = PhysicalDeviceFeatures::default(); // defaults to all 0 (false)

        // let layer_names = get_validation_layer_names_as_ptrs();

        let layer_names = VALIDATION_LAYERS
            .iter()
            .map(|x| CString::new(*x).unwrap())
            .collect::<Vec<_>>();
        let layer_names = layer_names.iter().map(|x| x.as_ptr()).collect::<Vec<_>>();

        let create_info = DeviceCreateInfo {
            s_type: StructureType::DEVICE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: DeviceCreateFlags::empty(),
            queue_create_info_count: 1,
            p_queue_create_infos: &queue_create_info,
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

        (logical_device, graphics_queue)
    }

    pub fn get_required_extensions() -> Vec<*const i8> {
        vec![
            ash::extensions::khr::Surface::name().as_ptr(),
            ash::extensions::khr::XlibSurface::name().as_ptr(),
            ash::extensions::ext::DebugUtils::name().as_ptr(),
        ]
    }

    fn create_instance(entry: &Entry) -> Instance {
        if ENABLE_VALIDATION_LAYERS && !Self::check_validation_layer_support(entry) {
            panic!("validation layers requested but not available!");
        }

        let appname = CString::new("Hello triangle!").unwrap();
        let enginename = CString::new("No Engine.").unwrap();
        let appinfo = ApplicationInfo {
            s_type: StructureType::APPLICATION_INFO,
            p_next: std::ptr::null(),
            p_application_name: appname.as_ptr(),
            application_version: make_version(1, 0, 0),
            p_engine_name: enginename.as_ptr(),
            engine_version: make_version(1, 0, 0),
            api_version: API_VERSION_1_0,
        };

        let debug_utils_create_info = populate_debug_messenger_create_info();

        // don't combine these two maps. it doesn't work.
        // let layer_names = VALIDATION_LAYERS
        //     .iter()
        //     .map(|x| CString::new(*x).unwrap())
        //     .collect::<Vec<_>>();
        // let layer_names = layer_names.iter().map(|x| x.as_ptr()).collect::<Vec<_>>();

        // let layer_names = get_validation_layer_names_as_ptrs();
        let layer_names = VALIDATION_LAYERS
            .iter()
            .map(|x| CString::new(*x).unwrap())
            .collect::<Vec<_>>();
        let layer_names = layer_names.iter().map(|x| x.as_ptr()).collect::<Vec<_>>();

        let extension_names = Self::get_required_extensions();

        let createinfo = InstanceCreateInfo {
            s_type: StructureType::INSTANCE_CREATE_INFO,
            p_next: if ENABLE_VALIDATION_LAYERS {
                &debug_utils_create_info as *const DebugUtilsMessengerCreateInfoEXT as *const c_void
            } else {
                std::ptr::null()
            },
            flags: InstanceCreateFlags::empty(),
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
            self.instance.destroy_instance(None);
        }
    }
}

fn main() {
    let el = EventLoop::new();
    let _win = VkApp::init_window(&el);
    let app = VkApp::init_vulkan();
    app.main_loop(el);
}
