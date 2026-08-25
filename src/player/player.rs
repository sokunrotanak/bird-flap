use bevy::{
    color::palettes::tailwind::RED_400, 
    math::bounding::*, 
    prelude::*,
};
use crate::world::*;
use crate::pipes::*;
use crate::audio::*;

// #[derive(Component)]
// pub struct Bird;

#[derive(Component)]
pub struct Collider(pub Vec2);

#[derive(Component)]
#[require(Gravity(GRAVITY), Velocity)]
pub struct Player;

#[derive(Component)]
pub struct Gravity(pub f32);

#[derive(Component, Default)]
pub struct Velocity(pub f32);

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_birdy);
        app.add_systems(Update, (
            control.run_if(in_state(GameState::Playing)),
            bird_rotation.run_if(in_state(GameState::Playing)),
            render_gizmos.run_if(in_state(GameState::Playing)),
        ));
        app.add_systems(FixedUpdate, (
            gravity,
            border_patrol,
            check_collisions,
        ).chain().run_if(in_state(GameState::Playing))
        );
    }
}


pub fn gravity (
    mut transform: Query <(&mut Transform, &mut Velocity, &Gravity)>,
    time: Res<Time>,
) {
    for (mut pos, mut vel, gravity) in &mut transform {
        vel.0 -= gravity.0 * time.delta_secs();
        pos.translation.y += vel.0 * time.delta_secs();
    }
}

pub fn spawn_birdy (
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    commands.spawn((
        Sprite {
            image: asset_server.load("bird-flap-bird128x105.png"),
            custom_size: Some(BIRD_SIZE),
            ..default()
        },
        Transform::from_xyz(SPAWN_X,SPAWN_Y, SPAWN_Z),
        Collider(COLLIDER),
        Player,
        DespawnOnExit(GameState::GameOver)
    ));
}

pub fn bird_rotation (
    mut bird: Single<
        (&mut Transform, &Velocity),
        With<Player>,
    >,
) {
    let calculated_velocity = Vec2::new(PIPE_SPEED, bird.1.0);
    bird.0.rotation = Quat::from_rotation_z(
        calculated_velocity.to_angle(),
    );
}

pub fn control (
    mut velocity: Single<&mut Velocity, With<Player>>,
    buttons: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
) {
    if buttons.just_pressed(KeyCode::Space){
        velocity.0 = VELOCITY;
    }
    for touch in touches.iter_just_pressed() {
        if touch.position() != SOUND_BUTTONVEC {
            velocity.0 = VELOCITY;
        }
        
    }
}

pub const CLAMP_VEL: f32 = 2000.0;

pub fn border_patrol (
    mut bird: Query<(&mut Transform, &mut Velocity, &Gravity, &Collider), With<Player>>,
    time: Res<Time>,
    mut game_state: ResMut<NextState<GameState>>
) {
    for (mut pos, mut vel, gravity, col) in &mut bird{
        if pos.translation.y > CANVAS_SIZE.y / 2.0 - col.0.y /2.0 { 
            pos.translation.y = CANVAS_SIZE.y / 2.0 - col.0.y /2.0;
            vel.0 -= gravity.0 * time.delta_secs();
            vel.0 = vel.0.clamp(-CLAMP_VEL, CLAMP_VEL);
            pos.translation.y += vel.0 * time.delta_secs();
        }
        if pos.translation.y <= (-CANVAS_SIZE.y / 2.0) - (col.0.y *1.5) {
            game_state.set(GameState::GameOver);
        }
    }
}

pub fn check_collisions(
    mut commands: Commands,
    bird: Single<(&Sprite, Entity), With<Player>>,
    pipe_segments: Query<
        Entity,
        Or<(With<PipeTop>, With<PipeBottom>)>,
    >,
    pipe_gaps: Query<(&Sprite, Entity), With<PointsGate>>,
    transform_helper: TransformHelper,
    mut gizmos: Gizmos,
    mut play_pitch_writer: MessageWriter<PlayPitch>,
    mut game_state: ResMut<NextState<GameState>>
) -> Result<()> {
    let bird_transform = transform_helper
        .compute_global_transform(bird.1)?;
    let bird_collider = BoundingCircle::new(
        bird_transform.translation().xy(),
        (BIRD_SIZE.y - PADDING_Y) / 2.0,
    );

    gizmos.circle_2d(
        bird_transform.translation().xy(),
        (BIRD_SIZE.y - PADDING_Y) / 2.0,
        RED_400,
    );
    let custom_size = Vec2::new(PIPE_SIZE.x - PIPE_X_OFFSET, PIPE_SIZE.y);

    for entity in &pipe_segments {
        let pipe_transform = transform_helper
            .compute_global_transform(entity)?;
        let pipe_collider = Aabb2d::new(
            pipe_transform.translation().xy(),
            custom_size / 2.0,
        );
        gizmos.rect_2d(
        pipe_transform.translation().xy(),
        custom_size,
        RED_400,
        );
        if bird_collider.intersects(&pipe_collider) {
            game_state.set(GameState::GameOver);
        }
    }

    for (sprite, entity) in &pipe_gaps {
        let gap_transform = transform_helper
            .compute_global_transform(entity)?;
        let gap_collider = Aabb2d::new(
            gap_transform.translation().xy(),
            sprite.custom_size.unwrap() / 2.0,
        );
        if bird_collider.intersects(&gap_collider) {
            commands.trigger(ScorePoint);
            commands.entity(entity).despawn();
            play_pitch_writer.write(PlayPitch);
        }
    }

    Ok(())
}

pub fn render_gizmos (
    mut config_store: ResMut<GizmoConfigStore>,
) {
    let (config, _) = config_store
        .config_mut::<DefaultGizmoConfigGroup>();
    config.enabled = false;
}