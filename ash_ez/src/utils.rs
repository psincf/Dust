pub fn pipeline_shader_stage_create_info_helper(module: ash::vk::ShaderModule, stage: ash::vk::ShaderStageFlags, name: &std::ffi::CStr) -> ash::vk::PipelineShaderStageCreateInfo {
    ash::vk::PipelineShaderStageCreateInfo::builder()
        .module(module)
        .stage(stage)
        .name(name)
        .build()
}

pub fn viewport_helper(width: f32, height: f32) -> ash::vk::Viewport {
    ash::vk::Viewport {
        x: 0.0,
        y: 0.0,
        width,
        height,
        min_depth: 0.0,
        max_depth: 1.0,
    }
}

pub fn scissor_helper(width: u32, height: u32) -> ash::vk::Rect2D {
    ash::vk::Rect2D {
        offset: ash::vk::Offset2D::default(),
        extent: ash::vk::Extent2D {
            width,
            height,
        }
    }
}

pub fn pipeline_viewport_state_create_info_helper(width: u32, height: u32) -> ash::vk::PipelineViewportStateCreateInfo {
    let viewport = [viewport_helper(width as f32, height as f32)];
    let scissor = [scissor_helper(width, height)];
    ash::vk::PipelineViewportStateCreateInfo::builder()
        .viewports(&viewport)
        .scissors(&scissor)
        .build()
}

pub fn pipeline_viewport_state_create_info_helper_2(viewport: &[ash::vk::Viewport], scissor: &[ash::vk::Rect2D]) -> ash::vk::PipelineViewportStateCreateInfo {
    ash::vk::PipelineViewportStateCreateInfo::builder()
        .viewports(viewport)
        .scissors(scissor)
        .build()
}

pub fn pipeline_rasterization_state_create_info_helper() -> ash::vk::PipelineRasterizationStateCreateInfo {
    ash::vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(ash::vk::PolygonMode::FILL)
        .cull_mode(ash::vk::CullModeFlags::NONE)
        .front_face(ash::vk::FrontFace::CLOCKWISE)
        .depth_clamp_enable(false)
        .line_width(1.0)
        .build()
}

pub fn pipeline_multisample_state_create_info_helper() -> ash::vk::PipelineMultisampleStateCreateInfo {
    ash::vk::PipelineMultisampleStateCreateInfo::builder()
        .rasterization_samples(ash::vk::SampleCountFlags::TYPE_1)
        .sample_shading_enable(false)
        .min_sample_shading(0.0)
        //.sample_mask(&[])
        .alpha_to_coverage_enable(false)
        .alpha_to_one_enable(false)
        .build()
}

pub fn pipeline_color_blend_attachment_state_helper() -> ash::vk::PipelineColorBlendAttachmentState {
    ash::vk::PipelineColorBlendAttachmentState::builder()
        .blend_enable(true)
        .src_color_blend_factor(ash::vk::BlendFactor::SRC_ALPHA)
        .dst_color_blend_factor(ash::vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .color_blend_op(ash::vk::BlendOp::ADD)
        .src_alpha_blend_factor(ash::vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(ash::vk::BlendFactor::ZERO)
        .alpha_blend_op(ash::vk::BlendOp::ADD)
        .color_write_mask(ash::vk::ColorComponentFlags::RGBA)
        .build()
}


/*
pub fn pipeline_color_blend_state_create_info_helper(attachment: ash::vk::PipelineColorBlendAttachmentState) -> ash::vk::PipelineColorBlendStateCreateInfo{
    let attachment = [attachment];
    ash::vk::PipelineColorBlendStateCreateInfo::builder()
        .attachments(&attachment)
        .build()
}
*/

pub fn pipeline_dynamic_state_create_info_helper() -> ash::vk::PipelineDynamicStateCreateInfo {
    ash::vk::PipelineDynamicStateCreateInfo::builder()
        .build()
}

pub fn attachment_description_simple_helper(swapchain: &crate::Swapchain) -> ash::vk::AttachmentDescription {
    ash::vk::AttachmentDescription::builder()
        .format(swapchain.format)
        .samples(ash::vk::SampleCountFlags::TYPE_1)
        .load_op(ash::vk::AttachmentLoadOp::LOAD)
        .store_op(ash::vk::AttachmentStoreOp::STORE)
        .stencil_load_op(ash::vk::AttachmentLoadOp::LOAD)
        .stencil_store_op(ash::vk::AttachmentStoreOp::STORE)
        .initial_layout(ash::vk::ImageLayout::UNDEFINED)
        .final_layout(ash::vk::ImageLayout::PRESENT_SRC_KHR)
        .build()
}