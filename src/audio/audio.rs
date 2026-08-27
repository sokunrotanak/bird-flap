use bevy::prelude::*;
use bevy::audio::{AddAudioSource, Volume};
use std::time::Duration;
use std::num::*;
use bevy::audio::Source;
use crate::world::*;

pub const GLOBAL_VOLUME: f32 = 0.4;

pub struct PointAudioPlugin;

impl Plugin for PointAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlayPitch>();
        app.add_systems(Startup, (
            setup_audio,
            // spawn_mute_button.run_if(in_state(GameState::Playing)),
        ));
        app.add_systems(OnEnter(GameState::Playing), spawn_mute_button);
        app.add_systems(Update, (
            play_pitch, 
            play_delayed_pitch.after(play_pitch),
            mute_audio.run_if(in_state(GameState::Playing)),
        ));
        app.insert_resource(AudioEnabled(true));
        app.add_audio_source::<EnvelopedPitch>();
        // app.insert_resource(Volume(2));
        app.insert_resource(GlobalVolume::new(Volume::Linear(0.4 * GLOBAL_VOLUME)));
    }
}

#[derive(Asset, TypePath)]
pub struct EnvelopedPitch {
    pub frequency: f32,
    pub duration: Duration,
}

pub struct EnvelopedSine {
    freq: f32,
    sample_rate: u32,
    num_samples: usize,
    current: usize,
    fade_samples: usize,
}

#[derive(Resource)]
pub struct AudioEnabled(pub bool);

#[derive(Component)]
pub struct AudioToggleButton;

#[derive(Message, Default)]
pub struct PlayPitch;

#[derive(Resource)]
pub struct PitchFrequency(f32);

#[derive(Resource)]
pub struct PitchFrequency2(f32);

#[derive(Component)]
pub struct PendingPitch(Timer);


impl Iterator for EnvelopedSine {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if self.current >= self.num_samples {
            return None;
        }
        let t = self.current as f32 / self.sample_rate as f32;
        let mut v = (t * self.freq * std::f32::consts::TAU).sin();

        // fade in
        if self.current < self.fade_samples {
            v *= self.current as f32 / self.fade_samples as f32;
        }
        // fade out
        let remaining = self.num_samples - self.current;
        if remaining < self.fade_samples {
            v *= (remaining as f32 -1.0).max(0.0) / self.fade_samples as f32;
        }

        self.current += 1;
        Some(v * 0.5) // headroom so it isn't clipping-loud
    }
}

impl Source for EnvelopedSine {
    fn current_span_len(&self) -> Option<usize> { None } // older rodio: current_frame_len
    fn channels(&self) -> NonZero<u16> { NonZero::new(1).unwrap() }
    fn sample_rate(&self) -> NonZero<u32> { NonZero::new(self.sample_rate).unwrap() }
    fn total_duration(&self) -> Option<Duration> { None }
}

impl Decodable for EnvelopedPitch {
    //type DecoderItem = f32;
    type Decoder = EnvelopedSine;

    fn decoder(&self) -> Self::Decoder {
        let sample_rate = 44_100;
        let num_samples =
            (self.duration.as_secs_f32() * sample_rate as f32) as usize;
        EnvelopedSine {
            freq: self.frequency,
            sample_rate,
            num_samples,
            current: 0,
            fade_samples: (sample_rate as f32 * 0.015) as usize, // 15ms ramp
        }
    }
}


pub fn setup_audio(mut commands: Commands) {
    commands.insert_resource(PitchFrequency(1000.0));
    commands.insert_resource(PitchFrequency2(1370.0));
}

pub fn play_pitch(
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

pub fn play_delayed_pitch(
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
pub struct AudioIcons {
    on: Handle<Image>,
    off: Handle<Image>,
}

pub fn spawn_mute_button (
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    global_volume: Res<GlobalVolume>,
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
        ImageNode::new(icons.on.clone()),
        BackgroundColor(Color::NONE),
    ));
    commands.insert_resource(icons);
}

pub fn mute_audio(
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