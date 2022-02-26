use std::f32::consts::PI;
use std::time::Duration;

use bevy::prelude::*;
use bevy_asset_loader::AssetLoader;
use heron::prelude::*;
use ordered_float::OrderedFloat;
use rand::Rng;
use wasm_bindgen::prelude::*;

use self::game_assets::*;
use self::game_collisions::*;

mod game_assets;
mod game_collisions;

const WHITE_COLOR: Color = Color::rgb(0.922, 0.922, 0.922);
const BLUE_COLOR: Color = Color::rgb(0.706, 0.706, 1.);

const PADDLE_SPEED: f32 = 10.0;
const BALL_TOUCH_PADDLE_SPEED_UP: f32 = 0.1;
const BALL_TOUCH_EDGE_SPEED_UP: f32 = 0.05;
const BALL_SCORE_DAMAGE: usize = 10;

// For wasm-pack to be happy...
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn wasm_main() {
    init();
}

pub fn init() {
    let mut app = App::new();
    AssetLoader::new(States::AssetLoading)
        .with_collection::<GameAssets>()
        .with_collection::<BallAssets>()
        .with_collection::<CardAssets>()
        .continue_to_state(States::InitGameField)
        .build(&mut app);

    app.add_event::<GameCollisionEvent>()
        .add_event::<SpawnBonusEvent>()
        .add_state(States::AssetLoading)
        .insert_resource(ClearColor(Color::rgb(0.239, 0.239, 0.239)))
        .insert_resource(GameScore::default())
        .insert_resource(BonusesTimers { dummy: Timer::new(Duration::from_secs(5), false) })
        .add_plugins(DefaultPlugins)
        .add_plugin(PhysicsPlugin::default())
        .insert_resource(Gravity::from(Vec3::ZERO))
        .add_startup_system(camera_setup)
        .add_system_set(
            SystemSet::on_enter(States::InitGameField)
                .with_system(spawn_paddles)
                .with_system(spawn_goals)
                .with_system(spawn_edges)
                .with_system(spawn_field_lines)
                .with_system(ready_to_play),
        )
        .add_system_set(
            SystemSet::on_enter(States::WaitingPlayer)
                .with_system(spawn_static_ball)
                .with_system(reset_bonuses)
                .with_system(reset_bonuses_timers)
                .with_system(reset_paddles_velocity),
        )
        .add_system_set(SystemSet::on_update(States::WaitingPlayer).with_system(launch_ball))
        .add_system_set(
            SystemSet::on_update(States::InGame)
                .with_system(produce_game_collision_events)
                .with_system(move_player_paddle)
                .with_system(move_computer_paddle)
                .with_system(speed_up_balls_with_touched_paddles)
                .with_system(speed_up_balls_with_touched_edges)
                .with_system(track_scoring_balls)
                .with_system(track_balls_touching_paddles)
                .with_system(tick_bonuses_timers)
                .with_system(remove_stalled_balls)
                .with_system(spawn_bonuses)
                .with_system(track_taken_bonuses)
                .with_system(regame_when_no_balls),
        )
        .run();
}

fn camera_setup(mut commands: Commands) {
    let mut camera_bundle = OrthographicCameraBundle::new_2d();
    camera_bundle.orthographic_projection.scale = 1. / 50.;
    commands.spawn_bundle(camera_bundle);
}

fn spawn_paddles(mut commands: Commands) {
    // Player paddle (on the right)
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: BLUE_COLOR,
                custom_size: Some(Vec2::new(0.5, 5.)),
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(11., 0., 0.)),
            ..Default::default()
        })
        .insert(Velocity::default())
        .insert(RigidBody::KinematicVelocityBased)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(0.25, 2.5, 0.),
            border_radius: None,
        })
        .insert(RotationConstraints::lock())
        .insert(PhysicMaterial {
            restitution: PhysicMaterial::PERFECTLY_ELASTIC_RESTITUTION,
            ..Default::default()
        })
        .insert(
            CollisionLayers::none()
                .with_group(GamePhysicsLayer::Paddle)
                .with_masks(&[GamePhysicsLayer::Ball, GamePhysicsLayer::Edge]),
        )
        .insert(PlayerPaddle);

    // Computer paddle (on the left)
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: WHITE_COLOR,
                custom_size: Some(Vec2::new(0.5, 5.)),
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(-11., 0., 0.)),
            ..Default::default()
        })
        .insert(Velocity::default())
        .insert(RigidBody::KinematicVelocityBased)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(0.25, 2.5, 0.),
            border_radius: None,
        })
        .insert(RotationConstraints::lock())
        .insert(PhysicMaterial {
            restitution: PhysicMaterial::PERFECTLY_ELASTIC_RESTITUTION,
            ..Default::default()
        })
        .insert(
            CollisionLayers::none()
                .with_group(GamePhysicsLayer::Paddle)
                .with_masks(&[GamePhysicsLayer::Ball, GamePhysicsLayer::Edge]),
        )
        .insert(ComputerPaddle);
}

fn spawn_goals(mut commands: Commands) {
    // Player goal (on the right)
    commands
        .spawn()
        .insert(Transform::from_translation(Vec3::new(12.25, 0., 0.)))
        .insert(GlobalTransform::default())
        .insert(RigidBody::Sensor)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(1.0, 10., 0.),
            border_radius: None,
        })
        .insert(RotationConstraints::lock())
        .insert(
            CollisionLayers::none()
                .with_group(GamePhysicsLayer::Goal)
                .with_mask(GamePhysicsLayer::Ball),
        )
        .insert(PlayerGoal);

    // Computer goal (on the left)
    commands
        .spawn()
        .insert(Transform::from_translation(Vec3::new(-12.25, 0., 0.)))
        .insert(GlobalTransform::default())
        .insert(RigidBody::Sensor)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(1.0, 10., 0.),
            border_radius: None,
        })
        .insert(RotationConstraints::lock())
        .insert(
            CollisionLayers::none()
                .with_group(GamePhysicsLayer::Goal)
                .with_mask(GamePhysicsLayer::Ball),
        )
        .insert(ComputerGoal);
}

fn spawn_edges(mut commands: Commands) {
    // Top edge
    commands
        .spawn()
        .insert(Transform::from_translation(Vec3::new(0., 7., 0.)))
        .insert(GlobalTransform::default())
        .insert(RigidBody::Static)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(14., 1., 0.),
            border_radius: None,
        })
        .insert(PhysicMaterial {
            restitution: PhysicMaterial::PERFECTLY_ELASTIC_RESTITUTION,
            ..Default::default()
        })
        .insert(
            CollisionLayers::none()
                .with_group(GamePhysicsLayer::Edge)
                .with_masks(&[GamePhysicsLayer::Ball, GamePhysicsLayer::Paddle]),
        );

    // Bottom edge
    commands
        .spawn()
        .insert(Transform::from_translation(Vec3::new(0., -7., 0.)))
        .insert(GlobalTransform::default())
        .insert(RigidBody::Static)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(14., 1., 0.),
            border_radius: None,
        })
        .insert(PhysicMaterial {
            restitution: PhysicMaterial::PERFECTLY_ELASTIC_RESTITUTION,
            ..Default::default()
        })
        .insert(
            CollisionLayers::none()
                .with_group(GamePhysicsLayer::Edge)
                .with_masks(&[GamePhysicsLayer::Ball, GamePhysicsLayer::Paddle]),
        );
}

fn spawn_field_lines(mut commands: Commands) {
    // Top horizontal line
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: WHITE_COLOR,
            custom_size: Some(Vec2::new(22.5, 0.1)),
            ..Default::default()
        },
        transform: Transform::from_translation(Vec3::new(0., 6., 0.)),
        ..Default::default()
    });

    // Bottom horizontal line
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: WHITE_COLOR,
            custom_size: Some(Vec2::new(22.5, 0.1)),
            ..Default::default()
        },
        transform: Transform::from_translation(Vec3::new(0., -6., 0.)),
        ..Default::default()
    });

    // Top of the middle vertical line
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: WHITE_COLOR,
            custom_size: Some(Vec2::new(0.1, 5.)),
            ..Default::default()
        },
        transform: Transform::from_translation(Vec3::new(0., 3.5, 0.)),
        ..Default::default()
    });

    // Bottom of the middle vertical line
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: WHITE_COLOR,
            custom_size: Some(Vec2::new(0.1, 5.)),
            ..Default::default()
        },
        transform: Transform::from_translation(Vec3::new(0., -3.5, 0.)),
        ..Default::default()
    });
}

// TODO remove that system
fn ready_to_play(mut state: ResMut<State<States>>) {
    state.set(States::WaitingPlayer).unwrap()
}

fn spawn_static_ball(mut commands: Commands, assets: Res<BallAssets>) {
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: assets.texture_atlas.clone(),
            sprite: TextureAtlasSprite {
                index: 0,
                custom_size: Some(Vec2::new(0.5, 0.5)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Acceleration::default())
        .insert(Velocity::default())
        .insert(RigidBody::Dynamic)
        .insert(CollisionShape::Sphere { radius: 0.25 })
        .insert(PhysicMaterial {
            restitution: PhysicMaterial::PERFECTLY_ELASTIC_RESTITUTION,
            ..Default::default()
        })
        .insert(CollisionLayers::none().with_group(GamePhysicsLayer::Ball).with_masks(&[
            GamePhysicsLayer::Paddle,
            GamePhysicsLayer::Goal,
            GamePhysicsLayer::Edge,
            GamePhysicsLayer::Bonus,
        ]))
        .insert(Ball::default());
}

fn reset_bonuses_timers(mut bonuses_timers: ResMut<BonusesTimers>) {
    bonuses_timers.dummy.reset();
}

fn tick_bonuses_timers(
    time: Res<Time>,
    mut bonuses_timers: ResMut<BonusesTimers>,
    mut spawn_bonus_event: EventWriter<SpawnBonusEvent>,
) {
    if bonuses_timers.dummy.tick(time.delta()).just_finished() {
        spawn_bonus_event.send(SpawnBonusEvent(Bonus::Dummy));
    }
}

fn reset_bonuses(mut command: Commands, bonuses_query: Query<Entity, With<Bonus>>) {
    for entity in bonuses_query.iter() {
        command.entity(entity).despawn_recursive();
    }
}

// TODO we must also reset the translation, but we declared it as a
//      KinematicVelocityBased entity so rapier ignores when we touch the position.
fn reset_paddles_velocity(
    mut paddles_query: Query<&mut Velocity, Or<(With<PlayerPaddle>, With<ComputerPaddle>)>>,
) {
    for mut velocity in paddles_query.iter_mut() {
        *velocity = Default::default();
    }
}

fn launch_ball(
    mut keys: ResMut<Input<KeyCode>>,
    mut state: ResMut<State<States>>,
    mut balls_query: Query<&mut Velocity, With<Ball>>,
) {
    if keys.clear_just_released(KeyCode::Space) {
        state.set(States::InGame).unwrap();
        let mut rng = rand::thread_rng();
        for mut velocity in balls_query.iter_mut() {
            let radian = rng.gen_range(0.0..PI) - PI / 2.;
            let x = radian.cos() * 3.0;
            let y = radian.sin() * 3.0;
            velocity.linear = if rng.gen() { Vec3::new(x, y, 0.0) } else { Vec3::new(-x, y, 0.0) };
        }
    }
}

fn move_player_paddle(
    keys: Res<Input<KeyCode>>,
    mut player_paddle_query: Query<&mut Velocity, With<PlayerPaddle>>,
) {
    for mut velocity in player_paddle_query.iter_mut() {
        let y = if keys.any_pressed([KeyCode::Up, KeyCode::W]) {
            1.
        } else if keys.any_pressed([KeyCode::Down, KeyCode::S]) {
            -1.
        } else {
            0.
        };

        velocity.linear = Vec2::new(0., y).extend(0.) * PADDLE_SPEED;
    }
}

fn move_computer_paddle(
    mut computer_paddle_query: Query<(&mut Velocity, &GlobalTransform), With<ComputerPaddle>>,
    balls_query: Query<&GlobalTransform, With<Ball>>,
) {
    for (mut velocity, global_transform) in computer_paddle_query.iter_mut() {
        let position = global_transform.translation;
        if let Some(nearest_ball_transform) = balls_query
            .iter()
            .min_by_key(|t| OrderedFloat(t.translation.distance_squared(position)))
        {
            let distance_y = nearest_ball_transform.translation[1] - position[1];
            let speed = if distance_y < 1.0 && distance_y > -1.0 {
                0.0
            } else if distance_y < 0. {
                -PADDLE_SPEED
            } else {
                PADDLE_SPEED
            };
            velocity.linear = Vec3::new(0., speed, 0.);
        }
    }
}

fn speed_up_balls_with_touched_paddles(
    mut collision_events: EventReader<GameCollisionEvent>,
    mut balls_query: Query<(&mut Velocity, &mut Ball)>,
) {
    use GameCollisionEvent::*;

    for event in collision_events.iter() {
        if let BallAndPaddle { status: CollisionStatus::Stopped, ball, .. } = event {
            if let Ok((mut velocity, mut ball)) = balls_query.get_mut(*ball) {
                ball.touched_paddles += 1;
                velocity.linear *= 1. + BALL_TOUCH_PADDLE_SPEED_UP;
            }
        }
    }
}

fn speed_up_balls_with_touched_edges(
    mut collision_events: EventReader<GameCollisionEvent>,
    mut balls_query: Query<(&mut Velocity, &mut Ball)>,
) {
    use GameCollisionEvent::*;

    for event in collision_events.iter() {
        if let BallAndEdge { status: CollisionStatus::Stopped, ball, .. } = event {
            if let Ok((mut velocity, mut ball)) = balls_query.get_mut(*ball) {
                ball.touched_paddles += 1;
                velocity.linear *= 1. + BALL_TOUCH_EDGE_SPEED_UP;
            }
        }
    }
}

fn track_scoring_balls(
    mut commands: Commands,
    mut collision_events: EventReader<GameCollisionEvent>,
    mut score: ResMut<GameScore>,
    goals_query: Query<(Option<&PlayerGoal>, Option<&ComputerGoal>)>,
) {
    use GameCollisionEvent::*;

    for event in collision_events.iter() {
        if let BallAndGoal { status: CollisionStatus::Started, ball, goal } = event {
            if let Ok((player, computer)) = goals_query.get(*goal) {
                if player.is_some() {
                    score.player_health = score.player_health.saturating_sub(BALL_SCORE_DAMAGE);
                } else if computer.is_some() {
                    score.computer_health = score.computer_health.saturating_sub(BALL_SCORE_DAMAGE);
                }
                commands.entity(*ball).despawn_recursive();
            }
        }
    }
}

fn track_balls_touching_paddles(
    mut collision_events: EventReader<GameCollisionEvent>,
    mut balls_query: Query<&mut Ball>,
) {
    use GameCollisionEvent::*;

    for event in collision_events.iter() {
        if let BallAndPaddle { status: CollisionStatus::Stopped, ball, paddle } = event {
            if let Ok(mut ball) = balls_query.get_mut(*ball) {
                ball.last_touched_paddle = Some(*paddle);
            }
        }
    }
}

fn regame_when_no_balls(mut state: ResMut<State<States>>, balls_query: Query<(), With<Ball>>) {
    if balls_query.is_empty() {
        state.set(States::WaitingPlayer).unwrap();
    }
}

fn remove_stalled_balls(
    mut commands: Commands,
    balls_query: Query<(Entity, &Velocity), With<Ball>>,
) {
    for (entity, velocity) in balls_query.iter() {
        if velocity.linear[1].abs() < 0.1 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn spawn_bonuses(
    mut commands: Commands,
    mut spawn_bonus_event: EventReader<SpawnBonusEvent>,
    assets: Res<CardAssets>,
) {
    for SpawnBonusEvent(bonus) in spawn_bonus_event.iter() {
        match bonus {
            Bonus::Dummy => {
                commands
                    .spawn_bundle(SpriteSheetBundle {
                        texture_atlas: assets.texture_atlas.clone(),
                        sprite: TextureAtlasSprite {
                            index: 0,
                            custom_size: Some(Vec2::new(0.75, 1.0)),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(Velocity::default())
                    .insert(RigidBody::Sensor)
                    .insert(CollisionShape::Cuboid {
                        half_extends: Vec3::new(0.25, 0.625, 0.),
                        border_radius: None,
                    })
                    .insert(
                        CollisionLayers::none()
                            .with_group(GamePhysicsLayer::Bonus)
                            .with_masks(&[GamePhysicsLayer::Ball, GamePhysicsLayer::Edge]),
                    )
                    .insert(Bonus::Dummy);
            }
        }
    }
}

fn track_taken_bonuses(
    mut commands: Commands,
    mut collision_events: EventReader<GameCollisionEvent>,
    mut score: ResMut<GameScore>,
    balls_query: Query<&Ball>,
    bonuses_query: Query<&Bonus>,
    paddles_query: Query<(Option<&PlayerPaddle>, Option<&ComputerPaddle>)>,
) {
    use GameCollisionEvent::*;

    for event in collision_events.iter() {
        if let BallAndBonus { status: CollisionStatus::Started, ball, bonus } = event {
            if let Ok(ball) = balls_query.get(*ball) {
                if let Some(paddle) = ball.last_touched_paddle {
                    if let Ok((player, computer)) = paddles_query.get(paddle) {
                        if let Ok(bonus) = bonuses_query.get(*bonus) {
                            if player.is_some() {
                                score.player_bonuses.push(*bonus);
                            } else if computer.is_some() {
                                score.computer_bonuses.push(*bonus);
                            }
                        }
                        commands.entity(*bonus).despawn_recursive();
                    }
                }
            }
        }
    }
}

struct GameScore {
    computer_health: usize,
    computer_score: usize,
    computer_bonuses: Vec<Bonus>,

    player_health: usize,
    player_score: usize,
    player_bonuses: Vec<Bonus>,
}

impl Default for GameScore {
    fn default() -> GameScore {
        GameScore {
            computer_health: 100,
            computer_score: 0,
            computer_bonuses: Vec::new(),
            player_health: 100,
            player_score: 0,
            player_bonuses: Vec::new(),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum States {
    AssetLoading,
    InitGameField,
    WaitingPlayer,
    InGame,
}

#[derive(Component)]
struct PlayerPaddle;

#[derive(Component)]
struct PlayerGoal;

#[derive(Component)]
struct ComputerPaddle;

#[derive(Component)]
struct ComputerGoal;

#[derive(Default, Component)]
struct Ball {
    touched_paddles: usize,
    last_touched_paddle: Option<Entity>,
}

#[derive(Component)]
struct BonusesTimers {
    dummy: Timer,
}

struct SpawnBonusEvent(Bonus);

#[derive(Component, Clone, Copy)]
enum Bonus {
    Dummy,
}
