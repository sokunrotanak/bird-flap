use bevy::{
    prelude::*,
    color::palettes::tailwind::SLATE_50,
};
use crate::world::*;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), (
            instruction,//.run_if(in_state(GameState::MainMenu)),
        ));
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
                        font_size: Val::Px(40.).into(),
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
                            font_size: Val::Px(30.).into(),
                            ..default()
                        },
                        TextColor(SLATE_50.into()),
                    )]
                ),
                (
                    Text::new(" to Start."),
                    TextFont {
                        font_size: Val::Px(40.).into(),
                        ..default()
                    },
                    TextColor(SLATE_50.into()),
                )
            ]
        )]
    )
}