#![allow(unused_unsafe)]

pub mod entity;
mod gui;
mod threadpool;
pub mod world;

mod renderer_vulkan;
use renderer_vulkan as renderer;

use euclid::default::Point2D;
use world::Settings;
use world::World;

use gui::GUI;

const PRECISION: i32 = 1_000;
const SIZE_X: i32 = 1280;
const SIZE_Y: i32 = 720;

fn main() {
    let event_loop = winit::event_loop::EventLoop::new();

    let window = winit::window::WindowBuilder::new()
        .with_title("Dust")
        .with_inner_size(winit::dpi::PhysicalSize::new(SIZE_X, SIZE_Y))
        .build(&event_loop)
        .unwrap();
    
    let mut gui = GUI::init(&window);
    
    let mut renderer = renderer::Renderer::new(&window, &mut gui.imgui);

    let mut world = Box::new(World {
        position_camera: Point2D::new(SIZE_X * PRECISION / 2, SIZE_Y * PRECISION / 2),
        size_field: (SIZE_X * PRECISION, SIZE_Y * PRECISION),
        zoom: 0.5 * PRECISION as f32,
        debug: false,
        last_update: std::time::Instant::now(),
        elapsed_since_update: 1.0,
        entities: Vec::new(),
        entities_gravity: Vec::new(),
        settings: Settings::default(),
        num_particles: 2i32.pow(18),
        benchmark_update: 0.0,
        benchmark_draw: 0.0,
        threadpool: threadpool::Threadpool::new_with_threads(2),
    });
    world.reset();

    let mut mouse_position = winit::dpi::PhysicalPosition::<f64>::default();
    let mut run = true;
    event_loop.run(move |event, _event_loop_window_target, control_flow,| {
        let window_size = window.inner_size();
        
        let io = gui.imgui.io_mut();
        gui.imgui_winit_platform.handle_event(io, &window, &event);

        match event {
            winit::event::Event::MainEventsCleared => {
                if run == false { return }
                let draw_data = gui.update(&window, &mut world);

                world.update();
                world.draw(&mut renderer);
                renderer.draw_gui(draw_data);

                renderer.present();
            }
            winit::event::Event::WindowEvent{window_id: _, event} => {
                use winit::event::WindowEvent;
                use winit::event::ElementState;
                match event {
                    WindowEvent::CloseRequested => {
                        renderer.destroy();
                        *control_flow = winit::event_loop::ControlFlow::Exit;
                        run = false;
                    }
                    WindowEvent::Resized(size) => {
                        world.resize((size.width as i32, size.height as i32));
                        renderer.resize(&mut gui.imgui, size);
                    }
                    WindowEvent::CursorMoved{device_id:_, position, ..} => {
                        mouse_position = position;
                        world.update_mouse_gravity_info(mouse_position, window_size);
                    }
                    WindowEvent::MouseWheel{delta, ..} => {
                        use winit::event::MouseScrollDelta;
                        match delta {
                            MouseScrollDelta::LineDelta(_x, _y) => { /* world.zoom *= 1.1f32.powf(-y); */ }
                            MouseScrollDelta::PixelDelta(_) => { }
                        }
                    }
                    WindowEvent::MouseInput{device_id: _, state, button, ..} => {
                        use winit::event::MouseButton;
                        match button {
                            MouseButton::Left => {
                                match state {
                                    ElementState::Pressed => {
                                        if !gui.mouse_on_gui() {
                                            world.settings.mouse_gravity_active = true;
                                        }
                                        world.update_mouse_gravity_info(mouse_position, window_size);
                                    }
                                    ElementState::Released => {
                                        world.settings.mouse_gravity_active = false;
                                    }
                                }
                            }
                            _ => {  }
                        }
                    }
                    WindowEvent::KeyboardInput{device_id: _, input, is_synthetic: _} => {
                        use winit::event::VirtualKeyCode;
                        match input.state {
                            ElementState::Pressed => {
                                match input.virtual_keycode {
                                    Some(VirtualKeyCode::D) => { world.change_debug(); }
                                    Some(VirtualKeyCode::R) => { world.reset(); }
                                    Some(VirtualKeyCode::S) => { world.stop_speed(); }
                                    Some(VirtualKeyCode::T) => { world.tilt(); }
                                    _ => {  }
                                }
                            },
                            _ => {  }
                        }
                    }
                    _ => {  }
                }
            }
            _ => {  }
        }
    });
}