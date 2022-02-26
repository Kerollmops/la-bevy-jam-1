use bevy::prelude::shape::Icosphere;
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy_asset_loader::AssetLoader;
use heron::prelude::*;
use wasm_bindgen::prelude::*;

use self::game_assets::*;

mod game_assets;

const PADDLE_SPEED: f32 = 10.0;
const WHITE_COLOR: Color = Color::rgb(0.922, 0.922, 0.922);
const BLUE_COLOR: Color = Color::rgb(0.706, 0.706, 1.);

pub fn init() {
    let mut app = App::new();
    AssetLoader::new(States::AssetLoading)
        .with_collection::<GameAssets>()
        .continue_to_state(States::WaitingPlayer)
        .build(&mut app);

    app.add_state(States::AssetLoading)
        .insert_resource(ClearColor(Color::rgb(0.239, 0.239, 0.239)))
        .add_plugins(DefaultPlugins)
        .add_plugin(PhysicsPlugin::default())
        .insert_resource(Gravity::from(Vec3::ZERO))
        .init_resource::<BallAssets>()
        .add_startup_system(camera_setup)
        .add_startup_system(prepare_materials)
        .add_system_set(
            SystemSet::on_enter(States::WaitingPlayer)
                .with_system(spawn_paddles)
                .with_system(spawn_goals)
                .with_system(spawn_edges)
                .with_system(spawn_middle_line)
                .with_system(spawn_static_ball),
        )
        .add_system_set(SystemSet::on_update(States::WaitingPlayer).with_system(launch_ball))
        .add_system_set(
            SystemSet::on_update(States::InGame)
                .with_system(move_player_paddle)
                .with_system(track_balls_state),
        )
        .run();
}

// For wasm-pack to be happy...
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn wasm_main() {
    init();
}

fn camera_setup(mut commands: Commands) {
    let mut camera_bundle = OrthographicCameraBundle::new_2d();
    camera_bundle.orthographic_projection.scale = 1. / 50.;
    commands.spawn_bundle(camera_bundle);
}

fn prepare_materials(
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut ball_assets: ResMut<BallAssets>,
) {
    ball_assets.color_material = color_materials.add(ColorMaterial::from(Color::WHITE));
    ball_assets.mesh =
        Mesh2dHandle(meshes.add(Mesh::from(Icosphere { radius: 0.25, ..Default::default() })));
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
                .with_group(GamePhysicsLayers::Paddle)
                .with_masks(&[GamePhysicsLayers::Ball, GamePhysicsLayers::Edge]),
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
                .with_group(GamePhysicsLayers::Paddle)
                .with_masks(&[GamePhysicsLayers::Ball, GamePhysicsLayers::Edge]),
        )
        .insert(ComputerPaddle);
}

fn spawn_goals(mut commands: Commands) {
    // Player goal (on the right)
    commands
        .spawn()
        .insert(Transform::from_translation(Vec3::new(12., 0., 0.)))
        .insert(GlobalTransform::default())
        .insert(RigidBody::Sensor)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(1.0, 10., 0.),
            border_radius: None,
        })
        .insert(RotationConstraints::lock())
        .insert(
            CollisionLayers::none()
                .with_group(GamePhysicsLayers::Goal)
                .with_mask(GamePhysicsLayers::Ball),
        )
        .insert(PlayerGoal);

    // Computer goal (on the left)
    commands
        .spawn()
        .insert(Transform::from_translation(Vec3::new(-12., 0., 0.)))
        .insert(GlobalTransform::default())
        .insert(RigidBody::Sensor)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(1.0, 10., 0.),
            border_radius: None,
        })
        .insert(RotationConstraints::lock())
        .insert(
            CollisionLayers::none()
                .with_group(GamePhysicsLayers::Goal)
                .with_mask(GamePhysicsLayers::Ball),
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
                .with_group(GamePhysicsLayers::Edge)
                .with_masks(&[GamePhysicsLayers::Ball, GamePhysicsLayers::Paddle]),
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
                .with_group(GamePhysicsLayers::Edge)
                .with_masks(&[GamePhysicsLayers::Ball, GamePhysicsLayers::Paddle]),
        );
}

fn spawn_middle_line(mut commands: Commands) {
    // Top of the middle line
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: WHITE_COLOR,
            custom_size: Some(Vec2::new(0.1, 10.)),
            ..Default::default()
        },
        transform: Transform::from_translation(Vec3::new(0., 5.8, 0.)),
        ..Default::default()
    });

    // Bottom of the middle line
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: WHITE_COLOR,
            custom_size: Some(Vec2::new(0.1, 10.)),
            ..Default::default()
        },
        transform: Transform::from_translation(Vec3::new(0., -5.8, 0.)),
        ..Default::default()
    });
}

fn spawn_static_ball(mut commands: Commands, ball_assets: Res<BallAssets>) {
    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: ball_assets.mesh.clone(),
            material: ball_assets.color_material.clone(),
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
        .insert(CollisionLayers::none().with_group(GamePhysicsLayers::Ball).with_masks(&[
            GamePhysicsLayers::Paddle,
            GamePhysicsLayers::Goal,
            GamePhysicsLayers::Edge,
        ]))
        .insert(Ball::default());
}

fn launch_ball(
    mut keys: ResMut<Input<KeyCode>>,
    mut state: ResMut<State<States>>,
    mut balls_query: Query<&mut Velocity, With<Ball>>,
) {
    if keys.clear_just_released(KeyCode::Space) {
        state.set(States::InGame).unwrap();
        for mut velocity in balls_query.iter_mut() {
            velocity.linear = Vec3::new(3.0, 3.0, 0.0);
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

fn track_balls_state(_balls_query: Query<&mut Ball>) {
    // ...
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum States {
    AssetLoading,
    WaitingPlayer,
    InGame,
}

#[derive(PhysicsLayer)]
enum GamePhysicsLayers {
    Edge,
    Ball,
    Paddle,
    Goal,
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
}
