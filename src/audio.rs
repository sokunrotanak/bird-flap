use bevy::prelude::*;
use bevy::audio::Volume;
use std::time::Duration;
use crate::world::{*, menu::*};

pub const GLOBAL_VOLUME: f32 = 0.4;

pub struct PointAudioPlugin;

impl Plugin for PointAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlayPitch>();
        app.add_systems(Startup, (
            setup_audio,
            // spawn_mute_button.run_if(in_state(GameState::Playing)),
        ));
        app.add_systems(OnEnter(GameState::Pending), spawn_mute_button);
        app.add_systems(Update, (
            play_pitch, 
            play_delayed_pitch.after(play_pitch),
            mute_audio.run_if(check_pending_playing),
        ));
        app.insert_resource(AudioEnabled(true));
        app.insert_resource(GlobalVolume::new(Volume::Linear(0.4 * GLOBAL_VOLUME)));
    }
}

#[derive(Resource)]
pub struct AudioEnabled(pub bool);

#[derive(Component)]
struct AudioToggleButton;

#[derive(Message, Default)]
pub struct PlayPitch;

#[derive(Resource)]
struct PitchFrequency(f32);

#[derive(Resource)]
struct PitchFrequency2(f32);

#[derive(Component)]
struct PendingPitch(Timer);

fn check_pending_playing (
    state: Res<State<GameState>>
) -> bool {
    *state.get() == GameState::Pending || *state.get() == GameState::Playing
}

fn setup_audio(mut commands: Commands) {
    commands.insert_resource(PitchFrequency(1000.0));
    commands.insert_resource(PitchFrequency2(1370.0));
}

fn play_pitch(
    mut pitch_assets: ResMut<Assets<Pitch>>,
    frequency: Res<PitchFrequency>,
    mut play_pitch_reader: MessageReader<PlayPitch>,
    mut commands: Commands,
    audio_enabled: Res<AudioEnabled>,
) {

    for _ in play_pitch_reader.read() {
        if !audio_enabled.0 {
            continue;
        }
        commands.spawn((
            AudioPlayer(pitch_assets.add(Pitch::new(frequency.0, Duration::from_millis(80)))),
            PlaybackSettings::DESPAWN,
            DespawnOnExit(GameState::Playing)
        ));
        commands.spawn(PendingPitch(Timer::new(Duration::from_millis(80), TimerMode::Once)));
    }
}

fn play_delayed_pitch(
    time: Res<Time>,
    mut pitch_assets: ResMut<Assets<Pitch>>,
    frequency2: Res<PitchFrequency2>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut PendingPitch)>,
) {
    for (entity, mut pending) in &mut query {
        if pending.0.tick(time.delta()).just_finished() {
            commands.spawn((
                AudioPlayer(pitch_assets.add(Pitch::new(frequency2.0, Duration::from_millis(300)))),
                PlaybackSettings::DESPAWN,
                DespawnOnExit(GameState::Playing)
            ));
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Resource)]
struct AudioIcons {
    on: Handle<Image>,
    off: Handle<Image>,
}

fn spawn_mute_button (
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    global_volume: Res<GlobalVolume>,
    audio_enabled: Res<AudioEnabled>,
) {
    let icons = AudioIcons {
        on: asset_server.load("icons/sound_on.png"),
        off: asset_server.load("icons/sound_off.png"),
    };
    commands.spawn((
        Button,
        VolumeSetting(global_volume.volume),
        AudioToggleButton,
        DespawnOnExit(GameState::Playing),
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
        ImageNode::new(if audio_enabled.0 && !(global_volume.volume == Volume::Linear(0.)) { icons.on.clone() } else { icons.off.clone() }),
        BackgroundColor(Color::NONE),
    ));
    commands.insert_resource(icons);
}

fn mute_audio(
    mut interaction: Query<
        (&Interaction, &mut ImageNode, &ComputedNode, &VolumeSetting, &UiGlobalTransform),
        (Changed<Interaction>, With<AudioToggleButton>),
    >,
    mut audio_enabled: ResMut<AudioEnabled>,
    mut global_volume: ResMut<GlobalVolume>,
    icons: Res<AudioIcons>,
    touches: Res<Touches>,
) {
    for (interact, mut image, _, volume_setting, &global_pos) in &mut interaction {
        if *interact == Interaction::Pressed {
            audio_enabled.0 = !audio_enabled.0;
            global_volume.volume = if audio_enabled.0 {
                volume_setting.0 
            } else {
                Volume::Linear(0.)
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