use bevy::{
    app::AppExit,
    audio::Volume,
    color::palettes::{css::*, tailwind::SLATE_50},
   ecs::component::Mutable, 
    prelude::*,
//    ui_widgets::MenuButton,
};
use crate::{
    world::*,
    audio::{GLOBAL_VOLUME,PlayPitch}
};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(OnEnter(GameState::MainMenu), (
        //     instruction,//.run_if(in_state(GameState::MainMenu)),
        // ));
        app.init_state::<MenuState>();
        app.add_systems(OnEnter(GameState::MainMenu), menu_setup);
        app.add_systems(OnEnter(MenuState::Main), main_menu_setup);
        app.add_systems(OnEnter(MenuState::SettingsSound), sound_settings_menu_setup);
        app.add_systems(Update,(
            menu_action,
            button_system,
        ).run_if(|state: Res<State<GameState>>| {matches!(state.get(), GameState::MainMenu | GameState::GameOver)}));
        app.add_systems(Update, sound_setting_button.run_if(in_state(MenuState::SettingsSound)));

    }
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum MenuState {
    Main,
    SettingsSound,
    #[default]
    Disabled,
}

#[derive(Component)]
pub struct SelectedOption;

#[derive(Component)]
pub struct OnMainMenuScreen;

#[derive(Component)]
pub struct OnSoundSettingsMenuScreen;

#[derive(Component, PartialEq, Debug, Clone, Copy)]
pub struct VolumeSetting(pub Volume);

pub const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
pub const HOVERED_PRESSED_BUTTON: Color = Color::srgb(0.25, 0.65, 0.25);
pub const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);
pub const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
pub const TRANSPARENT_BACKGROUND: Color = Color::srgba(0.5, 0.5, 0.5, 0.5);

#[derive(Component)]
pub enum MenuButtonAction {
    Play,
    SettingsSound,
    BackToMainMenu,
    Quit,
}

pub fn button_system (
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&SelectedOption>),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut bacground_color, selected) in &mut interaction_query {
        *bacground_color = match (*interaction, selected) {
            (Interaction::Pressed, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
            (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
            (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
            (Interaction::None, None) => NORMAL_BUTTON.into(),
        }
    }
}

pub fn sound_setting_button (
    interaction_query: Query<
        (&Interaction, &VolumeSetting, Entity),
        (Changed<Interaction>, With<Button>),
    >,
    selected_query: Single<(Entity, &mut BackgroundColor), With<SelectedOption>>,
    mut global_volume: ResMut<GlobalVolume>,
    mut commands: Commands,
    mut play_pitch_writer: MessageWriter<PlayPitch>,
) {
    let (previous_button, mut previous_button_color) = selected_query.into_inner();
    for (interaction, volume_setting, entity) in &interaction_query {
        if *interaction == Interaction::Pressed && global_volume.volume != volume_setting.0 {
            *previous_button_color = NORMAL_BUTTON.into();
            commands.entity(previous_button).remove::<SelectedOption>();
            commands.entity(entity).insert(SelectedOption);
            global_volume.volume = volume_setting.0;
            play_pitch_writer.write(PlayPitch);
        }
    }
}



pub fn menu_setup (mut menu_state: ResMut<NextState<MenuState>>) {
    menu_state.set(MenuState::Main);
}

pub fn main_menu_setup (mut commands: Commands, asset_server: Res<AssetServer>) {
    let button_node = Node {
        width: px(300),
        height: px(65),
        margin: UiRect::all(px(20)),
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
    // let button_icon_node = Node {
    //     width: px(30),
    //     position_type: PositionType::Absolute,
    //     left: px(10),
    //     ..default()
    // };
    let button_text_font = TextFont {
        font_size: FontSize::Px(20.0),
        ..default()
    };
    // let right_icon = asset_server.load("icons/right.png");
    // let sound_icon = asset_server.load("icons/sound_on.png");
    // let exit_icon = asset_server.load("icons/exitRight.png");

    commands.spawn((
        DespawnOnExit(MenuState::Main),
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        OnMainMenuScreen,
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(TRANSPARENT_BACKGROUND),
            children![
                // Display the game name
                (
                    Text::new("Bird Flap"),
                    TextFont {
                        font_size: FontSize::Px(67.0),
                        ..default()
                    },
                    TextColor(Color::from(YELLOW)),
                    Node {
                        margin: UiRect::all(px(50)),
                        ..default()
                    },
                ),
                // Display three buttons for each action available
                (
                    Button,
                    button_node.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::Play,
                    children![
                        (
                            Text::new("New Game"),
                            button_text_font.clone(),
                            TextColor(TEXT_COLOR),
                        )
                    ]
                ),
                (
                    Button,
                    button_node.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::SettingsSound,
                    children![
                        (
                            Text::new("Sound Settings"),
                            button_text_font.clone(),
                            TextColor(TEXT_COLOR),
                        ),
                    ]
                ),
                (
                    Button,
                    button_node,
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::Quit,
                    children![
                        (Text::new("Quit"), button_text_font, TextColor(TEXT_COLOR),),
                    ]
                ),
            ]
        )]
    ));
}

pub fn sound_settings_menu_setup(mut commands: Commands, global_volume: Res<GlobalVolume>) {
    let button_node = Node {
        width: px(200),
        height: px(65),
        margin: UiRect::all(px(10)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = (
        TextFont {
            font_size: FontSize::Px(30.0),
            ..default()
        },
        TextColor(TEXT_COLOR),
    );

    let global_volume = *global_volume;
    let button_node_clone = button_node.clone();
    commands.spawn((
        DespawnOnExit(MenuState::SettingsSound),
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        OnSoundSettingsMenuScreen,
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(TRANSPARENT_BACKGROUND),
            children![
                (
                    Node {
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    BackgroundColor(TRANSPARENT_BACKGROUND),
                    Children::spawn((
                        Spawn((Text::new("Volume"), button_text_style.clone())),
                        SpawnWith(move |parent: &mut ChildSpawner| {
                            for volume_setting in [0.0, 0.04, 0.08, 0.2, 0.4, 0.6, 0.8, 1.0] {
                                let mut entity = parent.spawn ((
                                    Button,
                                    Node {
                                        width: px(20),
                                        height: px(30),
                                        padding: Val::Px(10.0).into(),
                                        ..button_node_clone.clone()
                                    },
                                    BackgroundColor(TRANSPARENT_BACKGROUND),
                                    VolumeSetting(Volume::Linear(volume_setting as f32 * GLOBAL_VOLUME)),
                                ));
                                if global_volume.volume == Volume::Linear(volume_setting as f32 * GLOBAL_VOLUME) {
                                    entity.insert(SelectedOption);
                                }
                            }
                        })
                    ))
                ),
                (
                    Button,
                    button_node,
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::BackToMainMenu,
                    children![(Text::new("Back"), button_text_style)]
                )
            ]
        )],
    ));
}

pub fn menu_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_exit_writer: MessageWriter<AppExit>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MenuButtonAction::Quit => {
                    app_exit_writer.write(AppExit::Success);
                }
                MenuButtonAction::Play => {
                    game_state.set(GameState::Playing);
                    menu_state.set(MenuState::Disabled);
                }
                MenuButtonAction::SettingsSound => {
                    menu_state.set(MenuState::SettingsSound);
                }
                MenuButtonAction::BackToMainMenu => {
                    game_state.set(GameState::MainMenu);
                    menu_state.set(MenuState::Main);
                },
            }
        }
    }
}

// pub fn instruction (
//     mut commands: Commands,
// ) {
//     commands.spawn((
//         Visibility::Visible,
//         instruction_text(),
//         DespawnOnExit(GameState::MainMenu)
//     ));
// }

// fn instruction_text () -> impl Bundle {
//     (
//         Node {
//             width: percent(100),
//             height: percent(100),
//             align_items: AlignItems::Center,
//             justify_content: JustifyContent::Center,
//             ..default() 
//         },
//         children![(
//             Node {
//                 width: percent(100),
//                 height: px(100),
//                 border: UiRect::all(px(5)),
//                 // horizontally center child text
//                 justify_content: JustifyContent::Center,
//                 // vertically center child text
//                 align_items: AlignItems::Center,
//                 border_radius: BorderRadius::MAX,
//                 ..default()
//             },
//             children![
//                 (
//                     Text::new("Press "),
//                     TextFont {
//                         font_size: Val::Px(40.).into(),
//                         ..default()
//                     },
//                     TextColor(SLATE_50.into()),
//                 ),
//                 (
//                     Node {
//                         width: px(200),
//                         height: px(65),
//                         border: UiRect::all(px(5)),
//                         // horizontally center child text
//                         justify_content: JustifyContent::Center,
//                         // vertically center child text
//                         align_items: AlignItems::Center,
//                         border_radius: BorderRadius::new(Val::Px(10.),Val::Px(10.),Val::Px(10.),Val::Px(10.)),
//                         ..default()
//                     },
//                     BorderColor::all(Color::WHITE),
//                     BackgroundColor(Color::srgb(0.5,0.5,0.5)),
//                     children![(
//                         Text::new("SPACE"),
//                         TextFont {
//                             font_size: Val::Px(30.).into(),
//                             ..default()
//                         },
//                         TextColor(SLATE_50.into()),
//                     )]
//                 ),
//                 (
//                     Text::new(" to Start."),
//                     TextFont {
//                         font_size: Val::Px(40.).into(),
//                         ..default()
//                     },
//                     TextColor(SLATE_50.into()),
//                 )
//             ]
//         )]
//     )
// }