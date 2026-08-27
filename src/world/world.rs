use bevy::{
    render::render_resource::AsBindGroup,
    prelude::*,
    color::palettes::tailwind::SLATE_50,
    shader::ShaderRef,
    sprite_render::Material2d,
    image::{ImageAddressMode, ImageLoaderSettings},
    camera::{ScalingMode,Viewport},
    window::WindowResized,
};

use crate::world::menu::*;

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
                fit_camera_viewport,
                score_update
                    .run_if(resource_changed::<Score>)
                    .run_if(in_state(GameState::Playing)),
                enter_playing,
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

const TARGET_WIDTH: f32 = 800.0;
const TARGET_HEIGHT: f32 = 600.0;

pub fn setup_camera(mut commands: Commands){
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: TARGET_WIDTH,
                height: TARGET_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}

fn fit_camera_viewport(
    mut resize_event: MessageReader<WindowResized>,
    windows: Query<&Window>,
    mut cameras: Query<&mut Camera>,
) {
    for event in resize_event.read() {
        let Ok(window) = windows.get(event.window) else {continue};
        let Ok(mut camera) = cameras.single_mut() else {continue};
        
        let window_size = window.physical_size().as_vec2();

        let target = Vec2::new(TARGET_WIDTH, TARGET_HEIGHT) * window.scale_factor();

        let viewport_size = target.min(window_size);
        let position = ((window_size - viewport_size) / 2.0).max(Vec2::ZERO);

        camera.viewport = Some(Viewport {
            physical_position: position.as_uvec2(),
            physical_size: viewport_size.as_uvec2(),
            ..default()
        });
        
    }
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
    state: Res<State<GameState>>,
    touches: Res<Touches>,
) {
    if input.just_pressed(KeyCode::Space) && *state.get() == GameState::GameOver {
        game_state.set(GameState::Playing);
    }
    for touch in touches.iter_just_pressed()  {
        if touch.position() != SOUND_BUTTONVEC && *state.get() == GameState::GameOver {
            game_state.set(GameState::Playing);
        }
    }
}

pub fn game_over (
    mut commands: Commands,
    score: Res<Score>,
) {
    let final_score = format!("Your final score is {}", score.0.to_string());
    let pad_px = 20.;
    let button_text_style = (
        TextFont {
            font_size: FontSize::Px(13.0),
            ..default()
        },
        TextColor(TEXT_COLOR),
    );
    let button_node = Node {
        width: px(150),
        height: px(40),
        margin: UiRect::all(px(10)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border_radius: BorderRadius::new(
            Val::Px(10.),
            Val::Px(10.),
            Val::Px(10.),
            Val::Px(10.)
        ),
        ..default()
    };
    commands
        .spawn(
            (
        
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(20),
            ..default() 
        },
        DespawnOnExit(GameState::GameOver),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: percent(100),
                        height: percent(100),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        row_gap: px(20),
                        padding: UiRect::new(
                            auto(),
                            auto(),
                            Val::Px(180.),
                            auto(),
                        ),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.7, 0.7, 0.7, 0.7)),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Game Over"),
                        TextFont { 
                            font_size: Val::Px(80.0).into(),
                            ..default()},
                        TextColor(Color::srgb(1.0, 0.84, 0.0)),
                    ));
                    parent.spawn((
                        Text::new(final_score),
                        TextFont { 
                            font_size: Val::Px(40.).into(), 
                            ..default() },
                        TextColor(SLATE_50.into()),
                    ));
                })
                .with_children(|parent| {
                    parent
                        .spawn(
                            Node {
                                width: percent(100),
                                height: percent(100),
                                padding: UiRect::new( 
                                    Val::Px(200.),
                                    Val::Px(200.), 
                                    Val::Px(0.), 
                                    auto(),//Val::Px(20.), 
                                ),
                                justify_content: JustifyContent::SpaceAround,
                                align_items: AlignItems::Center,
                                ..default()
                            }
                        )
                        .with_children(|parent| {
                            parent.spawn((
                                Button,
                                button_node.clone(),
                                BackgroundColor(NORMAL_BUTTON),
                                MenuButtonAction::BackToMainMenu,
                                children![
                                    (Text::new("Back To Main Menu"), button_text_style.clone())
                                ]
                            ));
                            parent.spawn((
                                Button,
                                button_node,
                                BackgroundColor(NORMAL_BUTTON),
                                MenuButtonAction::Play,
                                children![
                                    (Text::new("Restart"), button_text_style)
                                ]
                            ));
                        })
                    ;
                })
                ;

        });
        // children![
        //     (
        //         Text::new("Game Over"),
        //         TextFont { 
        //             font_size: Val::Px(80.0).into(),
        //             ..default()},
        //         TextColor(Color::srgb(1.0, 0.84, 0.0)),

        //     ),
        //     (
                // Text::new(final_score),
                // TextFont { 
                //     font_size: Val::Px(40.).into(), 
                //     ..default() },
                // TextColor(SLATE_50.into()),
        //     ),

            // (
            //     Button,
            //     button_node.clone(),
            //     BackgroundColor(NORMAL_BUTTON),
            //     MenuButtonAction::BackToMainMenu,
            //     children![
            //         (Text::new("Back To Main Menu"), button_text_style.clone())
            //     ]
            // ),
            // (
            //     Button,
            //     button_node,
            //     BackgroundColor(NORMAL_BUTTON),
            //     MenuButtonAction::BackToMainMenu,
            //     children![
            //         (Text::new("Restart"), button_text_style)
            //     ]
            // ),

//----
            // (
            //     Node {
            //         flex_direction: FlexDirection::Row,
            //         align_items: AlignItems::Center,
            //         column_gap: px(10),
            //         ..default()
            //     },
            //     children![
            //         (
            //             Text::new("Press"),
            //             TextFont { 
            //                 font_size: Val::Px(40.).into(), 
            //                 ..default()},
            //             TextColor(SLATE_50.into()),
            //         ),
            //         (
            //             Node {
            //                 width: px(150),
            //                 height: px(65),
            //                 border: UiRect::all(px(5)),
            //                 justify_content: JustifyContent::Center,
            //                 align_items: AlignItems::Center,
            //                 border_radius: BorderRadius::new(Val::Px(10.),Val::Px(10.),Val::Px(10.),Val::Px(10.)),
            //                 ..default()
            //             },
            //             BorderColor::all(Color::WHITE),
            //             BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
            //             children![(
            //                 Text::new("SPACE"),
            //                 TextFont { 
            //                     font_size: Val::Px(40.).into(), 
            //                     ..default()},
            //                 TextColor(SLATE_50.into()),
            //             )],
            //         ),
            //         (
            //             Text::new("to restart!"),
            //             TextFont {font_size: Val::Px(40.).into(),
            //                 ..default()},
            //             TextColor(SLATE_50.into()),
            //         )
            //     ],
            // )
        //]
            
}