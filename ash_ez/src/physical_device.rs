#[derive(Clone)]
pub struct PhysicalDevice {
    pub raw: ash::vk::PhysicalDevice,
    pub features: ash::vk::PhysicalDeviceFeatures,
    pub properties: ash::vk::PhysicalDeviceProperties,
    pub extensions_properties: Vec<ash::vk::ExtensionProperties>,
    pub memory_properties: ash::vk::PhysicalDeviceMemoryProperties,
    pub queue_family_properties: Vec<ash::vk::QueueFamilyProperties>,
}

impl PhysicalDevice {
    pub fn device_type(&self) -> ash::vk::PhysicalDeviceType {
        self.properties.device_type
    }

    pub fn is_discrete(&self) -> bool {
        self.device_type() == ash::vk::PhysicalDeviceType::DISCRETE_GPU
    }

    pub fn has_compute_queue(&self) -> bool {
        for queue_family in self.queue_family_properties.iter() {
            if queue_family.queue_flags.contains(ash::vk::QueueFlags::COMPUTE) {
                return true;
            }
        }
        return false
    }

    pub fn has_graphics_queue(&self) -> bool {
        for queue_family in self.queue_family_properties.iter() {
            if queue_family.queue_flags.contains(ash::vk::QueueFlags::GRAPHICS) {
                return true;
            }
        }
        return false
    }

    pub fn has_presentation_queue(&self, surface: &crate::Surface) -> bool {
        for queue_family in self.queue_family_properties.iter().enumerate() {
            unsafe { 
                if surface.util.get_physical_device_surface_support(self.raw, queue_family.0 as u32, surface.raw).unwrap() {
                    //return true
                }
            }
        }
        return false
    }

    pub fn get_graphic_queue_family_index(&self) -> Option<crate::QueueFamily> {
        for queue_family in self.queue_family_properties.iter().enumerate() {
            if queue_family.1.queue_flags.contains(ash::vk::QueueFlags::GRAPHICS) && queue_family.1.queue_count > 0 {
                let queue = crate::QueueFamily {
                    family_index: queue_family.0 as u32,
                    count: queue_family.1.queue_count,
                };
                return Some(queue);
            }
        }
        None
    }

    pub fn get_compute_queue_family_index(&self) -> Option<crate::QueueFamily> {
        for queue_family in self.queue_family_properties.iter().enumerate() {
            if queue_family.1.queue_flags.contains(ash::vk::QueueFlags::COMPUTE) && queue_family.1.queue_count > 0 {
                let queue = crate::QueueFamily {
                    family_index: queue_family.0 as u32,
                    count: queue_family.1.queue_count,
                };
                return Some(queue);
            }
        }
        None
    }

    pub fn get_presentation_queue_family_index(&self, surface: &crate::Surface) -> Option<crate::QueueFamily> {
        unsafe {
            for queue_family in self.queue_family_properties.iter().enumerate() {
                if surface.util.get_physical_device_surface_support(self.raw, queue_family.0 as u32, surface.raw).unwrap() && queue_family.1.queue_count > 0 {
                    let queue = crate::QueueFamily {
                        family_index: queue_family.0 as u32,
                        count: queue_family.1.queue_count,
                    };
                    return Some(queue);
                }
            }
            None
        }
    }

    pub fn limits(&self) -> ash::vk::PhysicalDeviceLimits {
        self.properties.limits
    }
}