pub struct Surface {
    pub raw: ash::vk::SurfaceKHR,
    pub util: ash::extensions::khr::Surface,
}

impl Surface {
    pub fn destroy(&self) {
        unsafe { self.util.destroy_surface(self.raw, None) };
    }
}