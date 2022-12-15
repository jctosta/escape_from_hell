use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    time::FixedTimestep,
};
// use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};
use bevy_egui::{
    egui::{self, Color32},
    EguiContext, EguiPlugin,
};
use bevy_kira_audio::{prelude::*, Audio};
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;

pub mod game;

const WINDOW_WIDTH: f32 = 1024.0;
const WINDOW_HEIGHT: f32 = 768.0;

const COLOR_ACCENT: Color = Color::rgb(199.0 / 255.0, 36.0 / 255.0, 177.0 / 255.0);
const COLOR_BACKGROUND: Color = Color::rgb(58.0 / 255.0, 58.0 / 255.0, 89.0 / 255.0);
const COLOR_ACCENT_INVERSE: Color = Color::rgb(113.0 / 255.0, 219.0 / 255.0, 212.0 / 255.0);
const COLOR_NEUTRAL: Color = Color::rgb(179.0 / 255.0, 176.0 / 255.0, 196.0 / 255.0);
const COLOR_YELLOW: Color = Color::rgb(1.0, 211.0 / 255.0, 25.0 / 255.0);
const COLOR_RED: Color = Color::rgb(1.0, 41.0 / 255.0, 117.0 / 255.0);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                width: WINDOW_WIDTH,
                height: WINDOW_HEIGHT,
                title: "Escape from Hell".into(),
                canvas: Some("#bevy".to_owned()),
                ..default()
            },
            ..default()
        }))
        .add_plugin(InputManagerPlugin::<game::Action>::default())
        .add_plugin(AudioPlugin)
        .add_plugin(EguiPlugin)
        .add_plugin(game::GamePlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(ClearColor(COLOR_BACKGROUND))
        .insert_resource(Msaa { samples: 1 })
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .insert_resource(RapierConfiguration {
            gravity: Vec2::ZERO,
            ..default()
        })
        .add_event::<game::GameOverEvent>()
        .run();
}


// Generic system that takes a component as a parameter, and will despawn all entities with that component
fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}
