use bevy::{prelude::*, core_pipeline::bloom::BloomSettings};
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;
use bevy_inspector_egui::{WorldInspectorPlugin};
use rand::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor { 
                width: 800.0, 
                height: 600.0, 
                title: "Escape From Hell".into(),
                ..default()
            },
            ..default()
        }))
        .add_plugin(InputManagerPlugin::<Action>::default())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(WorldInspectorPlugin::new())
        .insert_resource(RapierConfiguration {
            gravity: Vec2::ZERO,
            ..default()
        })
        .insert_resource(ClearColor(Color::MIDNIGHT_BLUE))
        .add_startup_system(setup)
        .add_system(movement)
        .run();
}

#[derive(Component)]
struct Player {
    id: usize,
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum Action {
    Move,
    Shoot,
}

fn spawn_player(id: usize, commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands
        .spawn(
            SpriteBundle {
                transform: Transform::from_translation(Vec2::new(0.0, 0.0).extend(1.0)),
                texture: asset_server.load("ship_H.png"),
                ..default()
            }
        )
        .insert(InputManagerBundle::<Action> {
            action_state: ActionState::default(),
            input_map: InputMap::default()
                .insert(DualAxis::left_stick(), Action::Move)
                .insert(VirtualDPad::wasd(), Action::Move)
                .insert(VirtualDPad::arrow_keys(), Action::Move)
                .set_gamepad(Gamepad { id: 0 })
                .build(),
        })
        .insert(Player { id })
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(32.0))
        .insert(ExternalForce {
            force: Vec2::ZERO,
            torque: 0.0,
        })
        .insert(Damping {
            linear_damping: 0.6,
            angular_damping: 5.0,
        })
        .insert(Restitution::coefficient(1.0))
        .insert((
            Camera2dBundle {
                camera: Camera {
                    hdr: true,
                    ..default()
                },
                ..default()
            },
            BloomSettings {
                intensity: 1.0,
                ..default()
            }
        ));

        // commands.spawn((
        //     Camera2dBundle {
        //         camera: Camera {
        //             hdr: true,
        //             ..default()
        //         },
        //         ..default()
        //     },
        //     BloomSettings {
        //         intensity: 1.0,
        //         ..default()
        //     },
        // ));

}

const MOVE_FORCE: f32 = 15000.0;

fn movement(mut query: Query<(&ActionState<Action>, &mut ExternalForce), With<Player>>, time: Res<Time>) {
    for (action_state, mut external_force) in &mut query {
        let axis_vector = action_state.clamped_axis_pair(Action::Move).unwrap().xy();
        external_force.force = axis_vector * MOVE_FORCE * time.delta_seconds(); 
    }
}

struct BulletSpawnEvent {
    transform: Transform,
    velocity: Velocity,
}

struct BulletDespawnEvent(Entity);


fn shoot(mut query: Query<(&ActionState<Action>, &Transform), With<Player>>, mut ev_shoot: EventWriter<BulletSpawnEvent>) {
    
    for (action_state, transform) in query.iter_mut() {
        let pressed = action_state.pressed(Action::Shoot);
        if pressed {
            ev_shoot.send(BulletSpawnEvent { 
                transform: transform.clone(), 
                velocity: Velocity::linear(Vec2::new(2.0, 2.0)) });
        }
    }
}

fn spawn_star(location: Vec2, rotation: f32, commands: &mut Commands, asset_server: &Res<AssetServer>) {
    let star_handle = asset_server.load("star_small.png");
    commands.spawn(SpriteBundle {
        texture: star_handle.clone(),
        transform: Transform {
            translation: location.extend(0.0),
            rotation: Quat::from_rotation_z(rotation),
            ..default()
        },
        ..default()
    });
}

#[derive(Component)]
struct Bullet {
    despawn_timer: Timer
}

fn spawn_bullet(commands: &mut Commands, asset_server: &Res<AssetServer>, mut bullet_spawn_event: EventReader<BulletSpawnEvent>) {
    let bullet_handle = asset_server.load("star_tiny.png");

    for spawn_event in bullet_spawn_event.iter() {
        let transform = spawn_event.transform;
        let velocity = spawn_event.velocity;
        commands.spawn((
            SpriteBundle {
                texture: bullet_handle.clone(),
                transform: Transform {
                    translation: transform.translation,
                    ..default()
                },
                ..default()
            },
            Bullet {
                despawn_timer: Timer::from_seconds(2.0, TimerMode::Once),
            },
            RigidBody::Dynamic,
            Collider::ball(2.5),
            velocity,
            Sensor,
            ActiveEvents::COLLISION_EVENTS,
        ));
    }
    
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // commands.spawn((
    //     Camera2dBundle {
    //         camera: Camera {
    //             hdr: true,
    //             ..default()
    //         },
    //         ..default()
    //     },
    //     BloomSettings {
    //         intensity: 1.0,
    //         ..default()
    //     },
    // ));
    // Spawn Player
    spawn_player(0, &mut commands, &asset_server);

    // Spawn Stars
    let mut rng = rand::thread_rng();
    let mut star_count = 0;
    while star_count < 100 {
        let x: f32 = rng.gen_range(-1500.0..1500.0);
        let y: f32 = rng.gen_range(-1500.0..1500.0);
        spawn_star(Vec2::new(x, y), 0.0, &mut commands, &asset_server);
        star_count = star_count + 1;
    }

}