//use crate::entity::*;
//use crate::world::*;

#[allow(unused)]
#[repr(packed(1))]
pub struct Uniform {
    window_size: (i32, i32),
    camera: (i32, i32),
    zoom: f32,
    alpha: f32,
    void: [u8; 8],
    color_base: (f32, f32, f32),
    void_2: [u8; 4],
    color_fast: (f32, f32, f32),
    color_ratio: f32,
}


pub struct ImguiRenderer {
    render_pass: ash::vk::RenderPass,
    renderer: Option<imgui_rs_vulkan_renderer::Renderer>,
}

impl ImguiRenderer {
    fn new(
        imgui: &mut imgui::Context,
        physical_device: &ash_ez::PhysicalDevice,
        instance: &ash_ez::Instance,
        device: &ash_ez::Device,
        swapchain: &ash_ez::Swapchain,
        command_pool: ash::vk::CommandPool,
    ) -> ImguiRenderer {

        let attachment_description = ash::vk::AttachmentDescription::builder()
            .format(swapchain.format)
            .samples(ash::vk::SampleCountFlags::TYPE_1)
            .load_op(ash::vk::AttachmentLoadOp::LOAD)
            .store_op(ash::vk::AttachmentStoreOp::STORE)
            .initial_layout(ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .final_layout(ash::vk::ImageLayout::PRESENT_SRC_KHR)
            .build();
    
        let attachment_descriptions = [attachment_description];

        let attachment_reference = ash::vk::AttachmentReference::builder()
            .attachment(0)
            .layout(ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build();
    
        let attachment_references = [attachment_reference];

        let subpass_description = ash::vk::SubpassDescription::builder()
            .pipeline_bind_point(ash::vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&attachment_references)
            .build();
        
        let subpass_descriptions = [subpass_description];

        let render_pass_info = ash::vk::RenderPassCreateInfo::builder()
            .attachments(&attachment_descriptions)
            .subpasses(&subpass_descriptions);
    
        let render_pass = unsafe { device.raw.create_render_pass(&render_pass_info, None).unwrap() };

        let renderer = imgui_rs_vulkan_renderer::Renderer::with_default_allocator(
            &instance.raw,
            physical_device.raw,
            device.raw.clone(),
            unsafe { device.raw.get_device_queue(device.graphic_queue.family_index, 0) },
            command_pool,

            render_pass,
            imgui,
            None
        ).unwrap();

        ImguiRenderer {
            render_pass: render_pass,
            renderer: Some(renderer),
        }
    }

    fn destroy(&mut self, device: &ash_ez::Device) {
        self.renderer = None;
        unsafe { device.raw.destroy_render_pass(self.render_pass, None); }
    }
}

pub struct GravityPipeline {
    vertex_shader: ash::vk::ShaderModule,
    fragment_shader: ash::vk::ShaderModule,
    render_pass: ash::vk::RenderPass,
    descriptor_set_layout: ash::vk::DescriptorSetLayout,
    pipeline_layout: ash::vk::PipelineLayout,
    pipeline: ash::vk::Pipeline,
}

impl GravityPipeline {
    pub fn create(device: &ash_ez::Device, swapchain: &ash_ez::Swapchain, window_size: winit::dpi::PhysicalSize<u32>) -> GravityPipeline {
        unsafe {
            let vertex_shader_raw_u8 = include_bytes!("../shaders/compiled/vertex_gravity.spv");
            let fragment_shader_raw_u8 = include_bytes!("../shaders/compiled/fragment_gravity.spv");
        
            let vertex_shader_raw = std::slice::from_raw_parts(vertex_shader_raw_u8.as_ptr() as *const u32, vertex_shader_raw_u8.len() / 4);
            let fragment_shader_raw = std::slice::from_raw_parts(fragment_shader_raw_u8.as_ptr() as *const u32, fragment_shader_raw_u8.len() / 4);
        
            let vertex_shader = device.create_shader(vertex_shader_raw);
            let fragment_shader = device.create_shader(fragment_shader_raw);
        
            let main_str = std::ffi::CString::new("main").unwrap();
        
            let stage_vertex = ash_ez::utils::pipeline_shader_stage_create_info_helper(vertex_shader, ash::vk::ShaderStageFlags::VERTEX, &main_str);
            let stage_fragment = ash_ez::utils::pipeline_shader_stage_create_info_helper(fragment_shader, ash::vk::ShaderStageFlags::FRAGMENT, &main_str);
            let stages = [stage_vertex, stage_fragment];
        
            let vertex_input_binding = ash::vk::VertexInputBindingDescription {
                binding: 0,
                stride: std::mem::size_of::<((f32, f32), (f32, f32), f32)>() as u32,
                input_rate: ash::vk::VertexInputRate::VERTEX,
            };
            let vertex_input_attribute = ash::vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: ash::vk::Format::R32G32_SINT,
                offset: 0,
            };
            let vertex_input_attribute_2 = ash::vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: ash::vk::Format::R32G32_SFLOAT,
                offset: 8,
            };
            let vertex_input_attribute_3 = ash::vk::VertexInputAttributeDescription {
                binding: 0,
                location: 2,
                format: ash::vk::Format::R32_SFLOAT,
                offset: 16,
            };
            let vertex_input_bindings = [vertex_input_binding];
            let vertex_input_attributes = [vertex_input_attribute, vertex_input_attribute_2, vertex_input_attribute_3];
            let vertex_input_state = ash::vk::PipelineVertexInputStateCreateInfo::builder()
                .vertex_binding_descriptions(&vertex_input_bindings)
                .vertex_attribute_descriptions(&vertex_input_attributes);
            
            let input_assembly_state = ash::vk::PipelineInputAssemblyStateCreateInfo::builder()
                .topology(ash::vk::PrimitiveTopology::POINT_LIST);
            
            let viewport = [ash_ez::utils::viewport_helper(window_size.width as f32, window_size.height as f32)];
            let scissor = [ash_ez::utils::scissor_helper(window_size.width, window_size.height)];
            let viewport_state = ash_ez::utils::pipeline_viewport_state_create_info_helper_2(&viewport, &scissor);
            let rasterization_state = ash_ez::utils::pipeline_rasterization_state_create_info_helper();
            let multisample_state = ash_ez::utils::pipeline_multisample_state_create_info_helper();
            let color_blend_attachment_state = ash_ez::utils::pipeline_color_blend_attachment_state_helper();
            let color_blend_attachment_states = [color_blend_attachment_state];
            let color_blend_state = ash::vk::PipelineColorBlendStateCreateInfo::builder()
                .attachments(&color_blend_attachment_states);

            let descriptor_set_layout_binding = ash::vk::DescriptorSetLayoutBinding::builder()
                .binding(0)
                .descriptor_type(ash::vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(ash::vk::ShaderStageFlags::VERTEX | ash::vk::ShaderStageFlags::FRAGMENT)
                .build();
            let bindings = [descriptor_set_layout_binding];

            let descriptor_set_layout_create_info = ash::vk::DescriptorSetLayoutCreateInfo::builder()
                .bindings(&bindings)
                .build();
            let descriptor_set_layout = device.raw.create_descriptor_set_layout(&descriptor_set_layout_create_info, None).unwrap();
            let descriptor_set_layouts = [descriptor_set_layout];

            let pipeline_layout_info = ash::vk::PipelineLayoutCreateInfo::builder()
                .set_layouts(&descriptor_set_layouts);
                
            let pipeline_layout = device.raw.create_pipeline_layout(&pipeline_layout_info, None).unwrap();
            
            let attachment_description = ash::vk::AttachmentDescription::builder()
                .format(swapchain.format)
                .samples(ash::vk::SampleCountFlags::TYPE_1)
                .load_op(ash::vk::AttachmentLoadOp::LOAD)
                .store_op(ash::vk::AttachmentStoreOp::STORE)
                .initial_layout(ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .final_layout(ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .build();
            
            let attachment_descriptions = [attachment_description];
        
            let attachment_reference = ash::vk::AttachmentReference::builder()
                .attachment(0)
                .layout(ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .build();
            
            let attachment_references = [attachment_reference];
            
            let subpass_description = ash::vk::SubpassDescription::builder()
                .pipeline_bind_point(ash::vk::PipelineBindPoint::GRAPHICS)
                .color_attachments(&attachment_references)
                .build();
                
            let subpass_descriptions = [subpass_description];
            
            let render_pass_info = ash::vk::RenderPassCreateInfo::builder()
                .attachments(&attachment_descriptions)
                .subpasses(&subpass_descriptions);
            
            let render_pass = device.raw.create_render_pass(&render_pass_info, None).unwrap();
        
            let pipeline_create_info = ash::vk::GraphicsPipelineCreateInfo::builder()
                .stages(&stages)
                .vertex_input_state(&vertex_input_state)
                .input_assembly_state(&input_assembly_state)
                .viewport_state(&viewport_state)
                .rasterization_state(&rasterization_state)
                .multisample_state(&multisample_state)
                //.depth_stencil_state(depth_stencil_state)
                .color_blend_state(&color_blend_state)
                .layout(pipeline_layout)
                .render_pass(render_pass)
                .subpass(0)
                .build();
        
            let pipeline_create_infos = [pipeline_create_info];
            let pipeline_cache = ash::vk::PipelineCache::null();
            let pipelines = device.raw.create_graphics_pipelines(pipeline_cache, &pipeline_create_infos, None).unwrap();
            let pipeline = pipelines[0];

            GravityPipeline {
                vertex_shader,
                fragment_shader,
                render_pass,
                descriptor_set_layout,
                pipeline_layout,
                pipeline,
            }
        }
    }

    fn destroy(&mut self, device: &ash_ez::Device) {
        unsafe {
            device.raw.destroy_pipeline(self.pipeline, None);
            device.raw.destroy_pipeline_layout(self.pipeline_layout, None);
            device.raw.destroy_render_pass(self.render_pass, None);
            device.raw.destroy_shader_module(self.vertex_shader, None);
            device.raw.destroy_shader_module(self.fragment_shader, None);
            device.raw.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
        }
    }
}

pub struct ParticlesPipeline {
    vertex_shader: ash::vk::ShaderModule,
    fragment_shader: ash::vk::ShaderModule,
    render_pass: ash::vk::RenderPass,
    descriptor_set_layout: ash::vk::DescriptorSetLayout,
    pipeline_layout: ash::vk::PipelineLayout,
    pipeline: ash::vk::Pipeline,
}

impl ParticlesPipeline {
    fn create(device: &ash_ez::Device, swapchain: &ash_ez::Swapchain, window_size: winit::dpi::PhysicalSize<u32>) -> ParticlesPipeline {
        unsafe {
            let vertex_shader_raw_u8 = include_bytes!("../shaders/compiled/vertex.spv");
            let fragment_shader_raw_u8 = include_bytes!("../shaders/compiled/fragment.spv");
        
            let vertex_shader_raw = std::slice::from_raw_parts(vertex_shader_raw_u8.as_ptr() as *const u32, vertex_shader_raw_u8.len() / 4);
            let fragment_shader_raw = std::slice::from_raw_parts(fragment_shader_raw_u8.as_ptr() as *const u32, fragment_shader_raw_u8.len() / 4);
        
            let vertex_shader = device.create_shader(vertex_shader_raw);
            let fragment_shader = device.create_shader(fragment_shader_raw);
        
            let main_str = std::ffi::CString::new("main").unwrap();
        
            let stage_vertex = ash_ez::utils::pipeline_shader_stage_create_info_helper(vertex_shader, ash::vk::ShaderStageFlags::VERTEX, &main_str);
            let stage_fragment = ash_ez::utils::pipeline_shader_stage_create_info_helper(fragment_shader, ash::vk::ShaderStageFlags::FRAGMENT, &main_str);
            let stages = [stage_vertex, stage_fragment];
        
            let vertex_input_binding = ash::vk::VertexInputBindingDescription {
                binding: 0,
                stride: std::mem::size_of::<((f32, f32), (f32, f32))>() as u32,
                input_rate: ash::vk::VertexInputRate::VERTEX,
            };
            let vertex_input_attribute = ash::vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: ash::vk::Format::R32G32_SINT,
                offset: 0,
            };
            let vertex_input_attribute_2 = ash::vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: ash::vk::Format::R32G32_SFLOAT,
                offset: 8,
            };
            let vertex_input_bindings = [vertex_input_binding];
            let vertex_input_attributes = [vertex_input_attribute, vertex_input_attribute_2];
            let vertex_input_state = ash::vk::PipelineVertexInputStateCreateInfo::builder()
                .vertex_binding_descriptions(&vertex_input_bindings)
                .vertex_attribute_descriptions(&vertex_input_attributes);
            
            let input_assembly_state = ash::vk::PipelineInputAssemblyStateCreateInfo::builder()
                .topology(ash::vk::PrimitiveTopology::POINT_LIST);
            
            let viewport = [ash_ez::utils::viewport_helper(window_size.width as f32, window_size.height as f32)];
            let scissor = [ash_ez::utils::scissor_helper(window_size.width, window_size.height)];
            let viewport_state = ash_ez::utils::pipeline_viewport_state_create_info_helper_2(&viewport, &scissor);
            let rasterization_state = ash_ez::utils::pipeline_rasterization_state_create_info_helper();
            let multisample_state = ash_ez::utils::pipeline_multisample_state_create_info_helper();
            let color_blend_attachment_state = ash_ez::utils::pipeline_color_blend_attachment_state_helper();
            let color_blend_attachment_states = [color_blend_attachment_state];
            let color_blend_state = ash::vk::PipelineColorBlendStateCreateInfo::builder()
                .attachments(&color_blend_attachment_states);

            let descriptor_set_layout_binding = ash::vk::DescriptorSetLayoutBinding::builder()
                .binding(0)
                .descriptor_type(ash::vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(ash::vk::ShaderStageFlags::VERTEX | ash::vk::ShaderStageFlags::FRAGMENT)
                .build();
            let bindings = [descriptor_set_layout_binding];

            let descriptor_set_layout_create_info = ash::vk::DescriptorSetLayoutCreateInfo::builder()
                .bindings(&bindings)
                .build();
            let descriptor_set_layout = device.raw.create_descriptor_set_layout(&descriptor_set_layout_create_info, None).unwrap();
            let descriptor_set_layouts = [descriptor_set_layout];

            let pipeline_layout_info = ash::vk::PipelineLayoutCreateInfo::builder()
                .set_layouts(&descriptor_set_layouts);
                
            let pipeline_layout = device.raw.create_pipeline_layout(&pipeline_layout_info, None).unwrap();
            
            let attachment_description = ash::vk::AttachmentDescription::builder()
                .format(swapchain.format)
                .samples(ash::vk::SampleCountFlags::TYPE_1)
                .load_op(ash::vk::AttachmentLoadOp::CLEAR)
                .store_op(ash::vk::AttachmentStoreOp::STORE)
                .initial_layout(ash::vk::ImageLayout::UNDEFINED)
                .final_layout(ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .build();
            
            let attachment_descriptions = [attachment_description];
        
            let attachment_reference = ash::vk::AttachmentReference::builder()
                .attachment(0)
                .layout(ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .build();
            
            let attachment_references = [attachment_reference];
            
            let subpass_description = ash::vk::SubpassDescription::builder()
                .pipeline_bind_point(ash::vk::PipelineBindPoint::GRAPHICS)
                .color_attachments(&attachment_references)
                .build();
                
            let subpass_descriptions = [subpass_description];
            
            let render_pass_info = ash::vk::RenderPassCreateInfo::builder()
                .attachments(&attachment_descriptions)
                .subpasses(&subpass_descriptions);
            
            let render_pass = device.raw.create_render_pass(&render_pass_info, None).unwrap();
        
            let pipeline_create_info = ash::vk::GraphicsPipelineCreateInfo::builder()
                .stages(&stages)
                .vertex_input_state(&vertex_input_state)
                .input_assembly_state(&input_assembly_state)
                .viewport_state(&viewport_state)
                .rasterization_state(&rasterization_state)
                .multisample_state(&multisample_state)
                //.depth_stencil_state(depth_stencil_state)
                .color_blend_state(&color_blend_state)
                .layout(pipeline_layout)
                .render_pass(render_pass)
                .subpass(0)
                .build();
        
            let pipeline_create_infos = [pipeline_create_info];
            let pipeline_cache = ash::vk::PipelineCache::null();
            let pipelines = device.raw.create_graphics_pipelines(pipeline_cache, &pipeline_create_infos, None).unwrap();
            let pipeline = pipelines[0];

            ParticlesPipeline {
                vertex_shader,
                fragment_shader,
                render_pass,
                descriptor_set_layout,
                pipeline_layout,
                pipeline,
            }
        }
    }

    fn destroy(&mut self, device: &ash_ez::Device) {
        unsafe {
            device.raw.destroy_pipeline(self.pipeline, None);
            device.raw.destroy_pipeline_layout(self.pipeline_layout, None);
            device.raw.destroy_render_pass(self.render_pass, None);
            device.raw.destroy_shader_module(self.vertex_shader, None);
            device.raw.destroy_shader_module(self.fragment_shader, None);
            device.raw.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
        }
    }
}

pub struct Renderer {
    physical_device: ash_ez::PhysicalDevice,
    instance: ash_ez::Instance,
    surface: ash_ez::Surface,
    device: ash_ez::Device,
    swapchain: ash_ez::Swapchain,
    command_pool: ash::vk::CommandPool,

    vertex_buffer_size: u64,
    vertex_buffer: ash::vk::Buffer,
    vertex_buffer_memory: ash::vk::DeviceMemory,

    staging_buffer_size: u64,
    staging_buffer: ash::vk::Buffer,
    staging_buffer_memory: ash::vk::DeviceMemory,

    uniform_buffer: ash::vk::Buffer,
    uniform_buffer_memory: ash::vk::DeviceMemory,

    particles_pipeline: ParticlesPipeline,
    gravity_pipeline: GravityPipeline,

    actual_image_index: u32,

    imgui_renderer: ImguiRenderer,
}

impl Renderer {
    pub fn new(window: &winit::window::Window, imgui: &mut imgui::Context) -> Renderer {
        unsafe {
            let window_size = window.inner_size();
            let extensions_surface = ash_window::enumerate_required_extensions(window).unwrap();
            let extensions_surface = extensions_surface.iter().map(|ext| ext.to_str().unwrap()).collect();
        
            //let instance =  ash_ez::Instance::new_custom(ash::vk::make_api_version(0, 1, 2, 0), extensions_surface, vec!["VK_LAYER_KHRONOS_validation"]);
            let instance =  ash_ez::Instance::new_custom(ash::vk::make_api_version(0, 1, 2, 0), extensions_surface, vec![]);
            let physical_devices = instance.enumerate_physical_devices();
            let physical_device = physical_devices.iter()
                .filter(|physical_device| 
                    physical_device.has_graphics_queue() && 
                    physical_device.is_discrete() &&
                    ash::vk::api_version_major(physical_device.properties.api_version) >= 1 &&
                    ash::vk::api_version_minor(physical_device.properties.api_version) >= 2
                )
                .find(|_| true).unwrap().clone();
        
            let surface = instance.create_surface(window);
            let device = instance.create_device(&physical_device, &surface, ash::vk::PhysicalDeviceFeatures::builder().build(), vec!["VK_KHR_swapchain"] , vec!["VK_LAYER_KHRONOS_validation"]);
            let mut swapchain = instance.create_swapchain(&physical_device, &device, &surface, ash::vk::PresentModeKHR::MAILBOX, window_size.into());
            swapchain.update_images_views(&device);
        

            let vertex_buffer_info = ash::vk::BufferCreateInfo::builder()
                .size(1_000_000)
                .usage(ash::vk::BufferUsageFlags::VERTEX_BUFFER | ash::vk::BufferUsageFlags::TRANSFER_DST)
                .sharing_mode(ash::vk::SharingMode::EXCLUSIVE);
            let vertex_buffer = unsafe { device.raw.create_buffer(&vertex_buffer_info, None).unwrap() };
            let vertex_buffer_memory_requirements = unsafe { device.raw.get_buffer_memory_requirements(vertex_buffer) };
            let memory_requirements: (usize, &ash::vk::MemoryType) = physical_device.memory_properties.memory_types
                .iter()
                .enumerate()
                .find(|(index, mem)| {
                    mem.property_flags.intersects(ash::vk::MemoryPropertyFlags::DEVICE_LOCAL) &&
                    ((1 << *index) & vertex_buffer_memory_requirements.memory_type_bits != 0)
                })
                .unwrap();
            let vertex_allocate_info = ash::vk::MemoryAllocateInfo::builder()
                .allocation_size(vertex_buffer_memory_requirements.size)
                .memory_type_index(memory_requirements.0 as u32);
            let vertex_allocated_memory = unsafe { device.raw.allocate_memory(&vertex_allocate_info, None).unwrap() };

            let staging_buffer_info = ash::vk::BufferCreateInfo::builder()
                .size(1_000_000)
                .usage(ash::vk::BufferUsageFlags::TRANSFER_SRC)
                .sharing_mode(ash::vk::SharingMode::EXCLUSIVE);
            let staging_buffer = unsafe { device.raw.create_buffer(&staging_buffer_info, None).unwrap() };
            let staging_buffer_memory_requirements = unsafe { device.raw.get_buffer_memory_requirements(staging_buffer) };
            let memory_requirements: (usize, &ash::vk::MemoryType) = physical_device.memory_properties.memory_types
                .iter()
                .enumerate()
                .find(|(index, mem)| {
                    mem.property_flags.intersects(ash::vk::MemoryPropertyFlags::HOST_VISIBLE | ash::vk::MemoryPropertyFlags::HOST_COHERENT) &&
                    ((1 << *index) & staging_buffer_memory_requirements.memory_type_bits != 0)
                })
                .unwrap();
            let staging_allocate_info = ash::vk::MemoryAllocateInfo::builder()
                .allocation_size(staging_buffer_memory_requirements.size)
                .memory_type_index(memory_requirements.0 as u32);
            let staging_allocated_memory = unsafe { device.raw.allocate_memory(&staging_allocate_info, None).unwrap() };


            let command_pool_info = ash::vk::CommandPoolCreateInfo::builder()
                .queue_family_index(device.graphic_queue.family_index);
            let command_pool = device.raw.create_command_pool(&command_pool_info, None).unwrap();

            let uniform_buffer_info = ash::vk::BufferCreateInfo::builder()
                .size(std::mem::size_of::<Uniform>() as u64)
                .usage(ash::vk::BufferUsageFlags::UNIFORM_BUFFER)
                .sharing_mode(ash::vk::SharingMode::EXCLUSIVE);
            let uniform_buffer = device.raw.create_buffer(&uniform_buffer_info, None).unwrap();
            
            let uniform_buffer_memory_requirements = device.raw.get_buffer_memory_requirements(uniform_buffer);
            let memory_requirements: (usize, &ash::vk::MemoryType) = physical_device.memory_properties.memory_types
                .iter()
                .enumerate()
                .find(|(index, mem)| {
                    mem.property_flags.intersects(ash::vk::MemoryPropertyFlags::HOST_VISIBLE | ash::vk::MemoryPropertyFlags::HOST_COHERENT) &&
                    ((1 << *index) & uniform_buffer_memory_requirements.memory_type_bits != 0)
                })
                .unwrap();
            let uniform_allocate_info = ash::vk::MemoryAllocateInfo::builder()
                .allocation_size(uniform_buffer_memory_requirements.size)
                .memory_type_index(memory_requirements.0 as u32);
            let uniform_allocated_memory = device.raw.allocate_memory(&uniform_allocate_info, None).unwrap();
            device.raw.bind_buffer_memory(uniform_buffer, uniform_allocated_memory, 0).unwrap();

            let particles_pipeline = ParticlesPipeline::create(&device, &swapchain, window_size);
            let gravity_pipeline = GravityPipeline::create(&device, &swapchain, window_size);

            let imgui_renderer = ImguiRenderer::new(imgui, &physical_device, &instance, &device, &swapchain, command_pool);



            Renderer {
                physical_device,
                instance,
                surface,
                device,
                swapchain,
                command_pool,

                vertex_buffer_size: 1_000_000,
                vertex_buffer: vertex_buffer,
                vertex_buffer_memory: vertex_allocated_memory,

                staging_buffer_size: 1_000_000,
                staging_buffer: staging_buffer,
                staging_buffer_memory: staging_allocated_memory,

                uniform_buffer: uniform_buffer,
                uniform_buffer_memory: uniform_allocated_memory,

                particles_pipeline,
                gravity_pipeline,

                actual_image_index: 0,

                imgui_renderer,
            }
        }
    }

    fn update_vertex_buffer_size(&mut self, new_size: u64) {
        if self.vertex_buffer_size >= new_size { return }

        unsafe { self.device.raw.free_memory(self.vertex_buffer_memory, None) };
        unsafe { self.device.raw.destroy_buffer(self.vertex_buffer, None) };

        let vertex_buffer_info = ash::vk::BufferCreateInfo::builder()
            .size(new_size)
            .usage(ash::vk::BufferUsageFlags::VERTEX_BUFFER | ash::vk::BufferUsageFlags::TRANSFER_DST)
            .sharing_mode(ash::vk::SharingMode::EXCLUSIVE);
        let vertex_buffer = unsafe { self.device.raw.create_buffer(&vertex_buffer_info, None).unwrap() };
        let vertex_buffer_memory_requirements = unsafe { self.device.raw.get_buffer_memory_requirements(vertex_buffer) };
        let memory_requirements: (usize, &ash::vk::MemoryType) = self.physical_device.memory_properties.memory_types
            .iter()
            .enumerate()
            .find(|(index, mem)| {
                mem.property_flags.intersects(ash::vk::MemoryPropertyFlags::DEVICE_LOCAL) &&
                ((1 << *index) & vertex_buffer_memory_requirements.memory_type_bits != 0)
            })
            .unwrap();
        let allocate_info = ash::vk::MemoryAllocateInfo::builder()
            .allocation_size(vertex_buffer_memory_requirements.size)
            .memory_type_index(memory_requirements.0 as u32);
        let allocated_memory = unsafe { self.device.raw.allocate_memory(&allocate_info, None).unwrap() };
        unsafe { self.device.raw.bind_buffer_memory(vertex_buffer, allocated_memory, 0).unwrap() };

        self.vertex_buffer_size = new_size;
        self.vertex_buffer_memory = allocated_memory;
        self.vertex_buffer = vertex_buffer;
    }

    fn update_staging_buffer_size(&mut self, new_size: u64) {
        if self.staging_buffer_size >= new_size { return }

        unsafe { self.device.raw.free_memory(self.staging_buffer_memory, None) };
        unsafe { self.device.raw.destroy_buffer(self.staging_buffer, None) };

        let staging_buffer_info = ash::vk::BufferCreateInfo::builder()
            .size(new_size)
            .usage(ash::vk::BufferUsageFlags::TRANSFER_SRC)
            .sharing_mode(ash::vk::SharingMode::EXCLUSIVE);
        let staging_buffer = unsafe { self.device.raw.create_buffer(&staging_buffer_info, None).unwrap() };
        let staging_buffer_memory_requirements = unsafe { self.device.raw.get_buffer_memory_requirements(staging_buffer) };
        let memory_requirements: (usize, &ash::vk::MemoryType) = self.physical_device.memory_properties.memory_types
            .iter()
            .enumerate()
            .find(|(index, mem)| {
                mem.property_flags.intersects(ash::vk::MemoryPropertyFlags::HOST_VISIBLE | ash::vk::MemoryPropertyFlags::HOST_COHERENT) &&
                ((1 << *index) & staging_buffer_memory_requirements.memory_type_bits != 0)
            })
            .unwrap();
        let allocate_info = ash::vk::MemoryAllocateInfo::builder()
            .allocation_size(staging_buffer_memory_requirements.size)
            .memory_type_index(memory_requirements.0 as u32);
        let allocated_memory = unsafe { self.device.raw.allocate_memory(&allocate_info, None).unwrap() };
        unsafe { self.device.raw.bind_buffer_memory(staging_buffer, allocated_memory, 0).unwrap() };

        self.staging_buffer_size = new_size;
        self.staging_buffer_memory = allocated_memory;
        self.staging_buffer = staging_buffer;
    }

    fn update_vertex_buffer(&mut self, data: &[u8]) {
        if data.len() == 0 { return }
        self.update_vertex_buffer_size(data.len() as u64);
        self.update_staging_buffer_size(data.len() as u64);
        unsafe {
            let mapped_memory = self.device.raw.map_memory(self.staging_buffer_memory, 0, self.staging_buffer_size, ash::vk::MemoryMapFlags::empty()).unwrap();
            let mapped_memory_type = std::mem::transmute::<*mut std::ffi::c_void, *mut u8>(mapped_memory);
            {
                std::ptr::copy(data.as_ptr(), mapped_memory_type, data.len());
            }
            self.device.raw.unmap_memory(self.staging_buffer_memory);

            let commander_buffer_info = ash::vk::CommandBufferAllocateInfo::builder()
                .command_pool(self.command_pool)
                .level(ash::vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(1);
            let command_buffer = self.device.raw.allocate_command_buffers(&commander_buffer_info).unwrap()[0];

            let buffer_copy = [ash::vk::BufferCopy {
                src_offset: 0,
                dst_offset: 0,
                size: data.len() as u64,
            }];

            let command_buffer_begin_info = ash::vk::CommandBufferBeginInfo::default();
            self.device.raw.begin_command_buffer(command_buffer, &command_buffer_begin_info).unwrap();
            self.device.raw.cmd_copy_buffer(command_buffer, self.staging_buffer, self.vertex_buffer, &buffer_copy);
            self.device.raw.end_command_buffer(command_buffer).unwrap();
            
            let command_buffers = [command_buffer];
            let submit_info = ash::vk::SubmitInfo::builder()
                .command_buffers(&command_buffers)
                .build();
            let submit_infos = [submit_info];
            let graphic_queue = self.device.raw.get_device_queue(self.device.graphic_queue.family_index, 0);

            let fence = self.device.raw.create_fence(&ash::vk::FenceCreateInfo::default(), None).unwrap();
            let fences = [fence];
            self.device.raw.queue_submit(graphic_queue, &submit_infos, fence).unwrap();
            self.device.raw.wait_for_fences(&fences, true, u64::MAX).unwrap();
            self.device.raw.destroy_fence(fence, None);
        }
    }

    fn update_uniform(&mut self, uniform: Uniform) {
        unsafe {
            let mapped_memory = self.device.raw.map_memory(self.uniform_buffer_memory, 0, std::mem::size_of::<Uniform>() as u64, ash::vk::MemoryMapFlags::empty()).unwrap();
            let mapped_memory_type = std::mem::transmute::<*mut std::ffi::c_void, *mut Uniform>(mapped_memory);
            std::ptr::copy(&uniform, mapped_memory_type, 1);
            self.device.raw.unmap_memory(self.uniform_buffer_memory);
        }
    }

    pub fn draw(&mut self, world: &crate::World) {
        unsafe {
            let fence_info = ash::vk::FenceCreateInfo::default();
            let fence = self.device.raw.create_fence(&fence_info, None).unwrap();
            let fences = [fence];
            let result = self.swapchain.util.acquire_next_image(self.swapchain.raw, u64::MAX, ash::vk::Semaphore::null(), fence);
            if result.is_err() { return }
            let (image_index, suboptimal) = result.unwrap();
            if suboptimal == true {
                
            }
            self.actual_image_index = image_index;
            self.device.raw.wait_for_fences(&fences, true, u64::MAX).unwrap();
            self.device.raw.destroy_fence(fence, None);

            let uniform = Uniform {
                window_size: (self.swapchain.extent.width as i32, self.swapchain.extent.height as i32),
                camera: world.position_camera.into(),
                zoom: world.zoom,
                alpha: world.settings.alpha,
                void: Default::default(),
                void_2: Default::default(),
                color_base: world.settings.color_base,
                color_fast: world.settings.color_fast,
                color_ratio: world.settings.color_ratio_speed,
            };

            self.update_uniform(uniform);

            let descriptor_pool_size = ash::vk::DescriptorPoolSize::builder()
                .ty(ash::vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .build();
            let descriptors_pool_size = [descriptor_pool_size];
            let descriptor_pool_create_info = ash::vk::DescriptorPoolCreateInfo::builder()
                .max_sets(1)
                .pool_sizes(&descriptors_pool_size);
            let descriptor_pool = self.device.raw.create_descriptor_pool(&descriptor_pool_create_info, None).unwrap();
            let descriptor_set_layouts = [self.particles_pipeline.descriptor_set_layout];
            let descriptor_set_allocate_info = ash::vk::DescriptorSetAllocateInfo::builder()
                .descriptor_pool(descriptor_pool)
                .set_layouts(&descriptor_set_layouts);
            let descriptor_set = self.device.raw.allocate_descriptor_sets(&descriptor_set_allocate_info).unwrap()[0];
            let descriptor_buffer_info = ash::vk::DescriptorBufferInfo::builder()
                .buffer(self.uniform_buffer)
                .offset(0)
                .range(std::mem::size_of::<Uniform>() as u64)
                .build();
            let descriptors_buffer_info = [descriptor_buffer_info];
            let mut write_descriptor_set = ash::vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_set)
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(ash::vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&descriptors_buffer_info)
                .build();
            write_descriptor_set.descriptor_count = 1;
            let write_descriptor_sets = [write_descriptor_set];
            self.device.raw.update_descriptor_sets(&write_descriptor_sets, &[]);

            let image_view_framebuffer = [self.swapchain.images_view[image_index as usize]];
            let framebuffer_create_info = ash::vk::FramebufferCreateInfo::builder()
                .render_pass(self.particles_pipeline.render_pass)
                .attachments(&image_view_framebuffer)
                .width(self.swapchain.extent.width)
                .height(self.swapchain.extent.height)
                .layers(1);
            let framebuffer = self.device.raw.create_framebuffer(&framebuffer_create_info, None).unwrap();

            self.draw_particles(framebuffer, descriptor_set, world);
            self.draw_gravity(framebuffer, descriptor_set, world);
            
            self.device.raw.destroy_descriptor_pool(descriptor_pool, None);
            self.device.raw.destroy_framebuffer(framebuffer, None);
        }
    }

    fn draw_particles(&mut self, framebuffer: ash::vk::Framebuffer, descriptor_set: ash::vk::DescriptorSet, world: &crate::World) {
        let num_entities = world.entities.len();
        let size_entity = std::mem::size_of::<crate::entity::Entity>();
        unsafe {
            let data_u8_ptr = std::mem::transmute::<*const crate::entity::Entity, *const u8>(world.entities.as_ptr());
            let data_u8 = std::slice::from_raw_parts(data_u8_ptr, num_entities * size_entity);
            self.update_vertex_buffer(data_u8);

            let commander_buffer_info = ash::vk::CommandBufferAllocateInfo::builder()
                .command_pool(self.command_pool)
                .level(ash::vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(1);
            let command_buffer = self.device.raw.allocate_command_buffers(&commander_buffer_info).unwrap()[0];
            let command_buffer_begin_info = ash::vk::CommandBufferBeginInfo::default();
            self.device.raw.begin_command_buffer(command_buffer, &command_buffer_begin_info).unwrap();
            let clear_values = [ash::vk::ClearValue {
                color: ash::vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 0.0],
                }
            }];
            let render_pass_begin_info = ash::vk::RenderPassBeginInfo::builder()
                .render_pass(self.particles_pipeline.render_pass)
                .framebuffer(framebuffer)
                .render_area(ash::vk::Rect2D {
                    offset: ash::vk::Offset2D {
                        x: 0,
                        y: 0,
                    },
                    extent: self.swapchain.extent,
                })
                .clear_values(&clear_values);
            self.device.raw.cmd_begin_render_pass(command_buffer, &render_pass_begin_info, ash::vk::SubpassContents::INLINE);
            self.device.raw.cmd_bind_pipeline(command_buffer, ash::vk::PipelineBindPoint::GRAPHICS, self.particles_pipeline.pipeline);
            let vertex_buffers = [self.vertex_buffer]; let offsets = [0];
            self.device.raw.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &offsets);
            self.device.raw.cmd_bind_descriptor_sets(command_buffer, ash::vk::PipelineBindPoint::GRAPHICS, self.particles_pipeline.pipeline_layout, 0, &[descriptor_set], &[]);
            self.device.raw.cmd_draw(command_buffer, num_entities as u32, 1, 0, 0);
            self.device.raw.cmd_end_render_pass(command_buffer);

            self.device.raw.end_command_buffer(command_buffer).unwrap();

            let command_buffers = [command_buffer];
            let submit_info = ash::vk::SubmitInfo::builder()
                .command_buffers(&command_buffers)
                .build();
            let submit_infos = [submit_info];
            let graphic_queue = self.device.raw.get_device_queue(self.device.graphic_queue.family_index, 0);

            let fence = self.device.raw.create_fence(&ash::vk::FenceCreateInfo::default(), None).unwrap();
            let fences = [fence];
            self.device.raw.queue_submit(graphic_queue, &submit_infos, fence).unwrap();
            self.device.raw.wait_for_fences(&fences, true, u64::MAX).unwrap();
            self.device.raw.destroy_fence(fence, None);
        }
    }

    fn draw_gravity(&mut self, framebuffer: ash::vk::Framebuffer, descriptor_set: ash::vk::DescriptorSet, world: &crate::World) {
        if world.entities_gravity.len() == 0 { return }
        let data: Vec<(crate::entity::Entity, f32)> = world.entities_gravity
            .iter()
            .map(|g| {
                let entity = g.inner.clone();
                let gravity_force = match g.gravity.compute {
                    crate::entity::GravityCompute::Linear(f) => f,
                    crate::entity::GravityCompute::Square(f) => f
                };
                (entity, gravity_force)
            })
            .collect();

        let data_u8_ptr = unsafe { std::mem::transmute::<*const (crate::entity::Entity, f32), *const u8>(data.as_ptr()) };
        let data_u8 = unsafe { std::slice::from_raw_parts(data_u8_ptr, data.len() * std::mem::size_of::<(crate::entity::Entity, f32)>()) };
        self.update_vertex_buffer(data_u8);

        unsafe {
            let commander_buffer_info = ash::vk::CommandBufferAllocateInfo::builder()
                .command_pool(self.command_pool)
                .level(ash::vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(1);
            let command_buffer = self.device.raw.allocate_command_buffers(&commander_buffer_info).unwrap()[0];
            let command_buffer_begin_info = ash::vk::CommandBufferBeginInfo::default();
            self.device.raw.begin_command_buffer(command_buffer, &command_buffer_begin_info).unwrap();
            let clear_values = [ash::vk::ClearValue {
                color: ash::vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 0.0],
                }
            }];
            let render_pass_begin_info = ash::vk::RenderPassBeginInfo::builder()
                .render_pass(self.gravity_pipeline.render_pass)
                .framebuffer(framebuffer)
                .render_area(ash::vk::Rect2D {
                    offset: ash::vk::Offset2D {
                        x: 0,
                        y: 0,
                    },
                    extent: self.swapchain.extent,
                })
                .clear_values(&clear_values);
            self.device.raw.cmd_begin_render_pass(command_buffer, &render_pass_begin_info, ash::vk::SubpassContents::INLINE);
            self.device.raw.cmd_bind_pipeline(command_buffer, ash::vk::PipelineBindPoint::GRAPHICS, self.gravity_pipeline.pipeline);
            let vertex_buffers = [self.vertex_buffer]; let offsets = [0];
            self.device.raw.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &offsets);
            self.device.raw.cmd_bind_descriptor_sets(command_buffer, ash::vk::PipelineBindPoint::GRAPHICS, self.gravity_pipeline.pipeline_layout, 0, &[descriptor_set], &[]);
            self.device.raw.cmd_draw(command_buffer, data.len() as u32, 1, 0, 0);
            self.device.raw.cmd_end_render_pass(command_buffer);

            self.device.raw.end_command_buffer(command_buffer).unwrap();

            let command_buffers = [command_buffer];
            let submit_info = ash::vk::SubmitInfo::builder()
                .command_buffers(&command_buffers)
                .build();
            let submit_infos = [submit_info];
            let graphic_queue = self.device.raw.get_device_queue(self.device.graphic_queue.family_index, 0);

            let fence = self.device.raw.create_fence(&ash::vk::FenceCreateInfo::default(), None).unwrap();
            let fences = [fence];
            self.device.raw.queue_submit(graphic_queue, &submit_infos, fence).unwrap();
            self.device.raw.wait_for_fences(&fences, true, u64::MAX).unwrap();
            self.device.raw.destroy_fence(fence, None);
        }
    }

    pub fn draw_gui(&mut self, draw_data: &imgui::DrawData) {
        let commander_buffer_info = ash::vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.command_pool)
            .level(ash::vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        let command_buffer = unsafe { self.device.raw.allocate_command_buffers(&commander_buffer_info).unwrap()[0] };
        let command_buffer_begin_info = ash::vk::CommandBufferBeginInfo::default();
        unsafe { self.device.raw.begin_command_buffer(command_buffer, &command_buffer_begin_info).unwrap() };
        
        let image_view_framebuffer = [self.swapchain.images_view[self.actual_image_index as usize]];
        let framebuffer_create_info = ash::vk::FramebufferCreateInfo::builder()
            .render_pass(self.imgui_renderer.render_pass)
            .attachments(&image_view_framebuffer)
            .width(self.swapchain.extent.width)
            .height(self.swapchain.extent.height)
            .layers(1);
        let framebuffer = unsafe { self.device.raw.create_framebuffer(&framebuffer_create_info, None).unwrap() };
        let clear_values = [ash::vk::ClearValue {
            color: ash::vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            }
        }];
        let render_pass_begin_info = ash::vk::RenderPassBeginInfo::builder()
            .render_pass(self.imgui_renderer.render_pass)
            .framebuffer(framebuffer)
            .render_area(ash::vk::Rect2D {
                offset: ash::vk::Offset2D {
                    x: 0,
                    y: 0,
                },
                extent: self.swapchain.extent,
            })
            .clear_values(&clear_values);
        
        unsafe { self.device.raw.cmd_begin_render_pass(command_buffer, &render_pass_begin_info, ash::vk::SubpassContents::INLINE); }

        self.imgui_renderer.renderer.as_mut().unwrap().cmd_draw(command_buffer, draw_data).unwrap();

        unsafe { self.device.raw.cmd_end_render_pass(command_buffer); }
        unsafe { self.device.raw.end_command_buffer(command_buffer).unwrap(); }

        let command_buffers = [command_buffer];
        let submit_info = ash::vk::SubmitInfo::builder()
            .command_buffers(&command_buffers)
            .build();
        let submit_infos = [submit_info];
        let graphic_queue = unsafe { self.device.raw.get_device_queue(self.device.graphic_queue.family_index, 0) };

        let fence = unsafe { self.device.raw.create_fence(&ash::vk::FenceCreateInfo::default(), None).unwrap() };
        let fences = [fence];
        
        unsafe {
            self.device.raw.queue_submit(graphic_queue, &submit_infos, fence).unwrap();
            self.device.raw.wait_for_fences(&fences, true, u64::MAX).unwrap();
            self.device.raw.destroy_fence(fence, None);
            self.device.raw.destroy_framebuffer(framebuffer, None);
        }
    }

    pub fn present(&mut self) {
        let swapchains = [self.swapchain.raw];
        let image_indices = [self.actual_image_index];
        let present_info = ash::vk::PresentInfoKHR::builder()
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        let presentation_queue = unsafe { self.device.raw.get_device_queue(self.device.presentation_queue.family_index, 0) };
        let _ = unsafe { self.swapchain.util.queue_present(presentation_queue, &present_info) };

        unsafe { self.device.raw.destroy_command_pool(self.command_pool, None); }
        let command_pool_info = ash::vk::CommandPoolCreateInfo::builder()
            .queue_family_index(self.device.graphic_queue.family_index);
        self.command_pool = unsafe { self.device.raw.create_command_pool(&command_pool_info, None).unwrap() };
    }

    pub fn resize(&mut self, imgui: &mut imgui::Context, size: winit::dpi::PhysicalSize<u32>) {
        unsafe { self.device.raw.device_wait_idle().unwrap(); }
        unsafe { self.imgui_renderer.destroy(&&self.device); }
        
        unsafe {
            self.gravity_pipeline.destroy(&self.device);
            self.particles_pipeline.destroy(&self.device);
            self.swapchain.destroy_image_views(&self.device);
            self.swapchain.destroy();
        }

        self.recreate_swapchain(size);
        self.imgui_renderer = ImguiRenderer::new(imgui, &self.physical_device, &self.instance, &self.device, &self.swapchain, self.command_pool);
    }

    fn recreate_swapchain(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        unsafe {
            let mut swapchain = self.instance.create_swapchain(&self.physical_device, &self.device, &self.surface, ash::vk::PresentModeKHR::MAILBOX, size.into());
            swapchain.update_images_views(&self.device);
            self.swapchain = swapchain;
            
            self.particles_pipeline = ParticlesPipeline::create(&self.device, &self.swapchain, size);
            self.gravity_pipeline = GravityPipeline::create(&self.device, &self.swapchain, size);
        }
    }

    pub fn destroy(&mut self) {

        unsafe { self.imgui_renderer.destroy(&self.device); }

        self.gravity_pipeline.destroy(&self.device);
        self.particles_pipeline.destroy(&self.device);

        unsafe {
            self.device.raw.destroy_buffer(self.staging_buffer, None);
            self.device.raw.destroy_buffer(self.vertex_buffer, None);
            self.device.raw.destroy_buffer(self.uniform_buffer, None);
            self.device.raw.free_memory(self.staging_buffer_memory, None);
            self.device.raw.free_memory(self.vertex_buffer_memory, None);
            self.device.raw.free_memory(self.uniform_buffer_memory, None);
            self.device.raw.destroy_command_pool(self.command_pool, None);
            self.swapchain.destroy_image_views(&self.device);
            self.swapchain.destroy();
            self.device.destroy();
            self.surface.destroy();
            self.instance.destroy();
        }
    }
}