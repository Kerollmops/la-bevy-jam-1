use std::f32::consts::PI;
use std::time::Duration;

use benimator::*;
use bevy::prelude::*;
use bevy_asset_loader::AssetLoader;
use bevy_kira_audio::{Audio, AudioPlugin};
use heron::prelude::*;
use heron::rapier_plugin::convert::IntoRapier;
use heron::rapier_plugin::rapier2d::dynamics::RigidBodySet;
use heron::rapier_plugin::RigidBodyHandle;
use ordered_float::OrderedFloat;
use rand::Rng;
use wasm_bindgen::prelude::*;

use self::assets::*;
use self::game_collisions::*;
use self::init::*;

mod assets;
mod game_collisions;
mod init;

const WHITE_COLOR: Color = Color::rgb(0.922, 0.922, 0.922);
const BLUE_COLOR: Color = Color::rgb(0.706, 0.706, 1.);
const RED_COLOR: Color = Color::rgb(1., 0.706, 0.706);

const PADDLE_SPEED: f32 = 10.0;
const BALL_SPEED: f32 = 10.0;
const BALL_TOUCH_PADDLE_SPEED_UP: f32 = 0.025;
const BALL_TOUCH_EDGE_SPEED_UP: f32 = 0.0125;
const PADDLE_ROTATION: f32 = PI / 15.;

// For wasm-pack to be happy...
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn wasm_main() {
    init();
}

pub fn init() {
    let mut app = App::new();
    AssetLoader::new(States::AssetLoading)
        .with_collection::<BallAssets>()
        .with_collection::<BonusesAssets>()
        .with_collection::<LifebarAssets>()
        .with_collection::<SpacebarAssets>()
        .with_collection::<AudioAssets>()
        .continue_to_state(States::InitGame)
        .build(&mut app);

    app.add_event::<GameCollisionEvent>()
        .add_event::<SpawnBonusEvent>()
        .add_event::<TakenBonusEvent>()
        .add_state(States::AssetLoading)
        .insert_resource(ClearColor(Color::rgb(0.239, 0.239, 0.239)))
        .insert_resource(GameScore::default())
        .insert_resource(BonusesTimers(vec![
            (Timer::new(Duration::from_secs(13), true), BonusType::SplitBall),
            (Timer::new(Duration::from_secs(15), false), BonusType::BallSpeedInArea),
            (Timer::new(Duration::from_secs(20), false), BonusType::ShrinkPaddleSize),
            (Timer::new(Duration::from_secs(25), false), BonusType::IncreasePaddleSize),
            (Timer::new(Duration::from_secs(25 + 1), false), BonusType::IncreasePaddleSize),
            (Timer::new(Duration::from_secs(25 + 2), false), BonusType::IncreasePaddleSize),
            (Timer::new(Duration::from_secs(25 + 3), false), BonusType::IncreasePaddleSize),
            (Timer::new(Duration::from_secs(30), false), BonusType::BallsVerticalGravity),
        ]))
        .add_plugins(DefaultPlugins)
        .add_plugin(PhysicsPlugin::default())
        .add_plugin(AnimationPlugin::default())
        .add_plugin(AudioPlugin::default())
        .insert_resource(Gravity::from(Vec3::ZERO))
        .add_startup_system(camera_setup)
        .add_system_set(
            SystemSet::on_enter(States::InitGame)
                .with_system(generate_animations)
                .with_system(spawn_paddles)
                .with_system(spawn_goals)
                .with_system(spawn_edges)
                .with_system(spawn_sides)
                .with_system(spawn_field_lines)
                .with_system(spawn_lifebars)
                .with_system(ready_to_wait_player),
        )
        .add_system_set(
            SystemSet::on_enter(States::WaitingPlayer)
                .with_system(display_spacebar_animation)
                .with_system(spawn_static_ball)
                .with_system(enable_spawned_balls_ccd)
                .with_system(reset_bonuses)
                .with_system(reset_owned_bonuses)
                .with_system(reset_bonuses_timers)
                .with_system(reset_paddles_velocity)
                .with_system(reset_paddle_transform)
                .with_system(reset_paddle_sizes),
        )
        .add_system_set(SystemSet::on_update(States::WaitingPlayer).with_system(launch_ball))
        .add_system_set(SystemSet::on_enter(States::InGame).with_system(hide_spacebar_animation))
        .add_system_set(
            SystemSet::on_update(States::InGame)
                .with_system(produce_game_collision_events)
                .with_system(enable_spawned_balls_ccd)
                .with_system(move_player_paddle)
                .with_system(move_computer_paddle)
                .with_system(tilt_paddle)
                .with_system(speed_up_balls_with_touched_paddles)
                .with_system(speed_up_balls_with_touched_edges)
                .with_system(track_scoring_balls)
                .with_system(track_balls_touching_paddles)
                .with_system(track_balls_entering_side)
                .with_system(blip_on_ball_collisions)
                .with_system(tick_bonuses_timers)
                .with_system(spawn_bonuses)
                .with_system(manage_taken_bonuses)
                .with_system(store_taken_bonuses_in_score)
                .with_system(manage_split_ball_bonus)
                .with_system(manage_ball_speed_on_area_bonus)
                .with_system(manage_balls_vertical_gravity_bonus)
                .with_system(manage_shrink_paddle_size_bonus)
                .with_system(manage_increase_paddle_size_bonus)
                .with_system(regame_when_no_balls),
        )
        .run();
}

fn generate_animations(
    mut spacebar_assets: ResMut<SpacebarAssets>,
    mut animations: ResMut<Assets<SpriteSheetAnimation>>,
) {
    spacebar_assets.loop_animation = animations
        .add(SpriteSheetAnimation::from_range(0..=2, Duration::from_millis(180)).repeat());
}

fn display_spacebar_animation(mut commands: Commands, spacebar_assets: ResMut<SpacebarAssets>) {
    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                custom_size: Some(Vec2::new(4., 1.)),
                ..Default::default()
            },
            texture_atlas: spacebar_assets.texture_atlas.clone(),
            transform: Transform::from_translation(Vec3::new(0., -1.5, 0.)),
            ..Default::default()
        })
        .insert(spacebar_assets.loop_animation.clone())
        .insert(Play)
        .insert(SpacebarAnimation);
}

fn hide_spacebar_animation(
    mut commands: Commands,
    spacebar_query: Query<Entity, With<SpacebarAnimation>>,
    audio_assets: Res<AudioAssets>,
    audio: Res<Audio>,
) {
    for entity in spacebar_query.iter() {
        commands.entity(entity).despawn();
    }

    audio.play(audio_assets.whistle.clone());
}

// TODO remove that system if possible
fn ready_to_wait_player(mut state: ResMut<State<States>>) {
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
            GamePhysicsLayer::Side,
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
    bonuses_timers.0.iter_mut().for_each(|(timer, _)| timer.reset());
}

fn reset_paddle_sizes(mut paddles_query: Query<(&mut CollisionShape, &mut Sprite, &Paddle)>) {
    for (col, mut sprite, paddle) in paddles_query.iter_mut() {
        let default_size = match paddle {
            Paddle::Player => PLAYER_PADDLE_HEIGHT,
            Paddle::Computer => COMPUTER_PADDLE_HEIGHT,
        };

        if let CollisionShape::Cuboid { mut half_extends, .. } = col.into_inner() {
            half_extends[1] = default_size / 2.;
        }

        if let Some(size) = sprite.custom_size.as_mut() {
            size[1] = default_size;
        }
    }
}

fn reset_paddle_transform(mut paddles_query: Query<&mut Transform, With<Paddle>>) {
    for mut transform in paddles_query.iter_mut() {
        transform.translation.y = 0.;
        transform.rotation = Quat::IDENTITY;
    }
}

fn tick_bonuses_timers(
    time: Res<Time>,
    mut bonuses_timers: ResMut<BonusesTimers>,
    mut spawn_bonus_event: EventWriter<SpawnBonusEvent>,
) {
    for (timer, bonus) in bonuses_timers.0.iter_mut() {
        if timer.tick(time.delta()).just_finished() {
            spawn_bonus_event.send(SpawnBonusEvent(*bonus));
        }
    }
}

fn reset_bonuses(mut command: Commands, bonuses_query: Query<Entity, With<BonusType>>) {
    for entity in bonuses_query.iter() {
        command.entity(entity).despawn_recursive();
    }
}

fn reset_owned_bonuses(mut game_score: ResMut<GameScore>) {
    game_score.player_bonuses.clear();
    game_score.computer_bonuses.clear();
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
            let radian = rng.gen_range(0.0..PI / 2.) + 3. * PI / 4.;
            let x = radian.cos() * BALL_SPEED;
            let y = radian.sin() * BALL_SPEED;
            velocity.linear = if rng.gen() { Vec3::new(x, y, 0.0) } else { Vec3::new(-x, y, 0.0) };
        }
    }
}

fn move_player_paddle(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mut paddle_query: Query<(&mut Transform, &mut Velocity, &Paddle)>,
) {
    for (mut transform, mut velocity, paddle) in paddle_query.iter_mut() {
        if let Paddle::Player = paddle {
            let direction = if keys.any_pressed([KeyCode::Up, KeyCode::W]) {
                1.
            } else if keys.any_pressed([KeyCode::Down, KeyCode::S]) {
                -1.
            } else {
                0.
            };

            velocity.linear.y = direction * PADDLE_SPEED;
            transform.translation.y += time.delta_seconds() * direction * PADDLE_SPEED;
            transform.translation.y = transform.translation.y.clamp(-8., 8.);
        }
    }
}

fn move_computer_paddle(
    time: Res<Time>,
    mut paddle_query: Query<(&mut Transform, &GlobalTransform, &Paddle)>,
    balls_query: Query<&GlobalTransform, With<Ball>>,
) {
    for (mut transform, global_transform, paddle) in paddle_query.iter_mut() {
        if let Paddle::Computer = paddle {
            let position = global_transform.translation;
            if let Some(nearest_ball_transform) = balls_query
                .iter()
                .min_by_key(|t| OrderedFloat((t.translation[0] - position[0]).abs()))
            {
                let distance_y = nearest_ball_transform.translation[1] - position[1];
                let speed = if distance_y < 1.0 && distance_y > -1.0 {
                    0.0
                } else if distance_y < 0. {
                    -PADDLE_SPEED
                } else {
                    PADDLE_SPEED
                };

                transform.translation.y += time.delta_seconds() * speed;
                transform.translation.y = transform.translation.y.clamp(-8., 8.);
            }
        }
    }
}

fn tilt_paddle(mut paddle_query: Query<(&mut Transform, &Velocity), With<Paddle>>) {
    for (mut transform, velocity) in paddle_query.iter_mut() {
        if velocity.linear.y > 0.1 {
            transform.rotation = Quat::from_rotation_z(2. * PI - PADDLE_ROTATION);
        } else if velocity.linear.y < -0.1 {
            transform.rotation = Quat::from_rotation_z(PADDLE_ROTATION);
        } else {
            transform.rotation = Quat::IDENTITY;
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
    mut balls_query: Query<&mut Velocity, With<Ball>>,
) {
    use GameCollisionEvent::*;

    for event in collision_events.iter() {
        if let BallAndEdge { status: CollisionStatus::Stopped, ball, .. } = event {
            if let Ok(mut velocity) = balls_query.get_mut(*ball) {
                velocity.linear *= 1. + BALL_TOUCH_EDGE_SPEED_UP;
            }
        }
    }
}

fn track_scoring_balls(
    mut commands: Commands,
    mut collision_events: EventReader<GameCollisionEvent>,
    mut lifebar_query: Query<(&mut TextureAtlasSprite, &Lifebar)>,
    mut score: ResMut<GameScore>,
    goals_query: Query<&Goal>,
    audio_assets: Res<AudioAssets>,
    audio: Res<Audio>,
) {
    use GameCollisionEvent::*;

    for event in collision_events.iter() {
        if let BallAndGoal { status: CollisionStatus::Started, ball, goal } = event {
            if let Ok(goal) = goals_query.get(*goal) {
                match goal {
                    Goal::Player => {
                        score.player_health = score.player_health.saturating_sub(1);
                        for (mut texture_atlas_sprite, lifebar) in lifebar_query.iter_mut() {
                            if let Lifebar::Player = lifebar {
                                texture_atlas_sprite.index = score.player_health;
                            }
                        }
                    }
                    Goal::Computer => {
                        score.computer_health = score.computer_health.saturating_sub(1);
                        for (mut texture_atlas_sprite, lifebar) in lifebar_query.iter_mut() {
                            if let Lifebar::Computer = lifebar {
                                texture_atlas_sprite.index = score.computer_health;
                            }
                        }
                    }
                }

                audio.play(audio_assets.goal.clone());
                commands.entity(*ball).despawn_recursive();
            }
        }
    }
}

fn track_balls_touching_paddles(
    mut collision_events: EventReader<GameCollisionEvent>,
    mut balls_query: Query<(&mut TextureAtlasSprite, &mut Ball)>,
    paddles_query: Query<&Paddle>,
) {
    use GameCollisionEvent::*;

    for event in collision_events.iter() {
        if let BallAndPaddle { status: CollisionStatus::Stopped, ball, paddle } = event {
            if let Ok((mut texture_atlas_sprite, mut ball)) = balls_query.get_mut(*ball) {
                ball.last_touched_paddle = Some(*paddle);
                if let Ok(paddle) = paddles_query.get(*paddle) {
                    match paddle {
                        Paddle::Player => texture_atlas_sprite.color = BLUE_COLOR,
                        Paddle::Computer => texture_atlas_sprite.color = RED_COLOR,
                    }
                }
            }
        }
    }
}

fn track_balls_entering_side(
    mut collision_events: EventReader<GameCollisionEvent>,
    sides_query: Query<&Side>,
    mut balls_query: Query<&mut Ball>,
) {
    use CollisionStatus::*;
    use GameCollisionEvent::*;

    for event in collision_events.iter() {
        if let BallAndSide { status: Started, ball, side } = event {
            if let Ok(side) = sides_query.get(*side) {
                if let Ok(mut ball) = balls_query.get_mut(*ball) {
                    ball.current_side = Some(*side);
                }
            }
        }
    }
}

fn blip_on_ball_collisions(
    mut collision_events: EventReader<GameCollisionEvent>,
    audio_assets: Res<AudioAssets>,
    audio: Res<Audio>,
) {
    use GameCollisionEvent::*;

    for event in collision_events.iter() {
        // if matches!(event, BallAndEdge { .. }) {
        //     //audio.play(audio_assets.hit_0.clone());
        // }

        if matches!(event, BallAndPaddle { .. }) {
            audio.play(audio_assets.hit_1.clone());
        }
    }
}

fn regame_when_no_balls(mut state: ResMut<State<States>>, balls_query: Query<(), With<Ball>>) {
    if balls_query.is_empty() {
        state.set(States::WaitingPlayer).unwrap();
    }
}

fn spawn_bonuses(
    mut commands: Commands,
    mut spawn_bonus_event: EventReader<SpawnBonusEvent>,
    bonuses_assets: Res<BonusesAssets>,
    audio_assets: Res<AudioAssets>,
    audio: Res<Audio>,
) {
    let mut rng = rand::thread_rng();
    for SpawnBonusEvent(bonus) in spawn_bonus_event.iter() {
        let x = rng.gen_range(-10.0..10.0);
        let y = rng.gen_range(-5.5..5.5);

        let (texture_atlas, index) = match bonus {
            BonusType::SplitBall => (bonuses_assets.texture_atlas.clone(), 0),
            BonusType::BallSpeedInArea => (bonuses_assets.texture_atlas.clone(), 12),
            BonusType::BallsVerticalGravity => (bonuses_assets.texture_atlas.clone(), 4),
            BonusType::ShrinkPaddleSize => (bonuses_assets.paddle_texture_atlas.clone(), 1),
            BonusType::IncreasePaddleSize => (bonuses_assets.paddle_texture_atlas.clone(), 0),
        };

        let mut commands = commands.spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas,
            transform: Transform::from_translation(Vec3::new(x, y, 0.0)),
            sprite: TextureAtlasSprite {
                index,
                custom_size: Some(Vec2::new(0.75, 0.75)),
                ..Default::default()
            },
            ..Default::default()
        });
        commands
            .insert(Velocity::default())
            .insert(RigidBody::Sensor)
            .insert(CollisionShape::Cuboid {
                half_extends: Vec3::new(0.375, 0.375, 0.),
                border_radius: None,
            })
            .insert(
                CollisionLayers::none()
                    .with_group(GamePhysicsLayer::Bonus)
                    .with_masks(&[GamePhysicsLayer::Ball, GamePhysicsLayer::Edge]),
            );

        match bonus {
            BonusType::SplitBall => {
                commands.insert(BonusType::SplitBall);
            }
            BonusType::BallSpeedInArea => {
                commands.insert(BonusType::BallSpeedInArea);
            }
            BonusType::BallsVerticalGravity => {
                commands.insert(BonusType::BallsVerticalGravity);
            }
            BonusType::ShrinkPaddleSize => {
                commands.insert(BonusType::ShrinkPaddleSize);
            }
            BonusType::IncreasePaddleSize => {
                commands.insert(BonusType::IncreasePaddleSize);
            }
        }

        audio.play(audio_assets.powerup_spawn.clone());
    }
}

fn manage_taken_bonuses(
    mut commands: Commands,
    mut collision_events_reader: EventReader<GameCollisionEvent>,
    mut taken_bonus_writer: EventWriter<TakenBonusEvent>,
    balls_query: Query<&Ball>,
    bonuses_query: Query<&BonusType>,
    paddles_query: Query<&Paddle>,
    audio_assets: Res<AudioAssets>,
    audio: Res<Audio>,
) {
    use CollisionStatus::*;
    use GameCollisionEvent::*;

    for event in collision_events_reader.iter() {
        if let BallAndBonus { status: Started, ball: ball_entity, bonus: bonus_entity } = event {
            if let Ok(ball) = balls_query.get(*ball_entity) {
                if let Some(paddle_entity) = ball.last_touched_paddle {
                    if let Ok(&paddle) = paddles_query.get(paddle_entity) {
                        if let Ok(&bonus) = bonuses_query.get(*bonus_entity) {
                            let bonus = match bonus {
                                BonusType::SplitBall => TakenBonusEvent {
                                    bonus: Bonus::SplitBall { ball: *ball_entity },
                                    paddle,
                                },
                                BonusType::BallSpeedInArea => TakenBonusEvent {
                                    bonus: Bonus::BallSpeedInArea { benefiting_paddle: paddle },
                                    paddle,
                                },
                                BonusType::BallsVerticalGravity => TakenBonusEvent {
                                    bonus: Bonus::BallsVerticalGravity {
                                        benefiting_paddle: paddle,
                                    },
                                    paddle,
                                },
                                BonusType::ShrinkPaddleSize => TakenBonusEvent {
                                    bonus: Bonus::ShrinkPaddleSize {
                                        impacted_paddle: paddle.reverse(),
                                    },
                                    paddle,
                                },
                                BonusType::IncreasePaddleSize => TakenBonusEvent {
                                    bonus: Bonus::IncreasePaddleSize { benefiting_paddle: paddle },
                                    paddle,
                                },
                            };
                            taken_bonus_writer.send(bonus);
                            commands.entity(*bonus_entity).despawn_recursive();

                            audio.play(audio_assets.powerup_gain.clone());
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
    for TakenBonusEvent { bonus, .. } in taken_bonus_reader.iter() {
        if let Bonus::SplitBall { ball } = bonus {
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

fn manage_ball_speed_on_area_bonus(
    mut taken_bonus_reader: EventReader<TakenBonusEvent>,
    mut collision_events_reader: EventReader<GameCollisionEvent>,
    game_score: Res<GameScore>,
    mut balls_query: Query<(&mut Velocity, &Ball)>,
    side_query: Query<&Side>,
) {
    use Bonus::*;
    use CollisionStatus::*;

    fn change_velocity(
        velocity: &mut Velocity,
        side: Side,
        status: CollisionStatus,
        player_speedups: usize,
        computer_speedups: usize,
    ) {
        match (side, status) {
            (Side::Player, Started) => {
                velocity.linear *= 1. + 1.5 * computer_speedups as f32;
            }
            (Side::Player, Stopped) => {
                velocity.linear /= 1. + 1.5 * computer_speedups as f32;
            }
            (Side::Computer, Started) => {
                velocity.linear *= 1. + 1.5 * player_speedups as f32;
            }
            (Side::Computer, Stopped) => {
                velocity.linear /= 1. + 1.5 * player_speedups as f32;
            }
        }
    }

    let player_speedups =
        game_score.player_bonuses.iter().filter(|b| matches!(b, BallSpeedInArea { .. })).count();
    let computer_speedups =
        game_score.computer_bonuses.iter().filter(|b| matches!(b, BallSpeedInArea { .. })).count();

    // We speed-up or slow-down the ball at the moment we take it
    for event in taken_bonus_reader.iter() {
        if let TakenBonusEvent { bonus: BallSpeedInArea { benefiting_paddle }, .. } = event {
            for (mut velocity, ball) in balls_query.iter_mut() {
                if let Some(side) = ball.current_side {
                    let status = match (benefiting_paddle, side) {
                        (Paddle::Player, Side::Player) => Stopped,
                        (Paddle::Computer, Side::Computer) => Stopped,
                        (Paddle::Player, Side::Computer) => Started,
                        (Paddle::Computer, Side::Player) => Started,
                    };

                    change_velocity(
                        &mut velocity,
                        side,
                        status,
                        player_speedups,
                        computer_speedups,
                    );
                }
            }
        }
    }

    if player_speedups > 0 || computer_speedups > 0 {
        for event in collision_events_reader.iter() {
            if let GameCollisionEvent::BallAndSide { status, ball, side } = event {
                if let Ok(side) = side_query.get(*side) {
                    if let Ok((mut velocity, _ball)) = balls_query.get_mut(*ball) {
                        change_velocity(
                            &mut velocity,
                            *side,
                            *status,
                            player_speedups,
                            computer_speedups,
                        );
                    }
                }
            }
        }
    }
}

fn manage_balls_vertical_gravity_bonus(
    mut taken_bonus_reader: EventReader<TakenBonusEvent>,
    mut balls_query: Query<&mut Acceleration, With<Ball>>,
) {
    for TakenBonusEvent { bonus, .. } in taken_bonus_reader.iter() {
        if let Bonus::BallsVerticalGravity { benefiting_paddle } = bonus {
            for mut acceleration in balls_query.iter_mut() {
                acceleration.linear = match benefiting_paddle {
                    Paddle::Player => Vec3::new(-9.81, 0., 0.),
                    Paddle::Computer => Vec3::new(9.81, 0., 0.),
                };
            }
        }
    }
}

fn manage_shrink_paddle_size_bonus(
    mut taken_bonus_reader: EventReader<TakenBonusEvent>,
    mut paddles_query: Query<(&mut CollisionShape, &mut Sprite, &Paddle)>,
) {
    for TakenBonusEvent { bonus, .. } in taken_bonus_reader.iter() {
        if let Bonus::ShrinkPaddleSize { impacted_paddle } = bonus {
            for (col, mut sprite, paddle) in paddles_query.iter_mut() {
                if impacted_paddle == paddle {
                    if let CollisionShape::Cuboid { ref mut half_extends, .. } = col.into_inner() {
                        half_extends[1] = (half_extends[1] - 0.3).max(0.5);
                    }
                    if let Some(size) = sprite.custom_size.as_mut() {
                        size[1] = (size[1] - 0.6).max(1.);
                    }
                }
            }
        }
    }
}

fn manage_increase_paddle_size_bonus(
    mut taken_bonus_reader: EventReader<TakenBonusEvent>,
    mut paddles_query: Query<(&mut CollisionShape, &mut Sprite, &Paddle)>,
) {
    for TakenBonusEvent { bonus, .. } in taken_bonus_reader.iter() {
        if let Bonus::IncreasePaddleSize { benefiting_paddle } = bonus {
            for (col, mut sprite, paddle) in paddles_query.iter_mut() {
                if benefiting_paddle == paddle {
                    if let CollisionShape::Cuboid { ref mut half_extends, .. } = col.into_inner() {
                        half_extends[1] = (half_extends[1] + 0.3).min(5.);
                    }
                    if let Some(size) = sprite.custom_size.as_mut() {
                        size[1] = (size[1] + 0.6).min(10.);
                    }
                }
            }
        }
    }
}

struct GameScore {
    computer_health: usize, // from 1 to 16
    computer_score: usize,
    computer_bonuses: Vec<Bonus>,

    player_health: usize, // from 1 to 16
    player_score: usize,
    player_bonuses: Vec<Bonus>,
}

impl Default for GameScore {
    fn default() -> GameScore {
        GameScore {
            computer_health: 15,
            computer_score: 0,
            computer_bonuses: Vec::new(),
            player_health: 15,
            player_score: 0,
            player_bonuses: Vec::new(),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum States {
    AssetLoading,
    InitGame,
    WaitingPlayer,
    InGame,
}

#[derive(Component, Debug, Copy, Clone, PartialEq, Eq)]
enum Paddle {
    Player,
    Computer,
}

impl Paddle {
    fn reverse(&self) -> Paddle {
        match self {
            Paddle::Player => Paddle::Computer,
            Paddle::Computer => Paddle::Player,
        }
    }
}

#[derive(Component, Copy, Clone)]
enum Goal {
    Player,
    Computer,
}

#[derive(Component, Copy, Clone)]
enum Side {
    Player,
    Computer,
}

#[derive(Default, Component)]
struct Ball {
    touched_paddles: usize,
    last_touched_paddle: Option<Entity>,
    current_side: Option<Side>,
}

#[derive(Component)]
struct BonusesTimers(Vec<(Timer, BonusType)>);

struct SpawnBonusEvent(BonusType);

#[derive(Debug)]
struct TakenBonusEvent {
    bonus: Bonus,
    paddle: Paddle,
}

#[derive(Debug, Clone, Copy)]
enum Bonus {
    SplitBall { ball: Entity },
    BallSpeedInArea { benefiting_paddle: Paddle },
    BallsVerticalGravity { benefiting_paddle: Paddle },
    ShrinkPaddleSize { impacted_paddle: Paddle },
    IncreasePaddleSize { benefiting_paddle: Paddle },
}

#[derive(Component, Clone, Copy)]
enum BonusType {
    SplitBall,
    BallSpeedInArea,
    BallsVerticalGravity,
    ShrinkPaddleSize,
    IncreasePaddleSize,
}

#[derive(Component)]
enum Lifebar {
    Player,
    Computer,
}

#[derive(Component)]
struct SpacebarAnimation;
