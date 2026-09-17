use bevy::{
    app::AppExit,
    audio::Volume,
    color::palettes::{css::*, tailwind::SLATE_50},
    //ecs::component::Mutable, 
    prelude::*,
    ui_widgets::{checkbox_self_update, observe, Checkbox},
    ui::{Checked},
    input_focus::{
        tab_navigation::TabIndex,
    },
    picking::hover::Hovered,
    // feathers::{*, controls::FeathersCheckbox},
};
use crate::{
    audio::{AudioEnabled, GLOBAL_VOLUME, PlayPitch}, player::OneShot, world::*
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
            update_checkbox,
            update_checkbox2,
        ).run_if(|state: Res<State<GameState>>| {matches!(state.get(), GameState::MainMenu | GameState::GameOver)}));
        app.add_systems(Update, sound_setting_button.run_if(in_state(MenuState::SettingsSound)));

    }
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum MenuState {
    Main,
    SettingsSound,
    #[default]
    Disabled,
}

#[derive(Component)]
struct SelectedOption;

#[derive(Component)]
struct OnMainMenuScreen;

#[derive(Component)]
struct OnSoundSettingsMenuScreen;

#[derive(Component, PartialEq, Debug, Clone, Copy)]
pub struct VolumeSetting(pub Volume);

pub const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const HOVERED_PRESSED_BUTTON: Color = Color::srgb(0.25, 0.65, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);
pub const TEXT_COLOR: Color = Color::Srgba(SLATE_50);
const TRANSPARENT_BACKGROUND: Color = Color::srgba(0.5, 0.5, 0.5, 0.5);

#[derive(Component)]
pub enum MenuButtonAction {
    Play,
    SettingsSound,
    JumpMode,
    BackToMainMenu,
    Quit,
}

#[derive(Component)]
pub enum GameMode {
    Adventure,
    Endless,
}

fn button_system (
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

fn sound_setting_button (
    interaction_query: Query<
        (&Interaction, &VolumeSetting, Entity),
        (Changed<Interaction>, With<Button>),
    >,
    selected_query: Single<(Entity, &mut BackgroundColor), With<SelectedOption>>,
    mut global_volume: ResMut<GlobalVolume>,
    mut commands: Commands,
    mut play_pitch_writer: MessageWriter<PlayPitch>,
    mut audio_enabled: ResMut<AudioEnabled>,
) {
    let (previous_button, mut previous_button_color) = selected_query.into_inner();
    for (interaction, volume_setting, entity) in &interaction_query {
        if *interaction == Interaction::Pressed && global_volume.volume != volume_setting.0 {
            *previous_button_color = NORMAL_BUTTON.into();
            commands.entity(previous_button).remove::<SelectedOption>();
            commands.entity(entity).insert(SelectedOption);
            global_volume.volume = volume_setting.0;
            play_pitch_writer.write(PlayPitch);
            audio_enabled.0 = true;
        }
    }
}

fn menu_setup (mut menu_state: ResMut<NextState<MenuState>>) {
    menu_state.set(MenuState::Main);
}

#[derive(Component, Default)]
struct TestCheckBox;

fn main_menu_setup (mut commands: Commands, asset_server: ResMut<AssetServer>) {
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

    let button_text_font = TextFont {
        font_size: FontSize::Px(20.0),
        ..default()
    };

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
                    Node {
                        margin: UiRect::all(px(20)),
                        ..default()
                    },
                    Text::new("Bird Flap"),
                    TextFont {
                        font_size: FontSize::Px(60.0),
                        ..default()
                    },
                    TextColor(Color::from(YELLOW)),

                ),
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
                    button_node.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::JumpMode,
                    checkbox(&asset_server, "OneShot Jump Mode"),
                    observe(checkbox_self_update),
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

fn checkbox(asset_server: &AssetServer, caption: &str) -> impl Bundle {
    (
        // Node {
        //     display: Display::Flex,
        //     flex_direction: FlexDirection::Row,
        //     justify_content: JustifyContent::FlexStart,
        //     align_items: AlignItems::Center,
        //     align_content: AlignContent::Center,
        //     column_gap: px(4),
        //     ..default()
        // },
        Name::new("Checkbox"),
        Hovered::default(),
        TestCheckBox,
        Checkbox,
        Checked,
        TabIndex(0),
        Children::spawn((
            Spawn((
                Node {
                    display: Display::Flex,
                    width: px(16),
                    height: px(16),
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(3)),
                    padding: Val::Px(10.0).into(),
                    ..default()
                },
                BorderColor::all(TEXT_COLOR),
                children![(
                    Node {
                        display: Display::Flex,
                        width: px(16),
                        height: px(16),
                        position_type: PositionType::Absolute,
                        left: px(2),
                        top: px(2),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.35, 0.75, 0.35)),
                )]
            )),
            Spawn ((
                Node {
                    display: Display::Flex,
                    align_items: AlignItems::Center,
                    align_content: AlignContent::Center,
                    justify_content: JustifyContent::Center,
                    width: px(250),
                    height: px(65),
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(3)),
                    padding: Val::Px(10.0).into(),
                    ..default()
                },
                children![(
                    Text::new("OneShot JumpMode"),
                    TextFont { font_size: FontSize::Px(20.0), ..default()}
                )]
            ))
        ))

    )
}


fn update_checkbox (
    mut q_checkbox: Query< 
        (Has<Checked>, &Hovered, &Children),
        (
            With<TestCheckBox>,
            Or<(
                Added<TestCheckBox>,
                Added<Checked>,
                Changed<Hovered>,
            )>,
        ),
    >,
    mut q_border_color: Query<
        (&mut BorderColor, &mut Children),
        Without<TestCheckBox>
    >,
    mut q_bg_color: Query<&mut BackgroundColor, (Without<TestCheckBox>, Without<Children>)>,
    mut oneshot: ResMut<OneShot>,
) {
    for (checked, Hovered(is_hovering), children) in q_checkbox.iter_mut() {
        let Some(border_id) = children.first() else {
            continue;
        };

        let Ok((mut border_color, border_children)) = q_border_color.get_mut(*border_id) else {
            continue;
        };

        let Some(mark_id) = border_children.first() else {
            warn!("Checkbox does not have a mark entity.");
            continue;
        };

        let Ok(mut mark_bg) = q_bg_color.get_mut(*mark_id) else {
            warn!("Checkbox mark entity lacking a background color.");
            continue;
        };

        set_checkbox_style(
            *is_hovering,
            checked,
            &mut border_color,
            &mut mark_bg,
            &mut oneshot.0,
        );
    }
}

fn update_checkbox2 (
    mut q_checkbox: Query<
        (Has<Checked>, &Hovered, &Children),
        With<TestCheckBox>,
    >,
    mut q_border_color: Query<
        (&mut BorderColor, &mut Children),
        Without<TestCheckBox>
    >,
    mut q_bg_color: Query<
        &mut BackgroundColor,
        (Without<TestCheckBox>, Without<Children>),
    >,
    mut removed_checked: RemovedComponents<Checked>,
    mut oneshot: ResMut<OneShot>,
) {
    removed_checked
        .read()
        .for_each (|entity| {
            if let Ok((checked, Hovered(is_hovering), children)) = q_checkbox.get_mut(entity) {
                let Some(border_id) = children.first() else {
                    return;
                };

                let Ok((mut border_color, border_children)) = q_border_color.get_mut(*border_id) else {
                    return;
                };

                let Some(mark_id) = border_children.first() else {
                    warn!("Checkbox does not have a mark entity.");
                    return;
                };

                let Ok(mut mark_bg) = q_bg_color.get_mut(*mark_id) else {
                    warn!("Checkbox mark entity lacking a background color.");
                    return;
                };

                set_checkbox_style(
                    *is_hovering, 
                    checked, 
                    &mut border_color, 
                    &mut mark_bg,
                    &mut oneshot.0,
                );
            }
        })
}


const ELEMENT_OUTLINE: Color = Color::srgb(0.45, 0.45, 0.45);
const ELEMENT_FILL: Color = Color::srgb(0.35, 0.75, 0.35);


fn set_checkbox_style (
    hovering: bool,
    checked: bool,
    border_color: &mut BorderColor,
    mark_bg: &mut BackgroundColor,
    checked_state: &mut bool,
) {
    let color: Color = if hovering {
        // If hovering, use a lighter color
        ELEMENT_OUTLINE.lighter(0.2)
    } else {
        // Default color for the element
        ELEMENT_OUTLINE
    };

    // Update the background color of the element
    border_color.set_all(color);

    let mark_color: Color = match checked {
        true => ELEMENT_FILL,
        false => Srgba::NONE.into(),
    };

    if mark_bg.0 != mark_color {
        // Update the color of the element
        mark_bg.0 = mark_color;
    }

    *checked_state = checked;

}

fn sound_settings_menu_setup(mut commands: Commands, global_volume: Res<GlobalVolume>) {
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

fn menu_action(
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
                    game_state.set(GameState::Pending);
                    menu_state.set(MenuState::Disabled);
                }
                MenuButtonAction::SettingsSound => {
                    menu_state.set(MenuState::SettingsSound);
                }
                MenuButtonAction::JumpMode => {
                    //
                }
                MenuButtonAction::BackToMainMenu => {
                    game_state.set(GameState::MainMenu);
                    menu_state.set(MenuState::Main);
                },
            }
        }
    }
}
