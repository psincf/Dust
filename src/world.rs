use crate::entity::Entity;
use crate::entity::GravityEntity;
use crate::entity::GravityCompute;
use crate::entity::GravitySettings;
use crate::PRECISION;
use crate::renderer;
use crate::threadpool;

use euclid::default::{Point2D, Vector2D};

#[derive(Clone)]
pub struct Settings {
    pub time_factor: f32,
    pub mouse_gravity: GravityEntity,
    pub mouse_gravity_active: bool,
    pub gravity_power: f32,
    pub energy_loss: f32,
    pub max_speed: f32,
    pub max_gravity_speed: f32,
    pub block: bool,
    pub color_base: (f32, f32, f32),
    pub color_fast: (f32, f32, f32),
    pub color_ratio_speed: f32,
    pub alpha: f32,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            time_factor: 1.0,
            mouse_gravity: GravityEntity {
                gravity: GravitySettings {
                    compute: GravityCompute::Linear(100.0)
                },
                ..Default::default()
            },
            mouse_gravity_active: false,
            gravity_power: 100.0,
            energy_loss: 0.1,
            max_speed: 10_000.0,
            max_gravity_speed: 10_000.0,
            block: true,
            color_base: (1.0, 1.0, 1.0),
            color_fast: (1.0, 0.0, 0.0),
            color_ratio_speed: 1.0,
            alpha: 0.2,
        }
    }
}

pub struct World {
    pub position_camera: Point2D<i32>,
    pub size_field: (i32, i32),
    pub zoom: f32,
    pub debug: bool,
    pub last_update: std::time::Instant,
    pub elapsed_since_update: f32,
    pub entities: Vec<Entity>,
    pub entities_gravity: Vec<GravityEntity>,
    pub settings: Settings,
    pub num_particles: i32,
    pub benchmark_update: f32,
    pub benchmark_draw: f32,
    pub threadpool: threadpool::Threadpool,
}

impl World {
    pub fn change_debug(&mut self) {
        self.debug = !self.debug;
    }

    pub fn num_entities(&self) -> usize {
        self.entities.len() + self.entities_gravity.len()
    }

    pub fn update_mouse_gravity_info(&mut self, mouse_position: winit::dpi::PhysicalPosition<f64>, window_size: winit::dpi::PhysicalSize<u32>) {
        let relative_position = winit::dpi::PhysicalPosition::new(
            2.0 * (mouse_position.x - window_size.width as f64 / 2.0),
            2.0 * (mouse_position.y - window_size.height as f64 / 2.0)
        );
        self.settings.mouse_gravity.inner = Entity {
            position: Point2D::new(
                (relative_position.x * self.zoom as f64) as i32 + self.position_camera.x,
                (-relative_position.y * self.zoom as f64) as i32 + self.position_camera.y),
            speed: Vector2D::zero()
        };
        self.settings.mouse_gravity.inner.position = self.settings.mouse_gravity.inner.position.clamp(
            Point2D::zero(),
            Point2D::new(self.size_field.0, self.size_field.1)
        );
    }

    pub fn apply_particles_number(&mut self) {
        while self.num_particles > self.entities.len() as i32 {
            self.entities.push(Entity::new_random(self.size_field));
        }

        while self.num_particles < self.entities.len() as i32 {
            self.entities.pop().unwrap();
        }
    }

    pub fn reset(&mut self) {
        self.entities.clear();
        self.entities_gravity.clear();

        for _ in 0..self.num_particles {
            self.entities.push(Entity::new_random(self.size_field));
        }
    }

    pub fn resize(&mut self, new_size: (i32, i32)) {
        let old_size_field = self.size_field;
        let new_size_field = (new_size.0 * PRECISION, new_size.1 * PRECISION);

        self.size_field = new_size_field;
        self.position_camera = Point2D::new(new_size.0 * PRECISION / 2, new_size.1 * PRECISION / 2);

        for entity in self.entities.iter_mut() {
            entity.update_position_new_size(old_size_field, new_size_field);
        }

        for entity in self.entities_gravity.iter_mut() {
            entity.inner.update_position_new_size(old_size_field, new_size_field);
        }
    }

    pub fn tilt(&mut self) {
        for entity in self.entities.iter_mut() {
            entity.position.x += fastrand::i32(-PRECISION..PRECISION);
            entity.position.y += fastrand::i32(-PRECISION..PRECISION);
        }
        for entity in self.entities_gravity.iter_mut() {
            entity.inner.position.x += fastrand::i32(-PRECISION..PRECISION);
            entity.inner.position.y += fastrand::i32(-PRECISION..PRECISION);
        }
    }

    pub fn stop_speed(&mut self) {
        for entity in self.entities.iter_mut() {
            entity.speed = Vector2D::zero();
        }
        for entity in self.entities_gravity.iter_mut() {
            entity.inner.speed = Vector2D::zero();
        }
    }

    pub fn update(&mut self) {
        self.apply_particles_number();
        self.update_cpu();
    }

    pub fn update_cpu(&mut self) {
        let time = std::time::Instant::now();
        self.elapsed_since_update = time.duration_since(self.last_update).as_secs_f32().min(0.02) * self.settings.time_factor;

        self.last_update = time;
        
        let mut entities_gravity_cache: Vec<(usize, GravityEntity)> = self.entities_gravity.clone().into_iter().enumerate().collect();
        if self.settings.mouse_gravity_active { entities_gravity_cache.push((usize::MAX, self.settings.mouse_gravity.clone())) }
        
        self.update_cpu_multithread(&entities_gravity_cache);

        for (index_entity, entity) in self.entities_gravity.iter_mut().enumerate() {
            if !entity.movable { continue }
            for (index_gravity, gravity) in entities_gravity_cache.iter() {
                if index_entity == *index_gravity { continue }
                entity.inner.apply_gravity(gravity, &self.settings, self.elapsed_since_update, self.size_field);
            }
            entity.inner.update_position(&self.settings, self.elapsed_since_update, self.size_field);
        }

        self.benchmark_update = time.elapsed().as_secs_f32();
    }

    pub fn update_cpu_singlethread(&mut self, entities_gravity_cache: &Vec<(usize, GravityEntity)>) {
        for entity in self.entities.iter_mut() {
            for (_index, gravity) in entities_gravity_cache.iter() {
                entity.apply_gravity(gravity, &self.settings, self.elapsed_since_update, self.size_field);
            }
            entity.update_position(&self.settings, self.elapsed_since_update, self.size_field);
        }
    }

    pub fn update_cpu_multithread(&mut self, entities_gravity_cache: &Vec<(usize, GravityEntity)>) {
        let len = self.entities.len();
        let num_threads = self.threadpool.num_threads();
        let size_chunk = len / num_threads + 1;

        for thread in 0..num_threads {
            let entities = std::sync::atomic::AtomicPtr::new(&mut self.entities);
            let world = std::sync::atomic::AtomicPtr::new(self);
            let range = (thread * size_chunk)..((thread + 1) * size_chunk);
            let (range_begin, range_end) = (range.start, range.end);

            unsafe { self.threadpool.send_work_unsafe(move || {
                let entities = entities.load(std::sync::atomic::Ordering::Relaxed);
                let entities = &mut *entities;
                let world = world.load(std::sync::atomic::Ordering::Relaxed);
                let world = &mut *world;
                for i in range_begin..range_end {
                    if let Some(entity) = entities.get_mut(i) {
                        for (_index, gravity) in entities_gravity_cache.iter() {
                            entity.apply_gravity(gravity, &world.settings, world.elapsed_since_update, world.size_field);
                        }
                        entity.update_position(&world.settings, world.elapsed_since_update, world.size_field);
                    }
                }
            })};
        }
        self.threadpool.wait();
    }

    pub fn draw(&mut self, renderer: &mut renderer::Renderer) {
        let time = std::time::Instant::now();
        renderer.draw(self);

        self.benchmark_draw = time.elapsed().as_secs_f32();
    }
}