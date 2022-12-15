use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    time::FixedTimestep,
};
use bevy_kira_audio::{prelude::*, Audio};
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;
use rand::prelude::*;
use std::time::Duration;

use bevy_egui::{
    egui::{self, Color32},
    EguiContext, EguiPlugin,
};

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum Action {
    Move,
}

#[derive(Component)]
pub struct Player {
    move_speed: f32,
    life: u64,
}

#[derive(Component)]
pub struct GameTime {
    seconds: u64,
}

#[derive(Component)]
pub struct FpsText;

#[derive(Component)]
pub struct PlayerLife;

#[derive(Component)]
pub struct Enemy {
    move_speed: f32,
}

pub struct GameOverEvent(Entity);

fn spawn_player(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("ship_I.png"),
            transform: Transform::from_translation(Vec2::new(0.0, 0.0).extend(1.0)),
            sprite: Sprite {
                color: crate::COLOR_ACCENT,
                ..default()
            },
            ..default()
        },
        InputManagerBundle::<Action> {
            action_state: ActionState::default(),
            input_map: InputMap::default()
                .insert(DualAxis::left_stick(), Action::Move)
                .insert(VirtualDPad::wasd(), Action::Move)
                .insert(VirtualDPad::arrow_keys(), Action::Move)
                .set_gamepad(Gamepad { id: 0 })
                .build(),
        },
        Player {
            move_speed: 300.0,
            life: 10,
        },
        RigidBody::Dynamic,
        Velocity::zero(),
        Collider::ball(20.0),
        ActiveEvents::CONTACT_FORCE_EVENTS,
        Restitution::coefficient(1.0),
        Ccd::enabled(),
        Damping {
            linear_damping: 0.6,
            angular_damping: 5.0,
        },
        ExternalForce {
            force: Vec2::ZERO,
            torque: 0.0,
        },
    ));
}

fn player_movement(
    mut query: Query<(&ActionState<Action>, &mut Transform), With<Player>>,
    player_query: Query<&Player>,
    time: Res<Time>,
) {
    let player = player_query.single();
    for (action_state, mut transform) in &mut query {
        let axis_vector = action_state.clamped_axis_pair(Action::Move).unwrap();
        let movement = Vec2::new(axis_vector.x(), axis_vector.y());

        let rotate_player =
            Quat::from_rotation_arc(Vec3::Y, movement.extend(0.0).normalize_or_zero());
        transform.rotation = rotate_player;

        transform.translation += movement.extend(0.0) * time.delta_seconds() * player.move_speed;

        // Clamp player to window
        transform.translation.x = transform
            .translation
            .x
            .clamp(-(crate::WINDOW_WIDTH / 2.0), crate::WINDOW_WIDTH / 2.0);
        transform.translation.y = transform
            .translation
            .y
            .clamp(-(crate::WINDOW_HEIGHT / 2.0), crate::WINDOW_HEIGHT / 2.0);
    }
}

fn spawn_enemy_group(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut rng = rand::thread_rng();
    let group_size = 3;
    let mut enemy_count = 0;

    let colors = [
        crate::COLOR_ACCENT_INVERSE,
        crate::COLOR_YELLOW,
        crate::COLOR_RED,
    ];
    let images: [Handle<Image>; 3] = [
        asset_server.load("enemy_B.png"),
        asset_server.load("enemy_D.png"),
        asset_server.load("enemy_E.png"),
    ];

    while enemy_count < group_size {
        let x: f32 = rng.gen_range(-200.0..200.0);
        let y: f32 = rng.gen_range(0.0..200.0);

        let enemy_type: usize = rng.gen_range(0..3);

        let selected_color = colors[enemy_type].clone();
        let selected_sprite = images[enemy_type].clone();

        spawn_enemy(
            Vec3::new(x, y, 1.0),
            selected_color,
            selected_sprite,
            &mut commands,
        );

        enemy_count += 1;
    }
}

fn spawn_enemy(position: Vec3, color: Color, texture: Handle<Image>, commands: &mut Commands) {
    commands.spawn((
        SpriteBundle {
            texture,
            transform: Transform::from_translation(position),
            sprite: Sprite { color, ..default() },
            ..default()
        },
        Enemy { move_speed: 50.0 },
        RigidBody::Dynamic,
        Velocity::zero(),
        Collider::ball(20.0),
        ActiveEvents::CONTACT_FORCE_EVENTS,
        Restitution::coefficient(1.0),
        Ccd::enabled(),
        Damping {
            linear_damping: 0.6,
            angular_damping: 5.0,
        },
        ExternalForce {
            force: Vec2::ZERO,
            torque: 0.0,
        },
    ));
}

type OnlyEnemies = (With<Enemy>, Without<Player>);
type OnlyPlayer = (With<Player>, Without<Enemy>);

fn enemy_movement(
    mut query: Query<(&mut Transform, &Enemy), OnlyEnemies>,
    player_query: Query<&Transform, OnlyPlayer>,
    time: Res<Time>,
) {
    // let min = Vec3::new(-(WINDOW_WIDTH / 2.0), WINDOW_HEIGHT / 2.0, 1.0);
    // let max = Vec3::new(WINDOW_WIDTH / 2.0, -(WINDOW_HEIGHT / 2.0), 1.0);
    let player_transform = player_query.single();
    for (mut enemy_transform, enemy) in query.iter_mut() {
        // get player direction
        let angle =
            (player_transform.translation - enemy_transform.translation).normalize_or_zero();
        // rotate enemy to face player
        let rotate_to_player = Quat::from_rotation_arc(Vec3::Y, angle);
        enemy_transform.rotation = rotate_to_player;
        // move enemy to player
        enemy_transform.translation +=
            angle * time.delta_seconds() * (enemy.move_speed + time.elapsed_seconds());

        // Clamp enemy to window
        enemy_transform.translation.x = enemy_transform
            .translation
            .x
            .clamp(-(crate::WINDOW_WIDTH / 2.0), crate::WINDOW_WIDTH / 2.0);
        enemy_transform.translation.y = enemy_transform
            .translation
            .y
            .clamp(-(crate::WINDOW_HEIGHT / 2.0), crate::WINDOW_HEIGHT / 2.0);
    }
}

#[derive(Component)]
pub struct Star;

fn spawn_star(
    location: Vec2,
    rotation: f32,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) {
    let star_handle = asset_server.load("star_small.png");
    commands
        .spawn(SpriteBundle {
            texture: star_handle.clone(),
            transform: Transform {
                translation: location.extend(0.0),
                rotation: Quat::from_rotation_z(rotation),
                ..default()
            },
            sprite: Sprite {
                color: crate::COLOR_NEUTRAL.as_rgba().set_a(0.1).clone(),
                ..default()
            },
            ..default()
        })
        .insert(Star);
}

fn spawn_stars(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut rng = rand::thread_rng();
    let mut star_count = 0;
    while star_count < 100 {
        let x: f32 = rng.gen_range(-(crate::WINDOW_WIDTH / 2.0)..(crate::WINDOW_WIDTH / 2.0));
        let y: f32 = rng.gen_range(-(crate::WINDOW_HEIGHT / 2.0)..(crate::WINDOW_HEIGHT / 2.0));
        spawn_star(Vec2::new(x, y), 0.0, &mut commands, &asset_server);
        star_count = star_count + 1;
    }
}

fn update_game_time(mut query: Query<(&mut Text, &mut GameTime), With<GameTime>>) {
    let (mut game_time_text, mut game_time) = query.single_mut();
    game_time.seconds += 1;
    let new_duration = Duration::new(game_time.seconds, 0);
    // let seconds = new_duration.as_secs() % 60;
    let seconds = (new_duration.as_secs() / 60) % 60;
    let minutes = (new_duration.as_secs() / 60) / 60;
    let seconds_display = if seconds < 10 {
        format!("0{}", seconds)
    } else {
        format!("{}", seconds)
    };
    let minutes_display = if minutes < 10 {
        format!("0{}", minutes)
    } else {
        format!("{}", minutes)
    };
    game_time_text.sections[0].value = format!("{}:{}", minutes_display, seconds_display);
}

fn update_fps(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text, With<FpsText>>) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                text.sections[1].value = format!("{value:.2}");
            }
        }
    }
}

fn update_player_health(
    mut query: Query<&mut Text, With<PlayerLife>>,
    player_query: Query<&Player>,
) {
    let player = player_query.single();
    for mut text in &mut query {
        text.sections[1].value = format!("{}", player.life);
    }
}

fn configure_visuals(mut egui_ctx: ResMut<EguiContext>) {
    egui_ctx.ctx_mut().set_visuals(egui::Visuals {
        window_rounding: 0.0.into(),
        panel_fill: Color32::LIGHT_GREEN,
        ..Default::default()
    });
}

fn start_background_audio(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    audio
        .play(asset_server.load("music/Ludum_Dare_28_Track_1.ogg"))
        .with_volume(0.5)
        .looped();
}

fn player_contact(
    mut player_entity_query: Query<(Entity, &mut Player), With<Player>>,
    mut contact_event: EventReader<ContactForceEvent>,
    mut game_over_event: EventWriter<GameOverEvent>,
) {
    let (player_entity, mut player) = player_entity_query.single_mut();
    for contact_event in contact_event.iter() {
        if contact_event.collider1 == player_entity || contact_event.collider2 == player_entity {
            info!("Player collided");
            if player.life > 0 {
                player.life -= 1;
            } else {
                game_over_event.send(GameOverEvent(player_entity));
            }
        }
    }
}

fn handle_game_over(mut game_over_event: EventReader<GameOverEvent>) {
    for evt in game_over_event.iter() {
        info!("Entity {:?} lost the game", evt.0);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle {
        camera: Camera { ..default() },
        ..default()
    });

    commands.spawn((
        TextBundle::from_section(
            "00:00",
            TextStyle {
                font: asset_server.load("fonts/Minecraft.otf"),
                font_size: 30.0,
                color: Color::WHITE,
            },
        )
        .with_text_alignment(TextAlignment::TOP_CENTER)
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Px(5.0),
                right: Val::Px(15.0),
                ..default()
            },
            ..default()
        }),
        GameTime { seconds: 0 },
    ));

    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "FPS: ",
                TextStyle {
                    font: asset_server.load("fonts/Minecraft-Bold.otf"),
                    font_size: 10.0,
                    color: crate::COLOR_ACCENT,
                },
            ),
            TextSection::from_style(TextStyle {
                font: asset_server.load("fonts/Minecraft.otf"),
                font_size: 10.0,
                color: crate::COLOR_ACCENT,
            }),
        ]),
        FpsText,
    ));

    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "Health: ",
                TextStyle {
                    font: asset_server.load("fonts/Minecraft-Bold.otf"),
                    font_size: 30.0,
                    color: crate::COLOR_ACCENT,
                },
            ),
            TextSection::from_style(TextStyle {
                font: asset_server.load("fonts/Minecraft.otf"),
                font_size: 30.0,
                color: crate::COLOR_ACCENT,
            }),
        ])
        .with_text_alignment(TextAlignment::BOTTOM_CENTER)
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                bottom: Val::Px(10.0),
                left: Val::Px(10.0),
                ..default()
            },
            ..default()
        }),
        PlayerLife,
    ));

    spawn_player(&mut commands, &asset_server);

    // Spawn Stars
    spawn_stars(commands, asset_server);
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_startup_system(start_background_audio)
            .add_startup_system(configure_visuals)
            .add_system(player_movement)
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(10.0))
                    .with_system(spawn_enemy_group),
            )
            .add_system(enemy_movement)
            .add_system(update_game_time)
            .add_system(update_fps)
            .add_system(update_player_health)
            .add_system(player_contact)
            .add_system(handle_game_over);
    }
}
