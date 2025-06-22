use crate::{components::*, constants::*, resources::*, states::*};
use bevy::{
    color::palettes::css::*,
    math::{FloatPow, NormedVectorSpace},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_egui::*;
use core::f32;
use rand::Rng;
use std::collections::HashMap;

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(BoidSprite {
        fireball_handle: asset_server.load("textures/wave-fireball.png"),
        galaga_ship_handle: asset_server.load("textures/wave-blue-fireball.png"),
        size: (63.0, 35.0).into(),
    });
    commands.insert_resource(BoidConfiguration {
        speed: 100.0,
        inner_perception_radius: 100.0,
        outer_perception_radius: 500.0,
        separation_factor: 1.0,
        alignment_factor: 1.0,
        cohesion_factor: 1.0,
    });
    let max_boids = BoidConfiguration::MAX_BOIDS as usize;
    commands.insert_resource(BoidEntities {
        entities: Vec::with_capacity(max_boids + 1),
        current_id: max_boids,
    });

    commands.spawn(Camera2d);
}

pub fn inspector_ui(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
    else {
        return;
    };
    let mut egui_context = egui_context.clone();

    egui::Window::new("UI").show(egui_context.get_mut(), |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            bevy_inspector_egui::bevy_inspector::ui_for_world(world, ui);
            ui.heading("Entities");
            bevy_inspector_egui::bevy_inspector::ui_for_entities(world, ui);
        });
    });
}

pub fn spawn_boids(
    mut commands: Commands,
    boid_configuration: Res<BoidConfiguration>,
    boid_sprite: Res<BoidSprite>,
    mut boid_entities: ResMut<BoidEntities>,
) {
    let mut rng = rand::rng();
    let pi = f32::consts::PI;
    let bounds = SCREEN_SIZE / 2.0;
    for _ in 0..BoidConfiguration::MAX_BOIDS {
        let angle = rng.random_range(-pi..=pi) - f32::consts::FRAC_PI_2;
        let entity = commands
            .spawn((
                Boid {
                    speed: boid_configuration.speed,
                    angle,
                },
                Sprite {
                    image: boid_sprite.fireball_handle.clone(),
                    custom_size: Some(boid_sprite.size),
                    ..Default::default()
                },
                Transform {
                    translation: (
                        rng.random_range(-bounds.x..=bounds.x),
                        rng.random_range(-bounds.y..=bounds.y),
                        0.0,
                    )
                        .into(),
                    rotation: Quat::from_axis_angle(Vec3::Z, angle),
                    ..Default::default()
                },
            ))
            .id();
        boid_entities.entities.push(entity);
    }
    let selected_entity = commands
        .spawn((
            Boid {
                speed: 0.0,
                angle: 0.0,
            },
            Sprite {
                image: boid_sprite.galaga_ship_handle.clone(),
                custom_size: Some(boid_sprite.size),
                ..Default::default()
            },
            Transform::from_xyz(0.0, 0.0, 1.0),
            BoidTestingUnit,
        ))
        .id();
    boid_entities.entities.push(selected_entity);
}

pub fn update_boids(
    mut query: Query<(Entity, &mut Boid, &mut Transform)>,
    boid_configuration: Res<BoidConfiguration>,
    time: Res<Time>,
) {
    let mut steerings = HashMap::new();
    let mut combinations = query.iter_combinations();
    while let Some([(entity1, boid1, transform1), (entity2, boid2, transform2)]) =
        combinations.next()
    {
        let pos1 = transform1.translation.xy();
        let pos2 = transform2.translation.xy();
        let initial_values = (Vec2::ZERO, Vec2::ZERO, Vec2::ZERO, 0.0f32, 0.0f32);
        let pos1_to_pos2_squared = pos1.distance_squared(pos2);
        let pos1_to_pos2 = pos1.distance(pos2);
        let in_inner_radius =
            pos1_to_pos2_squared <= boid_configuration.inner_perception_radius.squared();
        let in_outer_radius =
            pos1_to_pos2_squared <= boid_configuration.outer_perception_radius.squared();

        let (cohesion1, separation1, alignment1, total_outer1, total_inner1) =
            steerings.entry(entity1).or_insert(initial_values);
        if in_outer_radius {
            *cohesion1 += pos2;
            *alignment1 += boid2.velocity();
            *total_outer1 += 1.0;
        }
        if in_inner_radius {
            let other_to_current = pos1 - pos2;
            *separation1 += other_to_current / pos1_to_pos2;
            *total_inner1 += 1.0;
        }

        let (cohesion2, separation2, alignment2, total_outer2, total_inner2) =
            steerings.entry(entity2).or_insert(initial_values);
        if in_outer_radius {
            *cohesion2 += pos1;
            *alignment2 += boid1.velocity();
            *total_outer2 += 1.0;
        }
        if in_inner_radius {
            let other_to_current = pos2 - pos1;
            *separation2 += other_to_current / pos1_to_pos2;
            *total_inner2 += 1.0;
        }
    }
    for (entity, mut boid, mut transform) in &mut query {
        if let Some((cohesion, separation, alignment, total_outer, total_inner)) =
            steerings.get_mut(&entity)
        {
            *cohesion /= *total_outer;
            *cohesion = (*cohesion - transform.translation.xy()) / 100.0;
            *cohesion *= boid_configuration.cohesion_factor;

            *separation /= *total_inner;
            *separation *= boid_configuration.separation_factor;

            *alignment /= *total_outer;
            *alignment = (*alignment - boid.velocity()) / 8.0;
            *alignment *= boid_configuration.alignment_factor;

            let mut velocity = boid.velocity();
            velocity += *cohesion + *alignment + *separation;
            // velocity += *cohesion + *alignment;
            // velocity += *separation;
            velocity =
                velocity.normalize_or(Vec2::from_angle(boid.angle)) * boid_configuration.speed;

            boid.speed = velocity.norm();
            boid.angle = velocity.to_angle();
            transform.translation += (velocity * time.delta_secs()).extend(0.0);
            transform.rotation = Quat::from_axis_angle(Vec3::Z, boid.angle);
        }
    }
}

pub fn wrap_edges(mut query: Query<&mut Transform, With<Boid>>) {
    for mut transform in &mut query {
        let position = &mut transform.translation;
        let Vec2 { x: bx, y: by } = SCREEN_SIZE / 2.0;
        if position.x > bx {
            position.x = -bx;
        } else if position.x < -bx {
            position.x = bx;
        }
        if position.y > by {
            position.y = -by;
        } else if position.y < -by {
            position.y = by;
        }
    }
}

pub fn update_debug_boid(
    boid_query: Option<Single<&Transform, With<BoidTestingUnit>>>,
    boid_configuration: Res<BoidConfiguration>,
    mut gizmos: Gizmos,
) {
    if let Some(boid) = boid_query {
        let transform = boid.into_inner();
        gizmos
            .circle_2d(
                transform.translation.xy(),
                boid_configuration.inner_perception_radius,
                RED,
            )
            .resolution(64);
        gizmos
            .circle_2d(
                transform.translation.xy(),
                boid_configuration.outer_perception_radius,
                GREEN,
            )
            .resolution(64);
    }
}

pub fn common_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut app_next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        app_next_state.set(AppState::Finished);
    }
}

pub fn exit(mut commands: Commands, window: Single<Entity, With<Window>>) {
    commands.entity(window.entity()).despawn();
}
