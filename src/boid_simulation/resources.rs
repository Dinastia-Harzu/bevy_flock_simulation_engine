use std::{collections::HashMap, fmt::Debug, ops::RangeInclusive};

use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;

#[derive(Resource, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct BoidConfiguration {
    pub min_speed: f32,
    pub max_speed: f32,
    pub boid_count: u32,
    pub scale: f32,
    pub scalar_parametres: HashMap<String, (f32, RangeInclusive<f32>)>,
}

impl BoidConfiguration {
    pub const SPEED_RANGE: RangeInclusive<f32> = 10.0..=500.0;
    pub const BOIDS_RANGE: RangeInclusive<u32> = 3..=5000;
    pub const SCALE_RANGE: RangeInclusive<f32> = 0.0..=3.0;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn lowest_speed() -> f32 {
        *Self::SPEED_RANGE.start()
    }

    pub fn highest_speed() -> f32 {
        *Self::SPEED_RANGE.end()
    }

    pub fn min_boids() -> u32 {
        *Self::BOIDS_RANGE.start()
    }

    pub fn max_boids() -> u32 {
        *Self::BOIDS_RANGE.end()
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
        *self.get_scalar_parametre(name).expect("No existe ningún parámetro con este nombre. Asegúrate de que lo has escrito bien o de que realmente has añadido este parámetro").0
    }

    pub fn scalar_parametre_mut(&mut self, name: &str) -> &mut f32 {
        &mut *self.get_scalar_parametre_mut(name).expect("No existe ningún parámetro con este nombre. Asegúrate de que lo has escrito bien o de que realmente has añadido este parámetro").0
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
            boid_count: Self::max_boids() / 10,
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

type CellBoids = Vec<SpatialGridBoid>;
type Cells = Vec<SpatialGridCell>;

#[derive(Reflect)]
pub struct SpatialGridCell {
    grid_pos: UVec2,
    rect: Rect,
    boids: CellBoids,
}

impl SpatialGridCell {
    fn new(row: u32, column: u32, size: f32, centre: Vec2) -> Self {
        Self {
            grid_pos: (row, column).into(),
            rect: Rect::from_center_size(centre, Vec2::new(size, size)),
            boids: Vec::with_capacity(BoidConfiguration::max_boids() as usize),
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
            "Prohibido crear un SpatialGrid unidimensional o nildimensional"
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

    #[must_use = "No vas a usar este SpatialGridCell?"]
    pub fn at(&self, row: usize, column: usize) -> &SpatialGridCell {
        &self.cells[row * self.columns as usize + column]
    }

    #[must_use = "No vas a usar este SpatialGridCell?"]
    pub fn at_mut(&mut self, row: usize, column: usize) -> &mut SpatialGridCell {
        &mut self.cells[row * self.columns as usize + column]
    }

    #[must_use = "No vas a usar este SpatialGridCell?"]
    pub fn at_world_position(&self, world_position: Vec2) -> &SpatialGridCell {
        self.at_index(self.index_from_world_position(world_position))
    }

    #[must_use = "No vas a usar este SpatialGridCell?"]
    pub fn at_world_position_mut(&mut self, world_position: Vec2) -> &mut SpatialGridCell {
        self.at_index_mut(self.index_from_world_position(world_position))
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
        let total_cells = self.rows * self.columns;
        let half_size = self.grid_size() / 2.0;
        assert!(
            (-half_size.x..half_size.x).contains(&world_position.x)
                && (-half_size.y..half_size.y).contains(&world_position.y),
            "La posición {world_position} no entra en ninguna celda del SpatialGrid"
        );
        let UVec2 { x: column, y: row } =
            ((world_position + half_size) / self.cell_size()).as_uvec2();
        let i = row * self.columns + column;
        assert!(i < total_cells, "La conversión posición global ({world_position}) -> índice debe dar menor que {total_cells}, pero ha dado {row} * {} + {column} = {i}", self.columns);
        i as usize
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

#[derive(Resource, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct SimulationConfiguration {
    pub should_draw: bool,
    pub with_predator: bool,
}

impl SimulationConfiguration {
    fn new(should_draw: bool, with_predator: bool) -> Self {
        Self {
            should_draw,
            with_predator,
        }
    }
}

impl Default for SimulationConfiguration {
    fn default() -> Self {
        Self::new(true, true)
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
