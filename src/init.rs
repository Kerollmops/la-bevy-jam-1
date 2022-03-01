use bevy::prelude::*;
use heron::prelude::*;

use crate::game_collisions::GamePhysicsLayer;
use crate::{Goal, Lifebar, LifebarAssets, Paddle, Side, BLUE_COLOR, RED_COLOR, WHITE_COLOR};

pub const PLAYER_PADDLE_HEIGHT: f32 = 5.;
pub const COMPUTER_PADDLE_HEIGHT: f32 = 8.;

pub fn camera_setup(mut commands: Commands) {
    let mut camera_bundle = OrthographicCameraBundle::new_2d();
    camera_bundle.orthographic_projection.scale = 1. / 50.;
    commands.spawn_bundle(camera_bundle);
}

pub fn spawn_paddles(mut commands: Commands) {
    // Player paddle (on the right)
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: BLUE_COLOR,
                custom_size: Some(Vec2::new(0.5, PLAYER_PADDLE_HEIGHT)),
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(11., 0., 0.)),
            ..Default::default()
        })
        .insert(Velocity::default())
        .insert(RigidBody::KinematicPositionBased)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(0.25, PLAYER_PADDLE_HEIGHT / 2., 0.),
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
        .insert(Paddle::Player);

    // Computer paddle (on the left)
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: RED_COLOR,
                custom_size: Some(Vec2::new(0.5, COMPUTER_PADDLE_HEIGHT)),
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(-11., 0., 0.)),
            ..Default::default()
        })
        .insert(Velocity::default())
        .insert(RigidBody::KinematicPositionBased)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(0.25, COMPUTER_PADDLE_HEIGHT / 2., 0.),
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
        .insert(Paddle::Computer);
}

pub fn spawn_goals(mut commands: Commands) {
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
        .insert(Goal::Player);

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
        .insert(Goal::Computer);
}

pub fn spawn_edges(mut commands: Commands) {
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

pub fn spawn_sides(mut commands: Commands) {
    // Right side
    commands
        .spawn()
        .insert(Transform::from_translation(Vec3::new(5.75, 0., 0.)))
        .insert(GlobalTransform::default())
        .insert(RigidBody::Sensor)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(5.75, 6., 0.),
            border_radius: None,
        })
        .insert(
            CollisionLayers::none()
                .with_group(GamePhysicsLayer::Side)
                .with_mask(GamePhysicsLayer::Ball),
        )
        .insert(Side::Player);

    // Left side
    commands
        .spawn()
        .insert(Transform::from_translation(Vec3::new(-5.75, 0., 0.)))
        .insert(GlobalTransform::default())
        .insert(RigidBody::Sensor)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(5.75, 6., 0.),
            border_radius: None,
        })
        .insert(
            CollisionLayers::none()
                .with_group(GamePhysicsLayer::Side)
                .with_mask(GamePhysicsLayer::Ball),
        )
        .insert(Side::Computer);
}

pub fn spawn_field_lines(mut commands: Commands) {
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
}

pub fn spawn_lifebars(mut commands: Commands, assets: Res<LifebarAssets>) {
    // Computer life bar
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: assets.texture_atlas.clone(),
            transform: Transform::from_translation(Vec3::new(-7.5, 6.625, 0.)),
            sprite: TextureAtlasSprite {
                index: 15,
                custom_size: Some(Vec2::new(6., 0.5)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Lifebar::Computer);

    // Player life bar
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: assets.texture_atlas.clone(),
            transform: Transform::from_translation(Vec3::new(7.5, 6.625, 0.)),
            sprite: TextureAtlasSprite {
                index: 15,
                flip_x: true,
                custom_size: Some(Vec2::new(6., 0.5)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Lifebar::Player);
}
