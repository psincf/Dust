use crate::PRECISION;
use crate::world::Settings;

use euclid::default::{Point2D, Vector2D};

#[derive(Clone, Default, Debug)]
pub struct Entity {
    pub position: Point2D<i32>,
    pub speed: Vector2D<f32>,
}

impl Entity {
    pub fn new_random(size_field: (i32, i32)) -> Entity {
        Entity {
            position: Point2D::new(
                fastrand::i32((0)..(size_field.0)),
                fastrand::i32((0)..(size_field.1)),
            ),
            speed: Vector2D::zero(),
        }
    }
    pub fn apply_gravity(&mut self, gravity: &GravityEntity, settings: &Settings, elapsed: f32, size_field: (i32, i32)) {
        let mut direction = {
            if self.position == gravity.inner.position {
                Vector2D::new(
                    fastrand::i32(1..100) * (fastrand::bool() as i32 * 2 - 1),
                    fastrand::i32(1..100) * (fastrand::bool() as i32 * 2 - 1)
                )
            } else {
                Vector2D::new(
                    gravity.inner.position.x - self.position.x,
                    gravity.inner.position.y - self.position.y
                )
            }
        };

        if !settings.block {
            if direction.x.abs() > size_field.0 / 2 { direction.x = -direction.x.signum() * (size_field.0 / 2 - direction.x.abs() % (size_field.0 / 2)); }
            if direction.y.abs() > size_field.1 / 2 { direction.y = -direction.y.signum() * (size_field.1 / 2 - direction.y.abs() % (size_field.1 / 2)); }
        }
        
        let direction_f32 = direction.to_f32();
        let distance_squared = direction_f32.square_length();
        let distance = direction_f32.length();

        let new_speed_total = (PRECISION as f32).powi(2) * settings.gravity_power * match gravity.gravity.compute {
            GravityCompute::Linear(p) => { distance.recip() * p * 10.0 },
            GravityCompute::Square(p) => { distance_squared.recip() * p * 1_000_000.0 }
        };
        let new_speed_total = new_speed_total.clamp(-settings.max_gravity_speed * PRECISION as f32, settings.max_gravity_speed * PRECISION as f32);

        let new_speed = direction_f32.normalize() * new_speed_total;

        self.speed += new_speed * elapsed;
    }

    pub fn update_position_new_size(&mut self, old_size: (i32, i32), new_size: (i32, i32)) {
        let ratio_position = (self.position.x as f64 / old_size.0 as f64, self.position.y as f64 / old_size.1 as f64);

        self.position.x = (ratio_position.0 * new_size.0 as f64) as i32;
        self.position.y = (ratio_position.1 * new_size.1 as f64) as i32;
    }

    pub fn update_position(&mut self, settings: &Settings, elapsed: f32, size_field: (i32, i32)) {
        self.speed = self.speed.with_max_length(PRECISION as f32 * settings.max_speed);

        self.speed *= (1.0 - elapsed * settings.energy_loss).max(0.1);

        self.position += (self.speed * elapsed).round().to_i32();
        
        if settings.block {
            if self.position.x <= 0 {
                self.speed.x = self.speed.x.abs() * 1.0;
                self.position.x = 0;
            }
            if self.position.x >= size_field.0 {
                self.speed.x = self.speed.x.abs() * -1.0;
                self.position.x = size_field.0;
            };


            if self.position.y <= 0 {
                self.speed.y = self.speed.y.abs() * 1.0;
                self.position.y = 0;
            }        
            if self.position.y >= size_field.1 {
                self.speed.y = self.speed.y.abs() * -1.0;
                self.position.y = size_field.1;
            };
        } else {
            self.position.x = ((self.position.x % size_field.0) + size_field.0) % size_field.0;
            self.position.y = ((self.position.y % size_field.1) + size_field.1) % size_field.1;
        }
    }
}

#[derive(Clone, Default)]
pub struct GravityEntity {
    pub inner: Entity,
    pub gravity: GravitySettings,
    pub movable: bool,
}

#[derive(Clone)]
pub enum GravityCompute {
    Linear(f32),
    Square(f32)
}

impl GravityCompute {
    pub fn get_force(&self) -> f32 {
        match self {
            GravityCompute::Linear(f) => { *f }
            GravityCompute::Square(f) => { *f }
        }
    }

    pub fn set_force(&mut self, force: f32) {
        match self {
            GravityCompute::Linear(f) => { *f = force; }
            GravityCompute::Square(f) => { *f = force; }
        }
    }
}

impl Default for GravityCompute {
    fn default() -> GravityCompute {
        GravityCompute::Linear(0.0)
    }
}

#[derive(Clone, Default)]
pub struct GravitySettings {
    pub compute: GravityCompute,
}