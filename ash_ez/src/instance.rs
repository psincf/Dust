use std::ffi::CStr;
use std::ffi::CString;

pub struct Instance {
    pub entry: ash::Entry,
    pub raw: ash::Instance,
}

impl Instance {
    pub unsafe fn new() -> Instance {
        Instance::new_custom(ash::vk::make_api_version(0, 1, 2, 0), Vec::new(), Vec::new())
    }

    pub unsafe fn new_custom(api_version: u32, extensions_requested: Vec<&str>, layers_requested: Vec<&str>) -> Instance {
        let extensions_requested: Vec<CString> = extensions_requested.iter().map(|extension_str| CString::new(*extension_str).unwrap()).collect();
        let layers_requested: Vec<CString> = layers_requested.iter().map(|layer_str| CString::new(*layer_str).unwrap()).collect();

        let extensions_requested_raw: Vec<*const i8> = extensions_requested.iter().map(|extension_str| extension_str.as_bytes_with_nul() as *const [u8] as *const i8).collect();
        let layers_requested_raw: Vec<*const i8> = layers_requested.iter().map(|layer_str| layer_str.as_bytes_with_nul() as *const [u8] as *const i8).collect();
        
        let entry = ash::Entry::load().unwrap();

        let application_info = ash::vk::ApplicationInfo::builder()
            .api_version(api_version);

        match check_instance_extensions(&entry, &extensions_requested) {
            Ok(_) => {}
            Err(extensions) => {
                panic!("The extension: {:?} is not available", extensions.first().unwrap());
            }
        }


        match check_instance_layers(&entry, &layers_requested) {
            Ok(_) => {}
            Err(layers) => {
                panic!("The layer: {:?} is not available", layers.first().unwrap());
            }
        }

        let instance_create_info = ash::vk::InstanceCreateInfo::builder()
            .application_info(&application_info)
            .enabled_extension_names(&extensions_requested_raw)
            .enabled_layer_names(&layers_requested_raw);

        let instance = entry.create_instance(&instance_create_info, None).unwrap();
        
        Instance {
            entry,
            raw: instance,
        }
    }

    pub unsafe fn enumerate_physical_devices(&self) -> Vec<crate::PhysicalDevice> {
        let mut physical_devices = Vec::new();
        let physical_devices_raw = self.raw.enumerate_physical_devices().unwrap();
        for physical_device in physical_devices_raw {
            let features = self.raw.get_physical_device_features(physical_device);
            let properties = self.raw.get_physical_device_properties(physical_device);
            let extensions_properties = self.raw.enumerate_device_extension_properties(physical_device).unwrap();
            let memory_properties = self.raw.get_physical_device_memory_properties(physical_device);
            let queue_family_properties = self.raw.get_physical_device_queue_family_properties(physical_device);
            physical_devices.push(crate::PhysicalDevice {
                raw: physical_device,
                features: features,
                properties: properties,
                extensions_properties: extensions_properties,
                memory_properties: memory_properties,
                queue_family_properties: queue_family_properties,
            })
        }
        
        return physical_devices;
    }

    pub unsafe fn create_device(&self, physical_device: &crate::PhysicalDevice, surface: &crate::Surface, features: ash::vk::PhysicalDeviceFeatures, extensions: Vec<&str>, layers: Vec<&str>) -> crate::Device {
        let layers: Vec<CString> = layers.iter().map(|layer_str| CString::new(*layer_str).unwrap()).collect();
        let layers_raw: Vec<*const i8> = layers.iter().map(|layer_str| layer_str.as_bytes_with_nul() as *const [u8] as *const i8).collect();
        
        let extensions: Vec<CString> = extensions.iter().map(|extension_str| CString::new(*extension_str).unwrap()).collect();
        let extensions_raw: Vec<*const i8> = extensions.iter().map(|layer_str| layer_str.as_bytes_with_nul() as *const [u8] as *const i8).collect();
        
        let extensions_available = self.raw.enumerate_device_extension_properties(physical_device.raw).unwrap();
        for extension_requested in extensions.iter() {
            let mut ok = false;
            for extension_available in extensions_available.iter() {
                if extension_requested.as_c_str() == CStr::from_ptr(&extension_available.extension_name as * const i8) { ok = true; }
            }
            if !ok { panic!("The extension: {:?} is not available", extension_requested); }
        }

        let graphic_queue_create_info = ash::vk::DeviceQueueCreateInfo {
            queue_family_index: physical_device.get_graphic_queue_family_index().unwrap().family_index,
            queue_count: 1,
            p_queue_priorities: &1.0,
            ..Default::default()
        };
        
        let compute_queue_create_info = ash::vk::DeviceQueueCreateInfo {
            queue_family_index: physical_device.get_compute_queue_family_index().unwrap().family_index,
            queue_count: 1,
            p_queue_priorities: &1.0,
            ..Default::default()
        };

        let presentation_queue_create_info = ash::vk::DeviceQueueCreateInfo {
            queue_family_index: physical_device.get_presentation_queue_family_index(surface).unwrap().family_index,
            queue_count: 1,
            p_queue_priorities: &1.0,
            ..Default::default()
        };

        let queue_create_infos = vec![graphic_queue_create_info, compute_queue_create_info, presentation_queue_create_info];

        // Remove queues with same index
        let mut queue_families = std::collections::HashSet::new();
        let queue_create_infos: Vec<ash::vk::DeviceQueueCreateInfo> = queue_create_infos.iter().filter_map(|queue| if queue_families.insert(queue.queue_family_index) { Some(*queue) } else { None }).collect();

        let device_create_info = ash::vk::DeviceCreateInfo::builder()
            .enabled_extension_names(&extensions_raw)
            .enabled_features(&features)
            .enabled_layer_names(&layers_raw)
            .queue_create_infos(&queue_create_infos);
            

        let device_raw = self.raw.create_device(physical_device.raw, &device_create_info, None).unwrap();

        crate::Device {
            raw: device_raw,
            graphic_queue: physical_device.get_graphic_queue_family_index().unwrap(),
            compute_queue: physical_device.get_compute_queue_family_index().unwrap(),
            presentation_queue: physical_device.get_presentation_queue_family_index(&surface).unwrap(),
        }
    }

    pub unsafe fn create_surface(&self, window: &dyn raw_window_handle::HasRawWindowHandle) -> crate::Surface {
        let surface_khr = ash_window::create_surface(&self.entry, &self.raw, window, None).unwrap();

        crate::Surface {
            raw: surface_khr,
            util: ash::extensions::khr::Surface::new(&self.entry, &self.raw),
        }
    }

    pub unsafe fn create_swapchain(&self,
        physical_device: &crate::PhysicalDevice,
        device: &crate::Device,
        surface: &crate::Surface,
        present_mode: ash::vk::PresentModeKHR,
        window_size: (u32, u32)
    ) -> crate::Swapchain {
        let swapchain_util = ash::extensions::khr::Swapchain::new(&self.raw, &device.raw);

        let formats_available = surface.util.get_physical_device_surface_formats(physical_device.raw, surface.raw).unwrap();
        let mut format = None;
        for format_available in formats_available {
            if format_available.format == ash::vk::Format::B8G8R8A8_SRGB && format_available.color_space == ash::vk::ColorSpaceKHR::SRGB_NONLINEAR {
                format = Some(format_available);
            }
        }
        let format = format.unwrap();

        let present_modes_available = surface.util.get_physical_device_surface_present_modes(physical_device.raw, surface.raw).unwrap();
        let mut present_mode_option = None;
        for present_mode in present_modes_available {
            if present_mode == present_mode {
                present_mode_option = Some(present_mode);
            }
        }
        if present_mode_option.is_none() { panic!() }

        let extent = ash::vk::Extent2D {
            width: window_size.0,
            height: window_size.1
        };

        let sharing_mode;
        if device.graphic_queue.family_index == device.presentation_queue.family_index {
            sharing_mode = (ash::vk::SharingMode::EXCLUSIVE, 0, [device.graphic_queue.family_index, device.presentation_queue.family_index]);
        } else {
            sharing_mode = (ash::vk::SharingMode::CONCURRENT, 2, [device.graphic_queue.family_index, device.presentation_queue.family_index]);
        }

        let swapchain_create_info = ash::vk::SwapchainCreateInfoKHR {
            flags: ash::vk::SwapchainCreateFlagsKHR::empty(),
            surface: surface.raw,
            min_image_count: surface.util.get_physical_device_surface_capabilities(physical_device.raw, surface.raw).unwrap().min_image_count + 1,
            image_format: format.format,
            image_color_space: format.color_space,
            image_extent: extent,
            image_array_layers: 1,
            image_usage: ash::vk::ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode: sharing_mode.0,
            queue_family_index_count: sharing_mode.1,
            p_queue_family_indices: &sharing_mode.2 as *const _,
            pre_transform: ash::vk::SurfaceTransformFlagsKHR::IDENTITY,
            composite_alpha: ash::vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode: present_mode,
            clipped: ash::vk::TRUE,
            ..Default::default()
        };

        let swapchain = swapchain_util.create_swapchain(&swapchain_create_info, None).unwrap();

        crate::Swapchain {
            raw: swapchain,
            extent: extent,
            format: format.format,
            images_view: Vec::new(),
            util: swapchain_util,
        }
    }

    pub unsafe fn destroy(&self) {
        self.raw.destroy_instance(None);
    }
}




fn check_instance_extensions(entry: &ash::Entry, extensions_requested: &Vec<CString>) -> Result<(), Vec<CString>> {
    let extensions_available = entry.enumerate_instance_extension_properties(None).unwrap();
    let mut extensions_unavailable = Vec::new();

    for extension_requested in extensions_requested.iter() {
        let mut ok = false;
        for extension_available in extensions_available.iter() {
            if extension_requested.as_c_str() == unsafe { CStr::from_ptr(&extension_available.extension_name as * const i8) } { ok = true; }
        }
        if !ok { extensions_unavailable.push(extension_requested.clone()); }
    }

    if extensions_unavailable.len() == 0 {
        return Ok(());
    } else {
        return Err(extensions_unavailable);
    }
}

fn check_instance_layers(entry: &ash::Entry, layers_requested: &Vec<CString>) -> Result<(), Vec<CString>> {
    let layers_available = entry.enumerate_instance_layer_properties().unwrap();
    let mut layers_unavailable = Vec::new();

    for layer_requested in layers_requested.iter() {
        let mut ok = false;
        for layer_available in layers_available.iter() {
            if layer_requested.as_c_str() == unsafe { CStr::from_ptr(&layer_available.layer_name as * const i8) } { ok = true; }
        }
        if !ok { layers_unavailable.push(layer_requested.clone()); }
    }
    
    if layers_unavailable.len() == 0 {
        return Ok(());
    } else {
        return Err(layers_unavailable);
    }
}