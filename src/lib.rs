use std::time::Duration;
use rand::RngExt;
use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy::prelude::*;
use bevy::audio::Source;
use bevy::audio::{AddAudioSource, Volume};

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    MainMenu,
    Playing,
    GameOver,
}

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

pub const CANVAS_SIZE: Vec2 = Vec2::new(800.0, 600.0);
pub const BIRD_CUSTOM: Vec2 = Vec2::new(128.0, 105.0);
pub const SCALE_FACTOR: f32 = 0.5;
pub const BIRD_SIZE: Vec2 = Vec2::new(BIRD_CUSTOM.x * SCALE_FACTOR, BIRD_CUSTOM.y * SCALE_FACTOR);
pub const PADDING_X: f32 = 20.0;
pub const PADDING_Y: f32 = 20.0;
pub const COLLIDER: Vec2 = Vec2::new(BIRD_SIZE.x - PADDING_X, BIRD_SIZE.y - PADDING_Y);
pub const PIPE_SIZE: Vec2 = Vec2::new(128.0, CANVAS_SIZE.y);
const GAP_SIZE: f32 = 180.0;
pub const PIPE_SPEED: f32 = 200.0;
pub const PIPE_X_OFFSET: f32 = 64.0;


#[derive(Component)]
pub struct Pipe;

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
    // (time.elapsed_secs() * 5. * 5.238924)
    //     .sin()
    //     *CANVAS_SIZE.y
    //     /4.;
    let transform = Transform::from_xyz(CANVAS_SIZE.x /2.0, 0., 1.);
    // let gap_y_position = 0.;
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

pub fn shift_pipes_to_the_left (
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
    fn current_frame_len(&self) -> Option<usize> { None } // older rodio: current_frame_len
    fn channels(&self) -> u16 { 1 }
    fn sample_rate(&self) -> u32 { self.sample_rate }
    fn total_duration(&self) -> Option<Duration> { None }
}

impl Decodable for EnvelopedPitch {
    type DecoderItem = f32;
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

// impl Plugin for PointAudioPlugin {
//     fn build(&self, app: &mut App) {
//         app.add_message::<PlayPitch>();
//         app.add_systems(Startup, setup);
//         app.add_systems(Update, (play_pitch, play_delayed_pitch.after(play_pitch), keyboard_input_system));
//         app.add_audio_source::<EnvelopedPitch>();
//         app.insert_resource(GlobalVolume::new(Volume::Linear(0.2)));
//     }
// }

#[derive(Resource)]
pub struct AudioEnabled(pub bool);

#[derive(Component)]
pub struct AudioToggleButton;

#[derive(Component)]
pub struct AudioButtonLabel;

#[derive(Message, Default)]
pub struct PlayPitch;

#[derive(Resource)]
pub struct PitchFrequency(f32);

#[derive(Resource)]
pub struct PitchFrequency2(f32);

#[derive(Component)]
pub struct PendingPitch(Timer);


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
    mut q: Query<(Entity, &mut PendingPitch)>,
) {
    for (entity, mut pending) in &mut q {
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
