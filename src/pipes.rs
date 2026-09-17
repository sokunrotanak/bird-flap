use rand::RngExt;
use bevy::{
    prelude::*,
    time::common_conditions::on_timer
};
use std::time::Duration;
use crate::world::*;

pub struct PipePlugin;

impl Plugin for PipePlugin {
    fn build(&self, app: &mut App){
        app.add_systems(
            FixedUpdate,(
            spawn_pipes
                .run_if(on_timer(Duration::from_millis(2000)))
                .run_if(in_state(GameState::Playing)),
            despawn_pipes.run_if(in_state(GameState::Playing)),
        ));
        app.add_systems(PostUpdate, 
            shift_pipes_to_the_left
            .before(TransformSystems::Propagate)
            .run_if(in_state(GameState::Playing))
        );
    }
}

#[derive(Component)]
struct Pipe;

#[derive(Component)]
pub struct PipeTop;

#[derive(Component)]
pub struct PipeBottom;

#[derive(Component)]
pub struct PointsGate;

fn spawn_pipes (
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    // time: Res<Time>,
) {

    let gap_y_position = rand::rng().random_range(-(CANVAS_SIZE.y/4.)..(CANVAS_SIZE.y/4.));
    let transform = Transform::from_xyz(CANVAS_SIZE.x /2.0, 0., 1.);
    let pipe_offset = PIPE_SIZE.y / 2.0 + GAP_SIZE / 2.0;
    let image = asset_server.load("pipe.png");
    let image_mode = SpriteImageMode::Sliced(
                TextureSlicer {
                    border: BorderRect::axes(48., 24.),
                    center_scale_mode: SliceScaleMode::Stretch,
                    ..default()
                },
            );

    commands.spawn((
        transform,
        DespawnOnExit(GameState::GameOver),
        Visibility::Visible,
        Pipe,
        children![
            (
                Sprite {
                    image: image.clone(),
                    custom_size: Some(PIPE_SIZE),
                    image_mode: image_mode.clone(),
                    ..default()
                },
                Transform::from_xyz(
                    0.0,
                    pipe_offset + gap_y_position,
                    0.0
                ),
                PipeTop
            ),
            (
                Visibility::Hidden,
                Sprite {
                    color: Color::WHITE,
                    custom_size: Some(Vec2::new(
                        10.0, GAP_SIZE,
                    )),
                    ..default()
                },
                Transform::from_xyz(
                    0.0,
                    gap_y_position,
                    1.0
                ),
                PointsGate,
            ),
            (
                Sprite {
                    image: image,
                    custom_size: Some(PIPE_SIZE),
                    image_mode: image_mode.clone(),
                    ..default()
                },
                Transform::from_xyz(
                    0.0,
                    -pipe_offset + gap_y_position,
                    0.0
                ),
                PipeBottom
            ),
        ],
    ));
}

fn shift_pipes_to_the_left (
    mut pipes: Query<&mut Transform, With<Pipe>>,
    time: Res<Time>,
) {
    for mut pipe in &mut pipes {
        pipe.translation.x -= PIPE_SPEED * time.delta_secs();
    }
}

fn despawn_pipes (
    mut commands: Commands,
    pipes: Query<(Entity, &Transform), With<Pipe>>,
) {
    for (entity, pos) in pipes.iter() {
        if pos.translation.x < -(CANVAS_SIZE.x/2.0 + PIPE_SIZE.x) {
            commands.entity(entity).despawn();
        }
    }
}
