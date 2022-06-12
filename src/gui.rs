use crate::{PRECISION, SIZE_X, SIZE_Y};
use crate::entity::{GravityCompute, GravityEntity};
use crate::world::{Settings, World};

use euclid::default::{Point2D, Vector2D};

pub struct GUI {
    pub imgui: imgui::Context,
    pub imgui_winit_platform: imgui_winit_support::WinitPlatform,
    mouse_on_gui: bool
}

impl GUI {
    pub fn mouse_on_gui(&self) -> bool {
        self.mouse_on_gui
    }
    
    pub fn init(window: &winit::window::Window) -> GUI {
        let mut imgui = imgui::Context::create();
        let mut imgui_winit_platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
        let imgui_io = imgui.io_mut();
        imgui_winit_platform.attach_window(imgui_io, window, imgui_winit_support::HiDpiMode::Default);

        GUI {
            imgui: imgui,
            imgui_winit_platform: imgui_winit_platform,
            mouse_on_gui: false,
        }
    }

    pub fn update(&mut self, window: &winit::window::Window, world: &mut World) -> &imgui::DrawData {
        let window_size = window.inner_size();
        let io = self.imgui.io_mut();
        self.imgui_winit_platform.prepare_frame(io, window).unwrap();

        let ui = self.imgui.frame();
        let _imgui_window = imgui::Window::new("Particles")
            .movable(false)
            .resizable(false)
            .position([0.0, 0.0], imgui::Condition::Always)
            .size([400.0 as f32, window_size.height as f32], imgui::Condition::Always)
            .scrollable(true)
            .bg_alpha(0.5)
            .build(&ui, || {
                let mut force_mouse = world.settings.mouse_gravity.gravity.compute.get_force();
                let mouse_gravity_mode_str = match world.settings.mouse_gravity.gravity.compute { GravityCompute::Linear(_) => { "linear" }, GravityCompute::Square(_) => { "squared" } };
                imgui::ComboBox::new("mouse_gravity_mode").preview_value(mouse_gravity_mode_str).build(&ui, || {
                    if imgui::Selectable::new("linear").build(&ui) { world.settings.mouse_gravity.gravity.compute = GravityCompute::Linear(force_mouse); }
                    if imgui::Selectable::new("squared").build(&ui) { world.settings.mouse_gravity.gravity.compute = GravityCompute::Square(force_mouse); }
                });
                imgui::Slider::new("force_mouse", -1_000.0, 1_000.0).flags(imgui::SliderFlags::ALWAYS_CLAMP).build(&ui, &mut force_mouse);
                world.settings.mouse_gravity.gravity.compute.set_force(force_mouse);

                ui.separator();

                imgui::Slider::new("max_speed", 1.0, 10_000.0).flags(imgui::SliderFlags::LOGARITHMIC | imgui::SliderFlags::ALWAYS_CLAMP).build(&ui, &mut world.settings.max_speed);
                imgui::Slider::new("max_gravity_speed", 1.0, 10_000.0).flags(imgui::SliderFlags::LOGARITHMIC | imgui::SliderFlags::ALWAYS_CLAMP).build(&ui, &mut world.settings.max_gravity_speed);
                imgui::Slider::new("energy_loss", 0.0, 10.0).flags(imgui::SliderFlags::LOGARITHMIC | imgui::SliderFlags::ALWAYS_CLAMP).build(&ui, &mut world.settings.energy_loss);
                imgui::Slider::new("gravity_power", 0.0, 1_000.0).flags(imgui::SliderFlags::LOGARITHMIC | imgui::SliderFlags::ALWAYS_CLAMP).build(&ui, &mut world.settings.gravity_power);
                ui.checkbox("block", &mut world.settings.block);
                imgui::Slider::new("time_factor", 0.1, 10.0).flags(imgui::SliderFlags::LOGARITHMIC | imgui::SliderFlags::ALWAYS_CLAMP).build(&ui, &mut world.settings.time_factor);

                let mut color_base = [world.settings.color_base.0, world.settings.color_base.1, world.settings.color_base.2];
                let mut color_fast = [world.settings.color_fast.0, world.settings.color_fast.1, world.settings.color_fast.2];
                imgui::ColorEdit::new("color_base", &mut color_base).build(&ui);
                imgui::ColorEdit::new("color_fast", &mut color_fast).build(&ui);
                world.settings.color_base = (color_base[0], color_base[1], color_base[2]);
                world.settings.color_fast = (color_fast[0], color_fast[1], color_fast[2]);
                imgui::Slider::new("color_ratio_speed", 0.1, 10.0).flags(imgui::SliderFlags::LOGARITHMIC | imgui::SliderFlags::ALWAYS_CLAMP).build(&ui, &mut world.settings.color_ratio_speed);

                imgui::Slider::new("alpha", 0.01, 1.0).flags(imgui::SliderFlags::LOGARITHMIC | imgui::SliderFlags::ALWAYS_CLAMP).build(&ui, &mut world.settings.alpha);

                ui.separator();
                
                ui.text("num_particles = ".to_string() + &world.num_particles.to_string()); ui.same_line();
                if ui.button("x2") { world.num_particles *= 2; world.apply_particles_number(); } ui.same_line();
                if ui.button("/2") { world.num_particles /= 2; world.num_particles = world.num_particles.max(1); world.apply_particles_number(); }

                ui.separator();

                if ui.button("reset_entities") { world.reset(); } ui.same_line();
                if ui.button("reset_settings") { world.settings = Settings::default(); }

                ui.separator();

                if ui.button("stop_speed") { world.stop_speed(); } ui.same_line();
                if ui.button("tilt") { world.tilt(); }

                ui.separator();

                if ui.button("new gravity_entity") {
                    let position = Point2D::new(
                        fastrand::i32((0)..(SIZE_X * PRECISION)),
                        fastrand::i32((0)..(SIZE_Y * PRECISION)),
                    );
                    let mut new_entity = GravityEntity::default();
                    new_entity.inner.position = position;
                    new_entity.gravity.compute = GravityCompute::Linear(10.0);
                    new_entity.movable = true;
                    world.entities_gravity.push(new_entity);
                }

                if ui.collapsing_header("gravity_entities", imgui::TreeNodeFlags::DEFAULT_OPEN) {
                    let mut to_delete = Vec::new();
                    for (index, gravity_entity) in world.entities_gravity.iter_mut().enumerate() {
                        ui.indent();
                        imgui::TreeNode::new(index.to_string()).flags(imgui::TreeNodeFlags::DEFAULT_OPEN).build(&ui, || {
                            let id_str = index.to_string();
                            ui.indent();
                            let mut force = gravity_entity.gravity.compute.get_force();
                            let gravity_mode_str = match gravity_entity.gravity.compute { GravityCompute::Linear(_) => { "linear".to_string() }, GravityCompute::Square(_) => { "squared".to_string() } };
                            imgui::ComboBox::new("mouse_gravity_mode").preview_value(gravity_mode_str + "##" + &id_str).build(&ui, || {
                                if imgui::Selectable::new("linear##".to_string() + &id_str).build(&ui) { gravity_entity.gravity.compute = GravityCompute::Linear(force_mouse); }
                                if imgui::Selectable::new("squared##".to_string() + &id_str).build(&ui) { gravity_entity.gravity.compute = GravityCompute::Square(force_mouse); }
                            });
                            imgui::Slider::new("gravity_force##".to_string() + &id_str, -100.0, 100.0).build(&ui, &mut force);
                            gravity_entity.gravity.compute.set_force(force);

                            ui.checkbox("movable##".to_string() + &id_str, &mut gravity_entity.movable);
                            if !gravity_entity.movable { gravity_entity.inner.speed = Vector2D::zero(); }
                            if ui.button("delete##".to_string() + &id_str) { to_delete.push(index); }
                            ui.unindent();
                        });
                        ui.unindent();
                    }
                    to_delete.sort();
                    to_delete.iter().rev().for_each(|i| { world.entities_gravity.remove(*i); });
                }

                ui.separator();
                if ui.collapsing_header("benchmark", imgui::TreeNodeFlags::DEFAULT_OPEN) {
                    ui.text("update_time = ".to_string() + &(world.benchmark_update * 1_000.0).to_string() + "ms");
                    ui.text("draw_time   = ".to_string() + &(world.benchmark_draw * 1_000.0).to_string() + "ms");
                }
                ui.separator();

                ui.text("num_threads = ".to_string() + &world.threadpool.num_threads().to_string()); ui.same_line();
                if ui.button("+1") { world.threadpool.new_thread(); } ui.same_line();
                if ui.button("-1") { if world.threadpool.num_threads() > 1 { world.threadpool.delete_thread(); } }
            }
        );

        if ui.is_any_item_hovered() || ui.is_window_hovered_with_flags(imgui::WindowHoveredFlags::ANY_WINDOW) { self.mouse_on_gui = true; }
        else { self.mouse_on_gui = false; }

        // Draw
        self.imgui_winit_platform.prepare_render(&ui, window);

        return ui.render();
        
    }
}