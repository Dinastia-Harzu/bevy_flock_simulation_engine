use crate::boid_simulation::resources::*;
use bevy::prelude::*;
use std::{
    fmt::Debug,
    ops::{AddAssign, DivAssign, Range, RangeInclusive},
};

#[derive(Reflect)]
pub struct SpatialGridBoid {
    pub entity: Entity,
    pub position: Vec2,
    pub velocity: Vec2,
}

impl SpatialGridBoid {
    pub fn new(entity: Entity, position: Vec2, velocity: Vec2) -> Self {
        Self {
            entity,
            position,
            velocity,
        }
    }
}

pub(crate) type CellBoids = Vec<SpatialGridBoid>;
pub(crate) type Cells = Vec<SpatialGridCell>;

#[derive(Reflect)]
pub struct SpatialGridCell {
    pub(crate) grid_pos: UVec2,
    pub(crate) rect: Rect,
    pub(crate) boids: CellBoids,
}

impl SpatialGridCell {
    pub fn new(row: u32, column: u32, size: f32, centre: Vec2) -> Self {
        Self {
            grid_pos: (row, column).into(),
            rect: Rect::from_center_size(centre, Vec2::new(size, size)),
            boids: Vec::with_capacity(SimulationConfiguration::max_boids() as usize),
        }
    }

    pub fn push(&mut self, boid: SpatialGridBoid) {
        self.boids.push(boid);
    }

    pub fn size(&self) -> f32 {
        self.rect.size().x
    }

    pub fn location(&self) -> Vec2 {
        self.rect.center()
    }

    pub fn cell_boids(&self) -> &CellBoids {
        &self.boids
    }

    pub fn contains(&self, location: Vec2) -> bool {
        self.rect.contains(location)
    }
}

impl Debug for SpatialGridCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} boids", self.boids.len())
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct OVec2(Option<Vec2>);

impl OVec2 {
    pub fn new(v: Vec2) -> Self {
        Self(Some(v))
    }

    pub fn get(&self) -> Option<Vec2> {
        self.0
    }
}

impl AddAssign<Vec2> for OVec2 {
    fn add_assign(&mut self, rhs: Vec2) {
        self.0 = Some(self.0.unwrap_or_default() + rhs);
    }
}

impl DivAssign<f32> for OVec2 {
    fn div_assign(&mut self, rhs: f32) {
        if self.0.is_some() {
            self.0 = Some(self.0.unwrap_or_default() / rhs);
        }
    }
}

pub trait ToroidalClamp {
    fn toroidal_clamp(&mut self, inf: Self, sup: Self);
    fn toroidal_clamp_exclusive(&mut self, inf: Self, sup: Self);
    fn toroidal_clamp_range(&mut self, range: RangeInclusive<Self>)
    where
        Self: Sized;
    fn toroidal_clamp_exclusive_range(&mut self, range: Range<Self>)
    where
        Self: Sized;
}

impl<T: PartialOrd + Sized + Clone + Copy> ToroidalClamp for T {
    fn toroidal_clamp(&mut self, inf: Self, sup: Self) {
        if *self >= sup {
            *self = inf;
        } else if *self <= inf {
            *self = sup;
        }
    }

    fn toroidal_clamp_exclusive(&mut self, inf: Self, sup: Self) {
        if *self > sup {
            *self = inf;
        } else if *self < inf {
            *self = sup;
        }
    }

    fn toroidal_clamp_range(&mut self, range: RangeInclusive<Self>) {
        let sup = *range.start();
        let inf = *range.end();
        if *self >= sup {
            *self = inf;
        } else if *self <= inf {
            *self = sup;
        }
    }

    fn toroidal_clamp_exclusive_range(&mut self, range: Range<Self>) {
        let sup = range.start;
        let inf = range.end;
        if *self > sup {
            *self = inf;
        } else if *self < inf {
            *self = sup;
        }
    }
}
