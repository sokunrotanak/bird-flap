use bevy::{
    asset::AssetMetaCheck,
    prelude::*,
    sprite_render::Material2dPlugin, 
};

mod audio;
mod world;
mod pipes;
mod player;

use audio::*;
use pipes::*;
use world::*;
use world::menu::*;
use player::*;


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
        .add_plugins(WorldPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(MenuPlugin)
        .add_plugins((
            PipePlugin,
            Material2dPlugin::<BackgroundMaterial>::default(),
        ))
        .add_plugins(PointAudioPlugin)
        // .init_resource::<DebugSettings>()
        .run();
}