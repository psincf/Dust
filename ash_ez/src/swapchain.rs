pub struct Swapchain {
    pub raw: ash::vk::SwapchainKHR,
    pub extent: ash::vk::Extent2D,
    pub format: ash::vk::Format,
    pub images_view: Vec<ash::vk::ImageView>,
    pub util: ash::extensions::khr::Swapchain,
}

impl Swapchain {
    pub unsafe fn update_images_views(&mut self, device: &crate::Device) {
        let images = self.get_swapchain_images();
        self.destroy_image_views(device);
        self.images_view.clear();
        for image in images {
            self.images_view.push(device.create_image_view(self, image));
        }
    }

    pub unsafe fn get_swapchain_images(&self) -> Vec<ash::vk::Image> {
        self.util.get_swapchain_images(self.raw).unwrap()
    }

    pub fn destroy_image_views(&mut self, device: &crate::Device) {
        for image in self.images_view.iter() {
            unsafe { device.raw.destroy_image_view(*image, None) }
        }
        self.images_view.clear();
    }

    pub fn destroy(&self) {
        unsafe { self.util.destroy_swapchain(self.raw, None) };
    }
}