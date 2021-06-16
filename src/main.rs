use std::ffi::CString;

use ash::version::EntryV1_0;
use ash::version::InstanceV1_0;
use ash::vk::make_version;
use ash::vk::ApplicationInfo;
// use ash::vk::Instance;
use ash::vk::InstanceCreateFlags;
use ash::vk::InstanceCreateInfo;
use ash::vk::StructureType;
use ash::vk::API_VERSION_1_0;
use ash::Entry;
use ash::Instance;
use ash_window::enumerate_required_extensions;
use winit::window::Window;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

struct VkApp {
    entry: Entry,
    instance: Instance,
}

impl VkApp {
    fn init_vulkan(window: &Window) -> Self {
        let entry = unsafe { Entry::new().unwrap() };
        let instance = VkApp::create_instance(&window, &entry);

        VkApp { entry, instance }
    }

    fn init_window(event_loop: &EventLoop<()>) -> Window {
        WindowBuilder::new()
            .with_inner_size(winit::dpi::LogicalSize::new(WIDTH, HEIGHT))
            .with_title("Vulkan")
            .build(&event_loop)
            .unwrap()
    }

    fn create_instance(window: &Window, entry: &Entry) -> Instance {
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

        let extension_names = enumerate_required_extensions(window)
            .unwrap()
            .iter()
            .map(|x| x.as_ptr())
            .collect::<Vec<*const i8>>();

        let createinfo = InstanceCreateInfo {
            s_type: StructureType::INSTANCE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: InstanceCreateFlags::empty(),
            p_application_info: &appinfo,
            enabled_layer_count: 0,
            pp_enabled_layer_names: std::ptr::null(),
            enabled_extension_count: extension_names.len() as u32,
            pp_enabled_extension_names: extension_names.as_ptr(),
        };

        let result = unsafe {
            entry
                .create_instance(&&createinfo, None)
                .expect("failed to create instance!")
        };

        result
    }

    fn main_loop(self, event_loop: EventLoop<()>) {
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => {}
                },
                _ => {}
            }
        })
    }
}

impl Drop for VkApp {
    fn drop(&mut self) {
        unsafe { self.instance.destroy_instance(None) }
    }
}

fn main() {
    let el = EventLoop::new();
    let win = VkApp::init_window(&el);
    let app = VkApp::init_vulkan(&win);
    app.main_loop(el);
}
