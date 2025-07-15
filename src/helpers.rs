use bevy::prelude::*;
use std::{
    fmt::Debug,
    ops::{AddAssign, DivAssign, Index, IndexMut},
};

use crate::boid_simulation::resources::SimulationConfiguration;

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

///////////////////////////////////////////////////////
///////////////////////////////////////////////////////

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

pub fn coulomb(r1: Vec2, r2: Vec2, q1: f32, q2: f32) -> Vec2 {
    let r = r2 - r1;
    let u = r.normalize_or_zero();
    u * q1 * q2 / r.length_squared()
}

///////////////////////////////////////////////////////
///////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy)]
pub struct CoordMapping {
    pub(crate) src: Rect,
    pub(crate) dest: UVec2,
    pub(crate) scale: Vec2,
    pub(crate) offset: Vec2,
}

impl CoordMapping {
    pub fn new(src: Rect, dest: UVec2) -> Self {
        let scale = dest.as_vec2() / src.size();
        Self {
            src,
            dest,
            scale,
            offset: src.min * scale,
        }
    }

    pub fn map_point(&self, point: Vec2) -> Vec2 {
        ((point * self.scale) + self.offset).floor()
    }
}

#[derive(Debug)]
pub struct Grid<T> {
    pub(crate) size: UVec2,
    pub(crate) data: Vec<T>,
}

impl<T: Clone + Copy> Grid<T> {
    pub fn new(size: UVec2, default: T) -> Self {
        Self {
            size,
            data: vec![default; (size.x * size.y) as usize],
        }
    }
}

impl<T> Grid<T> {
    pub fn columns(&self) -> u32 {
        self.size.x
    }

    pub fn rows(&self) -> u32 {
        self.size.y
    }
}

impl<T: Clone + Copy> Index<usize> for Grid<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<T: Clone + Copy> IndexMut<usize> for Grid<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl<T: Clone + Copy> Index<UVec2> for Grid<T> {
    type Output = T;

    fn index(&self, index: UVec2) -> &Self::Output {
        &self.data[(index.y * self.columns() + index.x) as usize]
    }
}

impl<T: Clone + Copy> IndexMut<UVec2> for Grid<T> {
    fn index_mut(&mut self, index: UVec2) -> &mut Self::Output {
        let w = self.columns();
        &mut self.data[(index.y * w + index.x) as usize]
    }
}
