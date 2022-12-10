use bevy::{prelude::*, core_pipeline::bloom::BloomSettings, time::FixedTimestep};
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
        .add_event::<BulletSpawnEvent>()
        .add_event::<BulletDespawnEvent>()
        .add_startup_system(setup)
        .add_system(movement)
        .add_system(dash)
        .add_system(shoot)
        .add_system(spawn_bullet)
        .add_system(bullet_timeout_system)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(10.))
                .with_system(spawn_enemies)
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1.))
                .with_system(move_enemies)   
        )
        .run();
}



#[derive(Component)]
struct Player;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum Action {
    Move,
    Shoot,
    Dash,
}

fn spawn_player(commands: &mut Commands, asset_server: &Res<AssetServer>) {
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
                .insert(KeyCode::Space, Action::Shoot)
                .insert(KeyCode::E, Action::Dash)
                .set_gamepad(Gamepad { id: 0 })
                .build(),
        })
        .insert(Player)
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(32.0))
        .insert(ExternalForce {
            force: Vec2::ZERO,
            torque: 0.0,
        })
        .insert(ExternalImpulse {
            impulse: Vec2::ZERO,
            torque_impulse: 0.0,
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

}

const MOVE_FORCE: f32 = 3000.0;

fn movement(mut query: Query<(&ActionState<Action>, &mut ExternalForce), With<Player>>, time: Res<Time>) {
    for (action_state, mut external_force) in &mut query {
        let axis_vector = action_state.clamped_axis_pair(Action::Move).unwrap().xy();
        external_force.force = axis_vector * MOVE_FORCE * time.delta_seconds(); 
    }
}

fn dash(mut query: Query<(&ActionState<Action>, &mut ExternalImpulse), With<Player>>, time: Res<Time>) {
    for (action_state, mut external_impulse) in &mut query {
        let pressed = action_state.just_pressed(Action::Dash);
        if pressed {
            external_impulse.impulse = Vec2::new(0.0, 2.0) * MOVE_FORCE * time.delta_seconds();
        }
        // @TODO: Add logic to limit the number of dashes a user can make
    }
}


#[derive(Component)]
struct Star;

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
    }).insert(Star);
}


// ******** Bullet Logic *********
// @TODO: Move this logic to another file
#[derive(Component)]
struct Bullet {
    despawn_timer: Timer
}

struct BulletSpawnEvent {
    transform: Transform,
    velocity: Velocity,
}

struct BulletDespawnEvent(Entity);


fn shoot(mut query: Query<(&ActionState<Action>, &Transform), With<Player>>, mut ev_shoot: EventWriter<BulletSpawnEvent>) {
    
    for (action_state, transform) in query.iter_mut() {
        let pressed = action_state.just_pressed(Action::Shoot);
        if pressed {
            ev_shoot.send(BulletSpawnEvent { 
                transform: transform.clone(), 
                velocity: Velocity::linear(Vec2::new(0.0, 300.0)) });
        }
    }
}

fn spawn_bullet(mut bullet_spawn_event: EventReader<BulletSpawnEvent>, mut commands: Commands, asset_server: Res<AssetServer>) {
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
                despawn_timer: Timer::from_seconds(5.0, TimerMode::Once),
            },
            RigidBody::Dynamic,
            Collider::ball(2.5),
            velocity,
            Sensor,
            ActiveEvents::COLLISION_EVENTS,
        ));
    }
    
}

fn bullet_timeout_system(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut Bullet)>) {
    for (entity, mut bullet) in query.iter_mut() {
        bullet.despawn_timer.tick(time.delta());
        if bullet.despawn_timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_stars(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut rng = rand::thread_rng();
    let mut star_count = 0;
    while star_count < 1000 {
        let x: f32 = rng.gen_range(-150.0..150.0);
        let y: f32 = rng.gen_range(0.0..50000.0);
        spawn_star(Vec2::new(x, y), 0.0, &mut commands, &asset_server);
        star_count = star_count + 1;
    }
}

#[derive(Component)]
struct Enemy;

fn spawn_enemy(location: Vec2, rotation: f32, commands: &mut Commands, asset_server: &Res<AssetServer>) {
    let enemy_handle = asset_server.load("enemy_A.png");
    commands.spawn((
        SpriteBundle {
            texture: enemy_handle.clone(),
            transform: Transform {
                translation: location.extend(0.0),
                rotation: Quat::from_rotation_z(rotation),
                ..default()
            },
            ..default()
        },
        RigidBody::Dynamic,
        Collider::ball(2.5),
        Velocity::linear(Vec2::new(1.0, 1.0)),
        Sensor,
        ActiveEvents::COLLISION_EVENTS,
    )).insert(Enemy);
}

fn spawn_enemies(mut commands: Commands, asset_server: Res<AssetServer>, mut query: Query<&Transform, With<Player>>) {
    for (transform) in query.iter_mut() {
        // println!("{:?}", transform);

        let mut rng = rand::thread_rng();
        let mut enemies_count = 0;
        while enemies_count < 10 {
            let x: f32 = rng.gen_range(-200.0..200.0);
            let y: f32 = rng.gen_range(-200.0..200.0);

            let final_x = transform.translation.x + x + 100.0;
            let final_y = transform.translation.y + y + 100.0;
            spawn_enemy(Vec2::new(final_x, final_y), 0.0, &mut commands, &asset_server);
            enemies_count = enemies_count + 1;
        }
    }
}

fn move_enemies(mut enemy_query: Query<(&mut Velocity), With<Enemy>>, time: Res<Time>, mut player_query: Query<&Transform, With<Player>>) {
    let player_transform = player_query.single_mut();
    for mut enemy_velocity in enemy_query.iter_mut() {
        enemy_velocity.linvel = Vec2::new(100.0, 200.0);
        enemy_velocity.angvel = 10.0;
    }
}

// Game Initial Setup
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    
    // Spawn Player
    spawn_player(&mut commands, &asset_server);

    // Spawn Stars
    spawn_stars(commands, asset_server);

}