use bevy::{
    asset::AssetMetaCheck, camera::ScalingMode, color::palettes::tailwind::{RED_400, SLATE_50}, image::{ImageAddressMode, ImageLoaderSettings}, prelude::{*, Color}, render::render_resource::AsBindGroup, shader::ShaderRef, sprite_render::{Material2d, Material2dPlugin},
};
use bevy::math::bounding::*;
use bevy::audio::{AddAudioSource, Volume};
use flappy_bird::*;


#[derive(Resource, Default)]
pub struct Score(pub u32);

#[derive(Event)]
pub struct ScorePoint;

#[derive(Component)]
pub struct ScoreText;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct BackgroundMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub color_texture: Handle<Image>,
}

impl Material2d for BackgroundMaterial {
    fn fragment_shader() -> ShaderRef {
        "background.wgsl".into()
    }
}


fn main() {
    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    canvas: Some("#bevy".into()), 
                    title: "bird flap".into(),
                    resolution: (800, 600).into(),
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest())
            // Web dev servers return index.html (200) for missing files, so
            // Bevy's per-asset `.meta` lookups get HTML instead of a 404 and
            // fail to load. Skipping the meta check fixes web; harmless on native.
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            }),
        )
        .add_plugins((
            PipePlugin,
            Material2dPlugin::<BackgroundMaterial>::default(),
        ))
        .init_state::<GameState>()
        .init_resource::<DebugSettings>()
        .init_resource::<Score>()
        .insert_resource(GlobalVolume::new(Volume::Linear(GLOBAL_VOLUME)))
        .add_audio_source::<EnvelopedPitch>()
        .add_message::<PlayPitch>()
        .add_systems(Startup, (
                setup_camera,
                startup,
                spawn_mute_button,
                setup_audio,
            )
            .chain()
        )
        .add_systems(
            OnEnter(GameState::Playing),
            (
                birdy,
                show_score
            )
        )
        .add_systems(
            OnEnter(GameState::GameOver),
            (
                game_over,
            )
        )
        .add_systems(Update, 
            (
                enter_playing
                    .run_if(check_state),
                control.run_if(in_state(GameState::Playing)),
                score_update
                    .run_if(resource_changed::<Score>)
                    .run_if(in_state(GameState::Playing)),
                bird_rotation.run_if(in_state(GameState::Playing)),
                instruction.run_if(in_state(GameState::MainMenu)),
                mute_audio
            )
        )
        .add_systems(FixedUpdate, (
                gravity,
                border_patrol,
                check_collisions,
                play_pitch,
                play_delayed_pitch
            )
            .chain()
            .run_if(in_state(GameState::Playing)),
        )
        .insert_resource(AudioEnabled(true))
        .add_observer(
            |_trigger: On<ScorePoint>, mut score: ResMut<Score>| {
                score.0 +=1;
            },
        )
        .run();
}

pub const GLOBAL_VOLUME: f32 = 0.15;

pub fn setup_camera(mut commands: Commands){
    commands.spawn((
        Camera2d,
    ));
}

pub fn check_state (
    state: Res<State<GameState>>
) -> bool {
    *state.get() == GameState::MainMenu || *state.get() == GameState::GameOver
}

pub fn startup (
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BackgroundMaterial>>,
) {
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(
            CANVAS_SIZE.x,
            CANVAS_SIZE.y,
        ))),
        MeshMaterial2d(materials.add(BackgroundMaterial {
            color_texture: asset_server.load_with_settings(
                "birdflap-background.png", 
                |settings: &mut ImageLoaderSettings| {
                    settings
                        .sampler
                        .get_or_init_descriptor()
                        .set_address_mode(
                            ImageAddressMode::Repeat,
                        );
                },
            ),
        })),
    ));
}

pub fn show_score (
    mut commands: Commands,
    mut score: ResMut<Score>,
) {
    score.0 = 0;
    commands.spawn((
        Node {
            width: percent(100.),
            margin: px(20.).top(),
            align_items: AlignItems::Center,
            ..default()
        },
        Text::new("0"),
        TextLayout::new_with_justify(Justify::Center),
        TextFont{
            font_size: 33.0,
            ..default()
        },
        TextColor(SLATE_50.into()),
        ScoreText,
        DespawnOnExit(GameState::Playing),
    ));
}

fn score_update (
    mut query: Query<&mut Text, With<ScoreText>>,
    score: Res<Score>,
) {
    for mut span in &mut query {
        span.0 = score.0.to_string();
    }
}

pub fn instruction (
    mut commands: Commands,
) {
    commands.spawn((
        Visibility::Visible,
        instruction_text(),
        DespawnOnExit(GameState::MainMenu)
    ));
}

fn instruction_text () -> impl Bundle {
    (
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default() 
        },
        children![(
            Node {
                width: percent(100),
                height: px(100),
                border: UiRect::all(px(5)),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                border_radius: BorderRadius::MAX,
                ..default()
            },
            children![
                (
                    Text::new("Press "),
                    TextFont {
                        font_size: 40.,
                        ..default()
                    },
                    TextColor(SLATE_50.into()),
                ),
                (
                    Node {
                        width: px(200),
                        height: px(65),
                        border: UiRect::all(px(5)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::new(Val::Px(10.),Val::Px(10.),Val::Px(10.),Val::Px(10.)),
                        ..default()
                    },
                    BorderColor::all(Color::WHITE),
                    BackgroundColor(Color::srgb(0.5,0.5,0.5)),
                    children![(
                        Text::new("SPACE"),
                        TextFont {
                            font_size: 30.,
                            ..default()
                        },
                        TextColor(SLATE_50.into()),
                    )]
                ),
                (
                    Text::new(" to Start."),
                    TextFont {
                        font_size: 40.,
                        ..default()
                    },
                    TextColor(SLATE_50.into()),
                )
            ]
        )]
    )
}

#[derive(Component)]
pub struct Bird;

#[derive(Component)]
pub struct Collider(pub Vec2);

#[derive(Resource)]
struct AudioIcons {
    on: Handle<Image>,
    off: Handle<Image>,
}

pub const SPAWN_X: f32 = -250.0;
pub const SPAWN_Y: f32 = 0.0;
pub const SPAWN_Z: f32 = 0.0;
pub const VELOCITY: f32 = 600.0;
pub const GRAVITY: f32 = 1200.0;
pub const SOUND_BUTTONVEC: Vec2 = Vec2::new(1134.0, 48.0);

#[derive(Resource, Default)]
pub struct DebugSettings {
    pub show_gizmos: bool,
}

#[derive(Component)]
#[require(Gravity(GRAVITY), Velocity)]
struct Player;

#[derive(Component)]
struct Gravity(f32);

#[derive(Component, Default)]
struct Velocity(f32);


fn gravity (
    mut transform: Query <(&mut Transform, &mut Velocity, &Gravity)>,
    time: Res<Time>,
) {
    for (mut pos, mut vel, gravity) in &mut transform {
        vel.0 -= gravity.0 * time.delta_secs();
        pos.translation.y += vel.0 * time.delta_secs();
    }
}

fn birdy (
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

fn bird_rotation (
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

fn control (
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


fn border_patrol (
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

fn check_collisions(
    mut commands: Commands,
    bird: Single<(&Sprite, Entity), With<Player>>,
    pipe_segments: Query<
        Entity,
        Or<(With<PipeTop>, With<PipeBottom>)>,
    >,
    pipe_gaps: Query<(&Sprite, Entity), With<PointsGate>>,
    transform_helper: TransformHelper,
    mut play_pitch_writer: MessageWriter<PlayPitch>,
    mut game_state: ResMut<NextState<GameState>>
) -> Result<()> {
    let bird_transform = transform_helper
        .compute_global_transform(bird.1)?;
    let bird_collider = BoundingCircle::new(
        bird_transform.translation().xy(),
        (BIRD_SIZE.y - PADDING_Y) / 2.0,
    );

    // gizmos.circle_2d(
    //     bird_transform.translation().xy(),
    //     BIRD_SIZE.x / 2.0,
    //     RED_400,
    // );
    let custom_size = Vec2::new(PIPE_SIZE.x - PIPE_X_OFFSET, PIPE_SIZE.y);

    for entity in &pipe_segments {
        let pipe_transform = transform_helper
            .compute_global_transform(entity)?;
        let pipe_collider = Aabb2d::new(
            pipe_transform.translation().xy(),
            custom_size / 2.0,
        );
        // gizmos.rect_2d(
        // pipe_transform.translation().xy(),
        // custom_size,
        // RED_400,
        // );
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


pub fn enter_playing (
    input: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<NextState<GameState>>,
    touches: Res<Touches>,
) {
    if input.just_pressed(KeyCode::Space) {
        game_state.set(GameState::Playing);
    }
    for touch in touches.iter_just_pressed() {
        if touch.position() != SOUND_BUTTONVEC {
            game_state.set(GameState::Playing);
        }
    }
}

fn game_over (
    mut commands: Commands,
    score: Res<Score>,
) {
    let final_score = format!("Your final score is {}", score.0.to_string());

    commands.spawn(
            (
        DespawnOnExit(GameState::GameOver),
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(30),
            ..default() 
        },
        BackgroundColor(Color::srgba(0.7, 0.7, 0.7, 0.8)),
        children![
            (
                Text::new("Game Over"),
                TextFont { font_size: 80.0, ..default()},
                TextColor(Color::srgb(1.0, 0.84, 0.0)),

            ),
            (
                Text::new(final_score),
                TextFont { font_size: 40., ..default() },
                TextColor(SLATE_50.into()),
            ),
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(10),
                    ..default()
                },
                children![
                    (
                        Text::new("Press"),
                        TextFont { font_size: 40., ..default()},
                        TextColor(SLATE_50.into()),
                    ),
                    (
                        Node {
                            width: px(150),
                            height: px(65),
                            border: UiRect::all(px(5)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::new(Val::Px(10.),Val::Px(10.),Val::Px(10.),Val::Px(10.)),
                            ..default()
                        },
                        BorderColor::all(Color::WHITE),
                        BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
                        children![(
                            Text::new("SPACE"),
                            TextFont { font_size: 40., ..default()},
                            TextColor(SLATE_50.into()),
                        )],
                    ),
                    (
                        Text::new("to restart!"),
                        TextFont {font_size: 40., ..default()},
                        TextColor(SLATE_50.into()),
                    )
                ],
            )
        ]
            
        ));
}

fn spawn_mute_button (
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let icons = AudioIcons {
        on: asset_server.load("icons/sound_on.png"),
        off: asset_server.load("icons/sound_off.png"),
    };
    commands.spawn((
        Button,
        AudioToggleButton,
        Node {
            position_type: PositionType::Absolute, // take it out of layout flow
            top: Val::Px(12.0),
            right: Val::Px(12.0),                  // pin to top-right
            width: Val::Px(48.0),
            height: Val::Px(48.0),
            // border: UiRect::all(Val::Px(2.0)),
            // justify_content: JustifyContent::Center,
            // align_items: AlignItems::Center,
            ..default()
        },
        ImageNode::new(icons.on.clone()),
    ));
    commands.insert_resource(icons);
}

fn mute_audio(
    mut interaction: Query<
        (&Interaction, &mut ImageNode, &ComputedNode, &UiGlobalTransform),
        (Changed<Interaction>, With<AudioToggleButton>),
    >,
    mut audio_enabled: ResMut<AudioEnabled>,
    mut global: ResMut<GlobalVolume>,
    icons: Res<AudioIcons>,
    touches: Res<Touches>,
) {
    for (interact, mut image, _, &global_pos) in &mut interaction {
        if *interact == Interaction::Pressed {
            audio_enabled.0 = !audio_enabled.0;
            global.volume = if audio_enabled.0 {
                Volume::Linear(GLOBAL_VOLUME)
            } else {
                Volume::SILENT
            };
            image.image = if audio_enabled.0 { icons.on.clone() } else { icons.off.clone() };
        }

        for touch in touches.iter_just_pressed() {
            if touch.position() == global_pos.translation {
                audio_enabled.0 = !audio_enabled.0;
            }
        }
    }
}