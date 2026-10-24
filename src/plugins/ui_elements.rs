use crate::plugins::window::{CustomUiData, EasyClosablePopup, Popup, NORMAL_BUTTON, SLIDER_TRACK, SLIDER_THUMB, PopupCloseButton};
use bevy::color::Color;
use bevy::ecs::children;
use bevy::math::Vec2;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::{observe, Slider, SliderRange, SliderValue, TrackClick, slider_self_update, SliderThumb};

pub fn button_bundle(text: String) -> impl Bundle {
    (
        Button,
        Node {
            padding: UiRect::axes(px(10.0), px(6.0)),
            border: UiRect::all(px(2.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(px(6.0)),
            ..default()
        },
        BorderColor::all(Color::WHITE),
        BackgroundColor(NORMAL_BUTTON),
        children![(
            Text::new(&text),
            TextFont {
                font_size: FontSize::Px(13.0),
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            TextShadow::default(),
        )],
        CustomUiData { name: text },
        Visibility::Visible,
    )
}

pub fn slider_bundle(text: String, min: f32, max: f32, initial: f32) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(4.0),
            ..default()
        },
        children![
            (
                Text::new(text.clone()),
                TextFont {
                    font_size: FontSize::Px(13.0),
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                TextShadow::default(),
                CustomUiData { name: text.clone() },
            ),
            (
                Slider {
                    track_click: TrackClick::Snap,
                    ..default()
                },
                SliderValue(initial),
                SliderRange::new(min, max),
                Node {
                    width: px(200.0),
                    height: px(24.0),
                    padding: UiRect::all(px(2.0)),
                    border: UiRect::all(px(2.0)),
                    ..default()
                },
                BorderColor::all(Color::WHITE),
                BackgroundColor(SLIDER_TRACK),
                CustomUiData { name: text },
                Hovered::default(),
                observe(slider_self_update),
                children![(
                    SliderThumb,
                    Node {
                        width: px(16.0),
                        height: px(20.0),
                        position_type: PositionType::Absolute,
                        left: px(0.0),
                        top: px(0.0),
                        ..default()
                    },
                    BackgroundColor(SLIDER_THUMB),
                )],
            ),
        ],
    )
}

pub fn help_popup(message: String) -> impl Bundle {
    (
        Popup,
        GlobalZIndex(100),
        Node {
            position_type: PositionType::Absolute,
            width: percent(100.0),
            height: percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        children![(
            // сама панель
            Node {
                width: px(360.0),
                padding: UiRect::all(px(16.0)),
                border: UiRect::all(px(2.0)),
                flex_direction: FlexDirection::Column,
                row_gap: px(12.0),
                border_radius: BorderRadius::all(px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
            BorderColor::all(Color::WHITE),
            children![
                close_button(),
                // текст попапа (переносится по ширине панели сам)
                (
                    Text::new(message),
                    TextFont {
                        font_size: FontSize::Px(14.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                ),
            ],
        )],
    )
}

pub fn close_button() -> impl Bundle {
    // строка с крестиком, прижатым вправо
    (
        Node {
            width: percent(100.0),
            justify_content: JustifyContent::FlexEnd,
            ..default()
        },
        children![(
            PopupCloseButton,
            Button,
            Node {
                width: px(26.0),
                height: px(26.0),
                border: UiRect::all(px(2.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(px(4.0)),
                ..default()
            },
            BackgroundColor(NORMAL_BUTTON),
            BorderColor::all(Color::WHITE),
            children![(
                Text::new("X"),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            )],
        )],
    )
}

pub fn selection_popup(xy: Vec2) -> impl Bundle {
    (
        EasyClosablePopup,
        GlobalZIndex(100),
        // сама панель
        Node {
            width: px(360.0),
            padding: UiRect::all(px(16.0)),
            border: UiRect::all(px(2.0)),
            flex_direction: FlexDirection::Column,
            row_gap: px(12.0),
            border_radius: BorderRadius::all(px(8.0)),
            left: Val::Px(xy.x),
            top: Val::Px(xy.y),
            ..default()
        },
        BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
        BorderColor::all(Color::WHITE),
        children![
            button_bundle(String::from("Copy")),
            button_bundle(String::from("Paste")),
            button_bundle(String::from("Save Blueprint"))
        ],
    )
}

pub fn enter_name_popup() -> impl Bundle {
    (
        Popup,
        GlobalZIndex(100),
        Node {
            position_type: PositionType::Absolute,
            width: percent(100.0),
            height: percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
        BorderColor::all(Color::WHITE),
        children![
            close_button(),

            button_bundle(String::from("Confirm")),],
    )
}