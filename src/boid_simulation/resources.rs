use std::fmt::Debug;

use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;

#[derive(Resource, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct BoidConfiguration {
    pub min_speed: f32,
    pub max_speed: f32,
    pub inner_perception_radius: f32,
    pub outer_perception_radius: f32,
    pub separation_factor: f32,
    pub alignment_factor: f32,
    pub cohesion_factor: f32,
    pub threshold: f32,
}

impl BoidConfiguration {
    pub const MAX_VEL: f32 = 100.0;
    pub const MAX_BOIDS: u32 = 500;
    pub const MAX_INNER_PERCEPTION_RADIUS: f32 = 500.0;
    pub const MAX_OUTER_PERCEPTION_RADIUS: f32 = 2000.0;
    pub const MAX_SEPARATION_FACTOR: f32 = 10.0;
    pub const MAX_ALIGNMENT_FACTOR: f32 = 10.0;
    pub const MAX_COHESION_FACTOR: f32 = 10.0;
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
            boids: Vec::with_capacity(BoidConfiguration::MAX_BOIDS as usize),
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
        if world_position.is_nan() {
            dbg!("AINHOA CÁLLATE");
        }
        let half_size = self.grid_size() / 2.0;
        assert!(
            (-half_size.x..=half_size.x).contains(&world_position.x)
                && (-half_size.y..=half_size.y).contains(&world_position.y),
            "La posición {world_position} no entra en ninguna celda del SpatialGrid"
        );
        let UVec2 { x: column, y: row } =
            ((world_position + half_size) / self.cell_size()).as_uvec2();
        (row * self.columns + column) as usize
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
}

impl SimulationConfiguration {
    fn new(should_draw: bool) -> Self {
        Self { should_draw }
    }
}

impl Default for SimulationConfiguration {
    fn default() -> Self {
        Self::new(true)
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
