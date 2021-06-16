use ash::version::EntryV1_0;
use ash::version::InstanceV1_0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let entry = unsafe { ash::Entry::new()? };
    let instance = unsafe { entry.create_instance(&Default::default(), None)? };
    unsafe { instance.destroy_instance(None) };
    Ok(())
}
