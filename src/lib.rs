use std::f32::consts::PI;
use std::time::Duration;

use bevy::prelude::*;
use bevy_asset_loader::AssetLoader;
use heron::prelude::*;
use heron::rapier_plugin::convert::IntoRapier;
use heron::rapier_plugin::rapier2d::dynamics::RigidBodySet;
use heron::rapier_plugin::RigidBodyHandle;
use ordered_float::OrderedFloat;
use rand::Rng;
use wasm_bindgen::prelude::*;

use self::game_assets::*;
use self::game_collisions::*;
use self::init::*;

mod game_assets;
mod game_collisions;
mod init;

const WHITE_COLOR: Color = Color::rgb(0.922, 0.922, 0.922);
const BLUE_COLOR: Color = Color::rgb(0.706, 0.706, 1.);

const PADDLE_SPEED: f32 = 10.0;
const BALL_SPEED: f32 = 5.0;
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
        .add_event::<TakenBonusEvent>()
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
                .with_system(enable_spawned_balls_ccd)
                .with_system(reset_bonuses)
                .with_system(reset_bonuses_timers)
                .with_system(reset_paddles_velocity),
        )
        .add_system_set(SystemSet::on_update(States::WaitingPlayer).with_system(launch_ball))
        .add_system_set(
            SystemSet::on_update(States::InGame)
                .with_system(produce_game_collision_events)
                .with_system(enable_spawned_balls_ccd)
                .with_system(move_player_paddle)
                .with_system(move_computer_paddle)
                .with_system(speed_up_balls_with_touched_paddles)
                .with_system(speed_up_balls_with_touched_edges)
                .with_system(track_scoring_balls)
                .with_system(track_balls_touching_paddles)
                .with_system(tick_bonuses_timers)
                .with_system(remove_stalled_balls)
                .with_system(spawn_bonuses)
                .with_system(manage_taken_bonuses)
                .with_system(store_taken_bonuses_in_score)
                .with_system(manage_split_ball_bonus)
                .with_system(regame_when_no_balls),
        )
        .run();
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

/// Enable the CCD to the spawned balls, things that can go fast.
/// <https://rapier.rs/docs/user_guides/bevy_plugin/rigid_bodies/#continuous-collision-detection>
fn enable_spawned_balls_ccd(
    mut rigid_bodies: ResMut<RigidBodySet>,
    new_handles: Query<&RigidBodyHandle, (Added<RigidBodyHandle>, With<Ball>)>,
) {
    for handle in new_handles.iter() {
        if let Some(body) = rigid_bodies.get_mut(handle.into_rapier()) {
            body.enable_ccd(true);
        }
    }
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
        spawn_bonus_event.send(SpawnBonusEvent(BonusType::SplitBall));
    }
}

fn reset_bonuses(mut command: Commands, bonuses_query: Query<Entity, With<BonusType>>) {
    for entity in bonuses_query.iter() {
        command.entity(entity).despawn_recursive();
    }
}

// TODO we must also reset the translation, but we declared it as a
//      KinematicVelocityBased entity so rapier ignores when we touch the position.
fn reset_paddles_velocity(mut paddles_query: Query<&mut Velocity, With<Paddle>>) {
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
            let x = radian.cos() * BALL_SPEED;
            let y = radian.sin() * BALL_SPEED;
            velocity.linear = if rng.gen() { Vec3::new(x, y, 0.0) } else { Vec3::new(-x, y, 0.0) };
        }
    }
}

fn move_player_paddle(
    keys: Res<Input<KeyCode>>,
    mut paddle_query: Query<(&mut Velocity, &Paddle)>,
) {
    for (mut velocity, paddle) in paddle_query.iter_mut() {
        if let Paddle::Player = paddle {
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
}

fn move_computer_paddle(
    mut computer_paddle_query: Query<(&mut Velocity, &GlobalTransform, &Paddle)>,
    balls_query: Query<&GlobalTransform, With<Ball>>,
) {
    for (mut velocity, global_transform, paddle) in computer_paddle_query.iter_mut() {
        if let Paddle::Computer = paddle {
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
    goals_query: Query<&Goal>,
) {
    use GameCollisionEvent::*;

    for event in collision_events.iter() {
        if let BallAndGoal { status: CollisionStatus::Started, ball, goal } = event {
            if let Ok(goal) = goals_query.get(*goal) {
                match goal {
                    Goal::Player => {
                        score.player_health = score.player_health.saturating_sub(BALL_SCORE_DAMAGE);
                    }
                    Goal::Computer => {
                        score.computer_health =
                            score.computer_health.saturating_sub(BALL_SCORE_DAMAGE);
                    }
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
        if velocity.linear[1].abs() < 0.5 {
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
            BonusType::SplitBall => {
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
                    .insert(BonusType::SplitBall);
            }
        }
    }
}

fn manage_taken_bonuses(
    mut commands: Commands,
    mut collision_events_reader: EventReader<GameCollisionEvent>,
    mut taken_bonus_writer: EventWriter<TakenBonusEvent>,
    balls_query: Query<&Ball>,
    bonuses_query: Query<&BonusType>,
    paddles_query: Query<&Paddle>,
) {
    use CollisionStatus::*;
    use GameCollisionEvent::*;

    for event in collision_events_reader.iter() {
        if let BallAndBonus { status: Started, ball: ball_entity, bonus: bonus_entity } = event {
            if let Ok(ball) = balls_query.get(*ball_entity) {
                if let Some(paddle_entity) = ball.last_touched_paddle {
                    if let Ok(&paddle) = paddles_query.get(paddle_entity) {
                        if let Ok(&bonus) = bonuses_query.get(*bonus_entity) {
                            taken_bonus_writer.send(TakenBonusEvent {
                                bonus: Bonus::SplitBall { ball: *ball_entity },
                                paddle,
                            });
                            commands.entity(*bonus_entity).despawn_recursive();
                        }
                    }
                }
            }
        }
    }
}

fn store_taken_bonuses_in_score(
    mut score: ResMut<GameScore>,
    mut taken_bonus_reader: EventReader<TakenBonusEvent>,
) {
    for TakenBonusEvent { bonus, paddle } in taken_bonus_reader.iter() {
        match paddle {
            Paddle::Player => score.player_bonuses.push(*bonus),
            Paddle::Computer => score.computer_bonuses.push(*bonus),
        }
    }
}

fn manage_split_ball_bonus(
    mut commands: Commands,
    mut taken_bonus_reader: EventReader<TakenBonusEvent>,
    balls_query: Query<(&Transform, &Velocity), With<Ball>>,
    assets: Res<BallAssets>,
) {
    let mut rng = rand::thread_rng();
    for event in taken_bonus_reader.iter() {
        if let TakenBonusEvent { bonus: Bonus::SplitBall { ball }, paddle } = event {
            if let Ok((transform, velocity)) = balls_query.get(*ball) {
                // Rotate the velocity of the original ball by a random angle
                let angle = rng.gen_range(0.0..2.0 * PI);
                let x1 = velocity.linear[0];
                let y1 = velocity.linear[1];
                let x2 = angle.cos() * x1 - angle.sin() * y1;
                let y2 = angle.sin() * x1 + angle.cos() * y1;
                let velocity = Vec3::new(x2, y2, 0.);

                commands
                    .spawn_bundle(SpriteSheetBundle {
                        texture_atlas: assets.texture_atlas.clone(),
                        transform: *transform,
                        sprite: TextureAtlasSprite {
                            index: 0,
                            custom_size: Some(Vec2::new(0.5, 0.5)),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(Acceleration::default())
                    .insert(Velocity::from_linear(velocity))
                    .insert(RigidBody::Dynamic)
                    .insert(CollisionShape::Sphere { radius: 0.25 })
                    .insert(PhysicMaterial {
                        restitution: PhysicMaterial::PERFECTLY_ELASTIC_RESTITUTION,
                        ..Default::default()
                    })
                    .insert(CollisionLayers::none().with_group(GamePhysicsLayer::Ball).with_masks(
                        &[
                            GamePhysicsLayer::Paddle,
                            GamePhysicsLayer::Goal,
                            GamePhysicsLayer::Edge,
                            GamePhysicsLayer::Bonus,
                        ],
                    ))
                    .insert(Ball::default());
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

#[derive(Component, Copy, Clone)]
enum Paddle {
    Player,
    Computer,
}

#[derive(Component, Copy, Clone)]
enum Goal {
    Player,
    Computer,
}

#[derive(Default, Component)]
struct Ball {
    touched_paddles: usize,
    last_touched_paddle: Option<Entity>,
}

#[derive(Component)]
struct BonusesTimers {
    dummy: Timer,
}

struct SpawnBonusEvent(BonusType);

struct TakenBonusEvent {
    bonus: Bonus,
    paddle: Paddle,
}

#[derive(Clone, Copy)]
enum Bonus {
    SplitBall { ball: Entity },
}

#[derive(Component, Clone, Copy)]
enum BonusType {
    SplitBall,
}
