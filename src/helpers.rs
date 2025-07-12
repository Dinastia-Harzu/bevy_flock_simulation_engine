use bevy::prelude::*;
use std::ops::{Index, IndexMut};

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
