use bevy::prelude::*;

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Name(String);

#[derive(Resource)]
struct GreetTimer(Timer);

pub struct HelloPlugin;

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GreetTimer(Timer::from_seconds(2.0, TimerMode::Repeating)));
        app.add_systems(Startup, add_people);
        app.add_systems(Update, (update_people, greet_people).chain());
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(HelloPlugin)
        .run();
}

fn add_people(mut commands: Commands) {
    commands.spawn((Person, Name("Arturo García Richardson".to_string())));
    commands.spawn((Person, Name("Ainhoa Palop Almansa".to_string())));
    commands.spawn((Person, Name("Paula Lario Llinares".to_string())));
    commands.spawn((Person, Name("Marta Gómez Verdú".to_string())));
    commands.spawn((Person, Name("Jonathan Ramírez Honrado".to_string())));
}

fn greet_people(time: Res<Time>, mut timer: ResMut<GreetTimer>, query: Query<&Name, With<Person>>) {
    if timer.0.tick(time.delta()).just_finished() {
        for name in &query {
            println!("Saludos, {}", name.0);
        }
    }
}

fn update_people(mut query: Query<&mut Name, With<Person>>) {
    for mut name in &mut query {
        if name.0 == "Arturo García Richardson" {
            name.0 = "Dinastía Harzu".to_string();
            break;
        }
    }
}
