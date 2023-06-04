pub struct Device {
    pub raw: ash::Device,
    pub graphic_queue: crate::QueueFamily,
    pub compute_queue: crate::QueueFamily,
    pub presentation_queue: crate::QueueFamily,
}

impl Device {
    pub unsafe fn create_image_view(&self, swapchain: &crate::Swapchain, image: ash::vk::Image) -> ash::vk::ImageView {
        let image_view_create_info = ash::vk::ImageViewCreateInfo {
            flags: ash::vk::ImageViewCreateFlags::default(),
            image: image,
            view_type: ash::vk::ImageViewType::TYPE_2D,
            format: swapchain.format,
            components: ash::vk::ComponentMapping {
                r: ash::vk::ComponentSwizzle::IDENTITY,
                g: ash::vk::ComponentSwizzle::IDENTITY,
                b: ash::vk::ComponentSwizzle::IDENTITY,
                a: ash::vk::ComponentSwizzle::IDENTITY,
            },
            subresource_range: ash::vk::ImageSubresourceRange {
                aspect_mask: ash::vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            ..Default::default()
        };
        
        self.raw.create_image_view(&image_view_create_info, None).unwrap()
    }

    pub unsafe fn create_shader(&self, code: &[u32]) -> ash::vk::ShaderModule {
        let shader_module_create_info = ash::vk::ShaderModuleCreateInfo::builder()
            .code(code);
        self.raw.create_shader_module(&shader_module_create_info, None).unwrap()
    }

    pub unsafe fn destroy(&self) {
        self.raw.destroy_device(None);
    }
}