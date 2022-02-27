use bevy::prelude::*;
use heron::prelude::*;

#[derive(Debug, Copy, Clone)]
pub enum GameCollisionEvent {
    BallAndPaddle { status: CollisionStatus, ball: Entity, paddle: Entity },
    BallAndGoal { status: CollisionStatus, ball: Entity, goal: Entity },
    BallAndEdge { status: CollisionStatus, ball: Entity, edge: Entity },
    BallAndBonus { status: CollisionStatus, ball: Entity, bonus: Entity },
    BallAndSide { status: CollisionStatus, ball: Entity, side: Entity },
}

#[derive(Debug, Copy, Clone)]
pub enum CollisionStatus {
    Started,
    Stopped,
}

#[derive(PhysicsLayer)]
pub enum GamePhysicsLayer {
    Ball,
    Paddle,
    Goal,
    Edge,
    Side,
    Bonus,
}

pub fn produce_game_collision_events(
    mut in_events: EventReader<CollisionEvent>,
    mut out_events: EventWriter<GameCollisionEvent>,
) {
    use CollisionStatus::*;
    use GameCollisionEvent::*;

    for event in in_events.iter() {
        let (entity_1, entity_2) = event.rigid_body_entities();
        let (layers_1, layers_2) = event.collision_layers();
        let status = if event.is_started() { Started } else { Stopped };

        // ball and paddle collide
        if is_paddle_layer(layers_1) && is_ball_layer(layers_2) {
            out_events.send(BallAndPaddle { status, ball: entity_2, paddle: entity_1 });
        } else if is_paddle_layer(layers_2) && is_ball_layer(layers_1) {
            out_events.send(BallAndPaddle { status, ball: entity_1, paddle: entity_2 });
        // ball and goal collide
        } else if is_goal_layer(layers_1) && is_ball_layer(layers_2) {
            out_events.send(BallAndGoal { status, ball: entity_2, goal: entity_1 });
        } else if is_goal_layer(layers_2) && is_ball_layer(layers_1) {
            out_events.send(BallAndGoal { status, ball: entity_1, goal: entity_2 });
        // ball and edge collide
        } else if is_edge_layer(layers_1) && is_ball_layer(layers_2) {
            out_events.send(BallAndEdge { status, ball: entity_2, edge: entity_1 });
        } else if is_edge_layer(layers_2) && is_ball_layer(layers_1) {
            out_events.send(BallAndEdge { status, ball: entity_1, edge: entity_2 });
        // ball and bonus collide
        } else if is_bonus_layer(layers_1) && is_ball_layer(layers_2) {
            out_events.send(BallAndBonus { status, ball: entity_2, bonus: entity_1 });
        } else if is_bonus_layer(layers_2) && is_ball_layer(layers_1) {
            out_events.send(BallAndBonus { status, ball: entity_1, bonus: entity_2 });
        // ball and side collide
        } else if is_side_layer(layers_1) && is_ball_layer(layers_2) {
            out_events.send(BallAndSide { status, ball: entity_2, side: entity_1 });
        } else if is_side_layer(layers_2) && is_ball_layer(layers_1) {
            out_events.send(BallAndSide { status, ball: entity_1, side: entity_2 });
        }
    }
}

fn is_paddle_layer(layers: CollisionLayers) -> bool {
    layers.contains_group(GamePhysicsLayer::Paddle)
}

fn is_goal_layer(layers: CollisionLayers) -> bool {
    layers.contains_group(GamePhysicsLayer::Goal)
}

fn is_ball_layer(layers: CollisionLayers) -> bool {
    layers.contains_group(GamePhysicsLayer::Ball)
}

fn is_edge_layer(layers: CollisionLayers) -> bool {
    layers.contains_group(GamePhysicsLayer::Edge)
}

fn is_side_layer(layers: CollisionLayers) -> bool {
    layers.contains_group(GamePhysicsLayer::Side)
}

fn is_bonus_layer(layers: CollisionLayers) -> bool {
    layers.contains_group(GamePhysicsLayer::Bonus)
}
