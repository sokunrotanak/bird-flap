use bevy::{
    render::render_resource::AsBindGroup,
    prelude::*,
    color::palettes::tailwind::{RED_400, SLATE_50},
    shader::ShaderRef,
    sprite_render::Material2d,
    image::{ImageAddressMode, ImageLoaderSettings},
};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (
            setup_camera,
            background,
        ).chain()
        );
        app.add_systems(OnEnter(GameState::Playing), (
            show_score,
        ));
        app.add_systems(Update, (
                score_update
                    .run_if(resource_changed::<Score>)
                    .run_if(in_state(GameState::Playing)),
                enter_playing
                    .run_if(check_state),
        ));
        app.add_systems(
            OnEnter(GameState::GameOver),
            (
                game_over,
            )
        );
        app.init_state::<GameState>();
        app.init_resource::<Score>();
        app.add_observer(
            |_trigger: On<ScorePoint>, mut score: ResMut<Score>| {
                score.0 +=1;
            },
        );
    }
}

pub const CANVAS_SIZE: Vec2 = Vec2::new(800.0, 600.0);
pub const BIRD_CUSTOM: Vec2 = Vec2::new(128.0, 105.0);
pub const SCALE_FACTOR: f32 = 0.5;
pub const BIRD_SIZE: Vec2 = Vec2::new(BIRD_CUSTOM.x * SCALE_FACTOR, BIRD_CUSTOM.y * SCALE_FACTOR);

pub const PADDING_X: f32 = 20.0;
pub const PADDING_Y: f32 = 20.0;
pub const COLLIDER: Vec2 = Vec2::new(BIRD_SIZE.x - PADDING_X, BIRD_SIZE.y - PADDING_Y);
pub const PIPE_SIZE: Vec2 = Vec2::new(128.0, CANVAS_SIZE.y);
pub const GAP_SIZE: f32 = 180.0;
pub const PIPE_SPEED: f32 = 200.0;
pub const PIPE_X_OFFSET: f32 = 64.0;

pub const SPAWN_X: f32 = -250.0;
pub const SPAWN_Y: f32 = 0.0;
pub const SPAWN_Z: f32 = 0.0;
pub const VELOCITY: f32 = 600.0;
pub const GRAVITY: f32 = 1200.0;
pub const SOUND_BUTTONVEC: Vec2 = Vec2::new(1134.0, 48.0);

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    MainMenu,
    Playing,
    GameOver,
}

#[derive(Resource, Default)]
pub struct Score(pub u32);

#[derive(Event)]
pub struct ScorePoint;

#[derive(Component)]
pub struct ScoreText;

pub fn setup_camera(mut commands: Commands){
    commands.spawn((
        Camera2d,
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
        TextLayout::justify(Justify::Center),
        TextFont{
            font_size: Val::Px(33.0).into(),
            ..default()
        },
        TextColor(SLATE_50.into()),
        ScoreText,
        DespawnOnExit(GameState::Playing),
    ));
}

pub fn score_update (
    mut query: Query<&mut Text, With<ScoreText>>,
    score: Res<Score>,
) {
    for mut span in &mut query {
        span.0 = score.0.to_string();
    }
}

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

pub fn background (
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
            color_texture: asset_server
                .load_builder()
                .with_settings(|settings: &mut ImageLoaderSettings| {
                    settings
                        .sampler
                        .get_or_init_descriptor()
                        .set_address_mode(
                            ImageAddressMode::Repeat,
                        );
                },)
                .load("birdflap-background.png"), 
                
            })),
        ));
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

pub fn check_state (
    state: Res<State<GameState>>
) -> bool {
    *state.get() == GameState::MainMenu || *state.get() == GameState::GameOver
}


pub fn game_over (
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
                TextFont { 
                    font_size: Val::Px(80.0).into(),
                    ..default()},
                TextColor(Color::srgb(1.0, 0.84, 0.0)),

            ),
            (
                Text::new(final_score),
                TextFont { 
                    font_size: Val::Px(40.).into(), 
                    ..default() },
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
                        TextFont { 
                            font_size: Val::Px(40.).into(), 
                            ..default()},
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
                            TextFont { 
                                font_size: Val::Px(40.).into(), 
                                ..default()},
                            TextColor(SLATE_50.into()),
                        )],
                    ),
                    (
                        Text::new("to restart!"),
                        TextFont {font_size: Val::Px(40.).into(),
                            ..default()},
                        TextColor(SLATE_50.into()),
                    )
                ],
            )
        ]
            
        ));
}