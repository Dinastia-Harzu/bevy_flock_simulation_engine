use crate::{constants::SCREEN_SIZE, helpers::*};
use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use core::panic;
use itertools::Itertools;
use std::{collections::HashMap, fmt::Debug, ops::RangeInclusive};

#[derive(Resource, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct BoidConfiguration {
    pub min_speed: f32,
    pub max_speed: f32,
    pub scale: f32,
    pub scalar_parametres: HashMap<String, (f32, RangeInclusive<f32>)>,
}

impl BoidConfiguration {
    pub const SPEED_RANGE: RangeInclusive<f32> = 0.0..=500.0;
    pub const SCALE_RANGE: RangeInclusive<f32> = 0.0..=3.0;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn average_speed(&self) -> f32 {
        (self.min_speed + self.max_speed) / 2.0
    }

    pub fn lowest_speed() -> f32 {
        *Self::SPEED_RANGE.start()
    }

    pub fn highest_speed() -> f32 {
        *Self::SPEED_RANGE.end()
    }

    pub fn add_scalar_parametre(
        &mut self,
        name: &str,
        value: f32,
        range: RangeInclusive<f32>,
    ) -> &mut Self {
        self.scalar_parametres
            .insert(name.to_owned(), (value, range));
        self
    }

    pub fn scalar_parametre(&self, name: &str) -> f32 {
        *self.get_scalar_parametre(name).expect(&format!("No existe el parámetro '{name}'")).0
    }

    pub fn scalar_parametre_mut(&mut self, name: &str) -> &mut f32 {
        &mut *self.get_scalar_parametre_mut(name).expect(&format!("No existe el parámetro '{name}'")).0
    }

    pub fn lower_scalar_constant(&self, name: &str) -> f32 {
        *self.get_scalar_range_constant(name).start()
    }

    pub fn upper_scalar_constant(&self, name: &str) -> f32 {
        *self.get_scalar_range_constant(name).end()
    }

    fn get_scalar_parametre(&self, name: &str) -> Option<(&f32, &RangeInclusive<f32>)> {
        match self.scalar_parametres.get(name) {
            Some((value, range)) => Some((value, range)),
            None => None,
        }
    }

    fn get_scalar_parametre_mut(&mut self, name: &str) -> Option<(&mut f32, &RangeInclusive<f32>)> {
        match self.scalar_parametres.get_mut(name) {
            Some((value, range)) => Some((value, range)),
            None => None,
        }
    }

    fn get_scalar_range_constant(&self, name: &str) -> RangeInclusive<f32> {
        self.get_scalar_parametre(name)
            .expect("No existe esto")
            .1
            .clone()
    }
}

impl Default for BoidConfiguration {
    fn default() -> Self {
        Self {
            min_speed: 100.0,
            max_speed: 300.0,
            scale: 1.0,
            scalar_parametres: HashMap::new(),
        }
    }
}

impl IntoIterator for BoidConfiguration {
    type Item = (String, (f32, RangeInclusive<f32>));
    type IntoIter = std::collections::hash_map::IntoIter<String, (f32, RangeInclusive<f32>)>;

    fn into_iter(self) -> Self::IntoIter {
        self.scalar_parametres.into_iter()
    }
}

impl<'a> IntoIterator for &'a BoidConfiguration {
    type Item = (&'a String, (&'a f32, &'a RangeInclusive<f32>));
    type IntoIter = std::iter::Map<
        std::collections::hash_map::Iter<'a, String, (f32, RangeInclusive<f32>)>,
        fn((&'a String, &'a (f32, RangeInclusive<f32>))) -> Self::Item,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.scalar_parametres
            .iter()
            .map(|(k, v)| -> Self::Item { (k, (&v.0, &v.1)) })
    }
}

impl<'a> IntoIterator for &'a mut BoidConfiguration {
    type Item = (&'a String, (&'a mut f32, &'a RangeInclusive<f32>));
    type IntoIter = std::iter::Map<
        std::collections::hash_map::IterMut<'a, String, (f32, RangeInclusive<f32>)>,
        fn((&'a String, &'a mut (f32, RangeInclusive<f32>))) -> Self::Item,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.scalar_parametres
            .iter_mut()
            .map(|(k, v)| -> Self::Item { (k, (&mut v.0, &v.1)) })
    }
}

#[derive(Resource, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct SimulationConfiguration {
    pub should_draw: bool,
    pub normal_boids: u32,
    pub predators: u32,
    pub predator_hunt_weight: f32,
}

impl SimulationConfiguration {
    pub const BOIDS_RANGE: RangeInclusive<u32> = 0..=10000;

    fn new(
        should_draw: bool,
        normal_boids: u32,
        predators: u32,
        predator_hunt_weight: f32,
    ) -> Self {
        Self {
            should_draw,
            normal_boids,
            predators,
            predator_hunt_weight,
        }
    }

    pub fn min_boids() -> u32 {
        *Self::BOIDS_RANGE.start()
    }

    pub fn max_boids() -> u32 {
        *Self::BOIDS_RANGE.end()
    }
}

impl Default for SimulationConfiguration {
    fn default() -> Self {
        Self::new(true, 100, 1, 0.25)
    }
}

pub struct BoidRuleParametres<'a> {
    pub entity: Entity,
    pub position: Vec2,
    pub velocity: Vec2,
    pub cell: &'a SpatialGridCell,
}

pub trait Rule: Fn(BoidRuleParametres, &BoidConfiguration) -> Vec2 + Send + Sync + 'static {}
impl<T: Fn(BoidRuleParametres, &BoidConfiguration) -> Vec2 + Send + Sync + 'static> Rule for T {}

#[derive(Resource, Default)]
pub struct BoidRules(Vec<Box<dyn Rule>>);

impl BoidRules {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, rule: impl Rule) -> &mut Self {
        self.0.push(Box::new(rule));
        self
    }
}

impl<'a> IntoIterator for &'a BoidRules {
    type Item = &'a Box<dyn Rule>;
    type IntoIter = std::slice::Iter<'a, Box<dyn Rule>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[derive(Resource, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct SpatialGrid {
    cells: Cells,
    rows: u32,
    columns: u32,
}

impl SpatialGrid {
    pub fn new(rows: u32, columns: u32, cell_size: f32) -> Self {
        assert!(
            rows > 0 && columns > 0,
            "Prohibido crear un SpatialGrid unidimensional o nildimensional o de dimensiones negativas"
        );
        assert!(
            cell_size > 1.0,
            "Es inútil trabajar con celdas más pequeñas que un píxel"
        );
        let mut cells = Vec::new();
        let offset = Vec2::new((columns - 1) as f32, (rows - 1) as f32) / 2.0;
        for r in 0..rows {
            for c in 0..columns {
                cells.push(SpatialGridCell::new(
                    r,
                    c,
                    cell_size,
                    (Vec2::new(c as f32, r as f32) - offset) * cell_size,
                ));
            }
        }
        Self {
            cells,
            rows,
            columns,
        }
    }

    pub fn with_cell_size(cell_size: f32) -> Self {
        assert!(
            cell_size > 1.0,
            "Es inútil trabajar con celdas más pequeñas que un píxel"
        );
        let UVec2 {
            x: columns,
            y: rows,
        } = (SCREEN_SIZE / cell_size).floor().as_uvec2();
        Self::new(rows, columns, cell_size)
    }

    pub fn cells(&self) -> &Cells {
        &self.cells
    }

    pub fn cells_mut(&mut self) -> &mut Cells {
        &mut self.cells
    }

    pub fn columns(&self) -> u32 {
        self.columns
    }

    pub fn rows(&self) -> u32 {
        self.rows
    }

    pub fn cell_size(&self) -> f32 {
        self.at_index(0).size()
    }

    pub fn grid_size(&self) -> Vec2 {
        UVec2::new(self.columns, self.rows).as_vec2() * self.cell_size()
    }

    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            cell.boids.clear();
        }
    }

    pub fn iter_radius(&self, centre: Vec2, radius: f32) -> SpatialGridInRadiusIter {
        SpatialGridInRadiusIter::new(self, centre, radius)
    }

    #[must_use = "No vas a usar este SpatialGridCell?"]
    pub fn at(&self, row: usize, column: usize) -> &SpatialGridCell {
        if column >= self.columns as usize {
            panic!("{column} no puede ser mayor que {}", self.columns);
        }
        &self.cells[row * self.columns as usize + column]
    }

    #[must_use = "No vas a usar este SpatialGridCell?"]
    pub fn at_mut(&mut self, row: usize, column: usize) -> &mut SpatialGridCell {
        &mut self.cells[row * self.columns as usize + column]
    }

    pub fn get(&self, row: usize, column: usize) -> Option<&SpatialGridCell> {
        if column >= self.columns as usize {
            None
        } else {
            self.cells.get(row * self.columns as usize + column)
        }
    }

    #[must_use = "No vas a usar este SpatialGridCell?"]
    pub fn at_world_position(&self, world_position: Vec2) -> &SpatialGridCell {
        self.at_index(self.index_from_world_position(world_position))
    }

    #[must_use = "No vas a usar este SpatialGridCell?"]
    pub fn at_world_position_mut(&mut self, world_position: Vec2) -> &mut SpatialGridCell {
        self.at_index_mut(self.index_from_world_position(world_position))
    }

    pub fn try_at_world_position(&self, world_position: Vec2) -> Option<&SpatialGridCell> {
        match self.try_index_from_world_position(world_position) {
            Ok(index) => Some(self.at_index(index)),
            Err(_) => None,
        }
    }

    #[must_use = "No vas a usar este SpatialGridCell?"]
    pub fn at_index(&self, index: usize) -> &SpatialGridCell {
        &self.cells[index]
    }

    #[must_use = "No vas a usar este SpatialGridCell?"]
    pub fn at_index_mut(&mut self, index: usize) -> &mut SpatialGridCell {
        &mut self.cells[index]
    }

    pub fn index_from_world_position(&self, world_position: Vec2) -> usize {
        self.try_index_from_world_position(world_position).unwrap()
    }

    pub fn try_index_from_world_position(&self, world_position: Vec2) -> Result<usize, String> {
        let total_cells = self.rows * self.columns;
        let half_size = self.grid_size() / 2.0;
        if !(-half_size.x..half_size.x).contains(&world_position.x)
            || !(-half_size.y..half_size.y).contains(&world_position.y)
        {
            return Err(format!(
                "La posición {world_position} no entra en el rango [x: {}..{}, y: {}..{}]",
                -half_size.x, half_size.x, -half_size.y, half_size.y
            ));
        }
        let UVec2 { x: column, y: row } =
            ((world_position + half_size) / self.cell_size()).as_uvec2();
        let i = row * self.columns + column;
        if i >= total_cells {
            return Err(format!("La conversión posición global ({world_position}) -> índice debe dar menor que {total_cells}, pero ha dado {row} * {} + {column} = {i}", self.columns));
        }
        Ok(i as usize)
    }
}

impl Debug for SpatialGrid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, cell) in self.cells.iter().enumerate() {
            let i = i as u32;
            writeln!(
                f,
                "Celda ({}, {}): {:?}",
                i / self.rows,
                i % self.columns,
                cell
            )?;
        }
        Ok(())
    }
}

pub struct SpatialGridInRadiusIter<'g> {
    spatial_grid: &'g SpatialGrid,
    index: UVec2,
    index_ranges: (RangeInclusive<u32>, RangeInclusive<u32>),
}

impl<'g> SpatialGridInRadiusIter<'g> {
    pub fn new(spatial_grid: &'g SpatialGrid, centre: Vec2, radius: f32) -> Self {
        let inf = centre - radius;
        let sup = centre + radius;
        let offset = spatial_grid.grid_size() / 2.0;
        let reckon_indices_from = |point: Vec2| {
            ((point + offset) / spatial_grid.cell_size())
                .floor()
                .as_uvec2()
        };
        let inf_index = reckon_indices_from(inf);
        let sup_index = reckon_indices_from(sup);
        let index_ranges = (inf_index.x..=sup_index.x, inf_index.y..=sup_index.y);
        Self {
            spatial_grid,
            index: inf_index,
            index_ranges,
        }
    }
}

impl<'g> Iterator for SpatialGridInRadiusIter<'g> {
    type Item = &'g SpatialGridCell;

    fn next(&mut self) -> Option<Self::Item> {
        let (x_range, y_range) = &self.index_ranges;
        loop {
            if !y_range.contains(&self.index.y) {
                break None;
            }
            let cell = self
                .spatial_grid
                .get(self.index.y as usize, self.index.x as usize);
            self.index.x += 1;
            if !x_range.contains(&self.index.x) {
                self.index.x = *x_range.start();
                self.index.y += 1;
            }
            if cell.is_some() {
                break cell;
            }
        }
    }
}
