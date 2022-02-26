use bevy::prelude::*;
use bevy_asset_loader::AssetLoader;
use heron::prelude::*;

use self::game_assets::GameAssets;

mod game_assets;

fn main() {
    let mut app = App::new();
    AssetLoader::new(States::AssetLoading)
        .with_collection::<GameAssets>()
        .continue_to_state(States::Next)
        .build(&mut app);

    app.add_state(States::AssetLoading)
        .insert_resource(ClearColor(Color::rgb(0.239, 0.239, 0.239)))
        .add_plugins(DefaultPlugins)
        .add_plugin(PhysicsPlugin::default())
        .insert_resource(Gravity::from(Vec3::ZERO))
        .add_startup_system(camera_setup)
        // .add_system_to_stage(CoreStage::PostUpdate, camera_follow)
        .add_system_set(
            SystemSet::on_enter(States::Next)
                .with_system(spawn_paddles)
                .with_system(spawn_goals)
                .with_system(spawn_edges)
                // .add_startup_system(setup)
                // .with_system(spawn_player)
                // .with_system(spawn_ennemies),
        )
        .add_system_set(
            SystemSet::on_update(States::Next), // .with_system(produce_game_collision_events)
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
                color: Color::rgb(0.839, 0.839, 0.839),
                custom_size: Some(Vec2::new(0.5, 5.0)),
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(10., 0., 0.)),
            ..Default::default()
        })
        .insert(Velocity::default())
        .insert(RigidBody::KinematicPositionBased)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(0.25, 2.5, 0.),
            border_radius: None,
        })
        .insert(RotationConstraints::lock())
        .insert(
            CollisionLayers::none()
                .with_group(GamePhysicsLayers::Paddle)
                .with_masks(&[GamePhysicsLayers::Ball, GamePhysicsLayers::Edge]),
        )
        .insert(Player);

    // Computer paddle (on the left)
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.839, 0.839, 0.839),
                custom_size: Some(Vec2::new(0.5, 5.0)),
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(-10., 0., 0.)),
            ..Default::default()
        })
        .insert(Velocity::default())
        .insert(RigidBody::KinematicPositionBased)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(0.25, 2.5, 0.),
            border_radius: None,
        })
        .insert(RotationConstraints::lock())
        .insert(
            CollisionLayers::none()
                .with_group(GamePhysicsLayers::Paddle)
                .with_masks(&[GamePhysicsLayers::Ball, GamePhysicsLayers::Edge]),
        )
        .insert(Computer);
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
        .insert(
            CollisionLayers::none()
                .with_group(GamePhysicsLayers::Edge)
                .with_masks(&[GamePhysicsLayers::Ball, GamePhysicsLayers::Paddle]),
        );
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum States {
    AssetLoading,
    Next,
}

#[derive(PhysicsLayer)]
enum GamePhysicsLayers {
    Edge,
    Ball,
    Paddle,
    Goal,
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct PlayerGoal;

#[derive(Component)]
struct Computer;

#[derive(Component)]
struct ComputerGoal;
