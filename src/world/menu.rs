use bevy::{
    app::AppExit, audio::Volume, color::palettes::{css::*, tailwind::SLATE_50}, input::keyboard::{self, *}, input_focus::{AutoFocus, tab_navigation::TabIndex}, picking::hover::Hovered, prelude::*, text::{EditableText, EditableTextFilter, TextCursorStyle}, ui::{Checked, FocusPolicy}, ui_widgets::{Checkbox, checkbox_self_update, observe}
};
use chrono::{DateTime, Local};
use crate::{
    audio::{AudioEnabled, GLOBAL_VOLUME, PlayPitch}, 
    player::OneShot, world::*,
    scoreboard::*,
};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(OnEnter(GameState::MainMenu), (
        //     instruction,//.run_if(in_state(GameState::MainMenu)),
        // ));
        app.init_state::<MenuState>();
        app.init_resource::<ScoreSubmittedThisRun>();
        app.add_systems(OnEnter(GameState::MainMenu), menu_setup);
        app.add_systems(OnEnter(MenuState::Main), main_menu_setup);
        app.add_systems(OnEnter(MenuState::SettingsSound), sound_settings_menu_setup);
        app.add_systems(OnEnter(OverlayState::Scoreboard), start_fetch);
        app.add_systems(OnEnter(OverlayState::Nickname), nickname_ui);
        app.add_systems(Update,(
            menu_action,
            button_system,
            update_checkbox,
            update_checkbox2,
        ).run_if(|state: Res<State<GameState>>| {matches!(state.get(), GameState::MainMenu | GameState::GameOver)}));
        app.add_systems(
            Update, 
            (
                sound_setting_button.run_if(in_state(MenuState::SettingsSound)),
                refresh_row.run_if(in_state(OverlayState::Scoreboard)),
                // nickname_input.run_if(in_state(OverlayState::Nickname)),
                update_submit_status.run_if(in_state(OverlayState::Nickname).and_then(resource_changed::<SubmitStatus>)),
                enter_submit.run_if(in_state(OverlayState::Nickname)),
        ));
        app.init_state::<OverlayState>();
        app.add_observer(on_submit);
        app.add_observer(on_submittted);
        app.add_systems(OnEnter(GameState::Pending),  reset_submission_check);
        
    }
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum MenuState {
    Main,
    // Scoreboard,
    SettingsSound,
    #[default]
    Disabled,
}

#[derive(Component)]
struct ScoreRows;

#[derive(Component)]
struct SelectedOption;

#[derive(Component)]
struct OnMainMenuScreen;

#[derive(Component)]
struct OnSoundSettingsMenuScreen;

#[derive(Component, PartialEq, Debug, Clone, Copy)]
pub struct VolumeSetting(pub Volume);

#[derive(States, Default, Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum OverlayState {
    #[default]
    None,
    Scoreboard,
    Nickname,
}

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
    SubmitScore,
    SubmitScoreUI,
    Scoreboard,
    ExitOverlay,
    Quit,
}

#[derive(Component, Default)]
pub enum GameMode {
    #[default]
    OneShot,
    DisabledOneShot,
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
        height: px(50),
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
                    MenuButtonAction::Scoreboard,
                    children![
                        (
                            Text::new("Scoreboard"),
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
                    Text::new(caption),
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
    mut commands: Commands,
    mut app_exit_writer: MessageWriter<AppExit>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut overlay_state: ResMut<NextState<OverlayState>>,
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
                MenuButtonAction::Scoreboard => {
                    //
                    overlay_state.set(OverlayState::Scoreboard);
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
                    overlay_state.set(OverlayState::None)
                },
                MenuButtonAction::SubmitScoreUI => {
                    //
                    overlay_state.set(OverlayState::Nickname);
                },
                MenuButtonAction::ExitOverlay => {
                    overlay_state.set(OverlayState::None);
                }
                MenuButtonAction::SubmitScore => {
                    commands.trigger(SubmitNickname);
                }
            }
        }
    }
}

fn start_fetch (
    mut commands: Commands,
    mut scoreboard: ResMut<ScoreboardLog>,
    channel: Res<ScoreboardChannel>,
) {
    scoreboard.status = LoadStatus::Loading;
    let tx = channel.tx.clone();
    trigger_fetch(tx);
    overlay_scoreboard(&mut commands);
}

const COL_RANK: Val = Val::Px(50.0);
const COL_NAME: Val = Val::Px(200.0);
const COL_SCORE: Val = Val::Px(100.0);
const COL_GAME_MODE: Val = Val::Px(200.0);
const COL_SUBMIT_TIME: Val = Val::Px(200.0);

fn cell (
    p: &mut ChildSpawnerCommands,
    text: impl Into<String>, width: Val, align: Justify
) {
    p.spawn((
        Node {width, ..default() },
        children![(
            Text::new(text),
            TextFont { font_size: Val::Px(16.0).into(), ..default() },
            TextColor(TEXT_COLOR),
            TextLayout::justify(align),
        )],
    ));
}

fn score_row (
    p: &mut ChildSpawnerCommands,
    rank: &str,
    name: &str,
    score: &str,
    // game_mode: &str,
    submitted_at: &str,
    bg: Color,
) {
    p.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(12),
            padding: UiRect::axes(px(12), px(4)),
            ..default()
        },
        BackgroundColor(bg),
    ))
    .with_children(|r| {
        cell(r, rank, COL_RANK, Justify::Left);
        cell(r, name, COL_NAME, Justify::Left);
        cell(r, score, COL_SCORE, Justify::Right);
        // cell(r, game_mode, COL_GAME_MODE, Justify::Center);
        cell(r, submitted_at, COL_SUBMIT_TIME, Justify::Center);
    });
}

#[derive(Component)]
struct OnScoreboardScreen;

fn overlay_scoreboard (
    commands: &mut Commands,
) {
    let button_node = Node {
        width: px(100),
        height: px(40),
        margin: UiRect::all(px(10)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = (
        TextFont {
            font_size: FontSize::Px(18.0),
            ..default()
        },
        TextColor(TEXT_COLOR),
    );
    commands.spawn((
        DespawnOnExit(OverlayState::Scoreboard),
        OnScoreboardScreen,
        FocusPolicy::Block,
        Node {
            width: percent(100), 
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        GlobalZIndex(10),
        BackgroundColor(TRANSPARENT_BACKGROUND),
    )).with_children(|root| {
        root.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(20)),
                row_gap: px(4),
                border_radius: BorderRadius::all(px(12)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 1.)),
        )).with_children(|panel| {
            panel.spawn((
                Text::new("Scoreboard"), 
                TextFont { font_size: Val::Px(24.0).into(), ..default() }, 
                TextColor(TEXT_COLOR)
            ));
            score_row(panel, "#", "Name", "Score", "Submit Time", Color::NONE);
            panel.spawn((
                ScoreRows,
                Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                }
            ));
            panel.spawn(
                (
                    Button,
                    button_node,
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::ExitOverlay,
                    children![(Text::new("Back"), button_text_style)]
                ),
            );
        });

    });
}

fn refresh_row (
    board: Res<ScoreboardLog>,
    container: Single<Entity, With<ScoreRows>>,
    mut commands: Commands,
){
    if !board.is_changed() { return; }
    commands.entity(*container).despawn_children();
    commands.entity(*container).with_children(|p| {
        match &board.status {
            LoadStatus::NotLoaded => {}
            LoadStatus::Loading => { 
                p.spawn((
                    Text::new("Loading..."),
                    TextColor(TEXT_COLOR)

                ));
                // board.rows.clear();
            }
            LoadStatus::Failed(e) => {
                p.spawn(
                    Text::new(format!("Couldn't load: {e}")),
                );
            },
            LoadStatus::Loaded => for (i, r) in board.rows.iter().enumerate() {
                let datetime = DateTime::parse_from_rfc3339(&r.submitted_at).expect("Failed to parse RFC3339 date");
                let local_datetime: DateTime<Local> = DateTime::from(datetime);
                let custom_format = local_datetime.format("%Y-%m-%d %H:%M");
                let bg = if Some(r.id) == board.last_submitted_id{ Color::srgba(1.0, 0.84, 0., 1.)}
                else if i % 2 == 0 { Color::srgba(1.,1.,1., 0.05) } else { Color::NONE};
                score_row(p, &(i+1).to_string(), &r.nickname, &r.score.to_string(), &custom_format.to_string(), bg);
                },
            }
        });
}

#[derive(Event)]
struct SubmitNickname;

#[derive(Resource, Default)]
pub struct ScoreSubmittedThisRun(pub bool);

fn reset_submission_check (
    mut submission_check: ResMut<ScoreSubmittedThisRun>,
) {
    submission_check.0 = false;
}

const MODE_ONESHOT: &str = "OneShot";
const MODE_DISABLE: &str = "Disabled OneShot";

fn on_submit (
    _: On<SubmitNickname>, 
    input: Single<&EditableText, With<NicknameInput>>,
    mut status: ResMut<SubmitStatus>,
    score: Res<Score>,
    channel: Res<ScoreboardChannel>,
    mode: Res<OneShot>,
) {

    if matches!(*status, SubmitStatus::Submitting) {return;} 

    let name = input.value().to_string().trim().to_owned();
    if name.is_empty() { 
        *status = SubmitStatus::Failed("Enter a name".into()); 
        return;
    }
    let jump_mode = if mode.0 { MODE_ONESHOT } else { MODE_DISABLE }.to_string();
    //     OneShot(true) => {&"OneShot".to_string()},
    //     OneShot(false) => {&"Disabled Oneshot".to_string()}, 
    // };

    *status = SubmitStatus::Submitting;
    request_submit(
        channel.tx.clone(), 
        SubmitScore {
            nickname: name, 
            score: score.0, 
            game_mode: jump_mode,
        }
    )

}

fn on_submittted (
    submitted_score: On<ScoreSubmitted>,
    mut board: ResMut<ScoreboardLog>,
    mut status: ResMut<SubmitStatus>,
    mut overlay: ResMut<NextState<OverlayState>>,
    mut submission_check: ResMut<ScoreSubmittedThisRun>,
    mut commands: Commands,
    score_button: Query<Entity, With<EnterNickNameButton>>,
) {
    board.last_submitted_id = Some(submitted_score.id);
    *status = SubmitStatus::Editing;
    overlay.set(OverlayState::Scoreboard);
    submission_check.0 = true;
    for e in &score_button {
        commands.entity(e).despawn();
    }
    
}

fn update_submit_status(
    status: Res<SubmitStatus>,
    mut q: Single<&mut Text, With<SubmitStatusText>>,
) {
    q.0 = match &*status {
        SubmitStatus::Editing => String::new(),
        SubmitStatus::Submitting => "Submitting...".into(),
        SubmitStatus::Failed(e) => e.clone(),
    };
}


#[derive(Component)]
pub struct NicknameInput;

#[derive(Component)]
pub struct SubmitScoreButton;

#[derive(Component)]
pub struct EnterNickNameButton;

#[derive(Component)]
pub struct SubmitStatusText;

fn nickname_ui (
    mut commands: Commands,
    mut status: ResMut<SubmitStatus>,
) {
    let button_node = Node {
        width: px(150),
        height: px(40),
        margin: UiRect::all(px(10)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border_radius: BorderRadius::all(px(8)),
        ..default()
    };
    let button_text_style = (
        TextFont {
            font_size: FontSize::Px(13.0),
            ..default()
        },
        TextColor(TEXT_COLOR),
    );
    commands.spawn((
        DespawnOnExit(OverlayState::Nickname),
        FocusPolicy::Block,
        Node {
            flex_direction: FlexDirection::Column,
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            align_content: AlignContent::Center,
            ..default()
        },
        GlobalZIndex(10),
        BackgroundColor(TRANSPARENT_BACKGROUND),
    )).with_children(|parent| {
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(px(5)),
                border_radius: BorderRadius::all(px(8)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 1.)),
        )).with_children(|panel| {
            panel.spawn((
                Text::new("Your nickname"),
                TextFont { font_size: Val::Px(24.0).into(), ..default()},
                TextColor(TEXT_COLOR)
            ));
            panel.spawn((
                NicknameInput,
                Node {
                    width: px(300),
                    border: UiRect::all(px(2.)),
                    padding: UiRect::all(px(10.)),
                    ..default()
                },
                EditableText { 
                    max_characters: Some(25),
                    ..default()
                },
                EditableTextFilter::new(|c| c.is_alphanumeric() || c =='_' || c=='-'),
                TextCursorStyle::default(),
                TextFont::from_font_size(24.),
                BorderColor::all(TEXT_COLOR),
                AutoFocus,
            ));
            panel.spawn((
                SubmitStatusText,
                Node {
                    width: px(300),
                    ..default()
                },
                Text::new(""),
                TextColor(Color::srgb(1., 0., 0.)),
            ));
            panel.spawn(
                Node {
                    width: percent(100),
                    height: percent(100),
                    align_items: AlignItems::Center,
                    align_content: AlignContent::Center,
                    justify_content: JustifyContent::SpaceAround,
                    flex_direction: FlexDirection::Row,
                    ..default()
                }
            ).with_children(|button|{
                button.spawn((
                    Button,
                    SubmitScoreButton,
                    button_node.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::SubmitScore,
                    children![(Text::new("Submit"), button_text_style.clone())]
                ));
                button.spawn((
                    Button,
                    button_node,
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::ExitOverlay,
                    children![(Text::new("Cancel"), button_text_style)]
                ));
            });
        });
    });
    if *status != SubmitStatus::Submitting {
        *status = SubmitStatus::Editing;
    }

}

fn enter_submit (
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    if keys.just_pressed(KeyCode::Enter) {
        commands.trigger(SubmitNickname);
    }
}