use std::ops::RangeInclusive;

use bevy::{ecs::system::Resource, prelude::*};
use bevy_kira_audio::{Audio, AudioChannel};
use rand::*;

use crate::menu::ButtonActiveEvt;

pub struct SfxPlugin;
impl Plugin for SfxPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<QueueSoundEvt>()
            .add_system_set_to_stage(
                CoreStage::Last,
                SystemSet::new()
                    .label(SfxPhase::ProcessQueue)
                    .after(SfxPhase::PrepareQueue)
                    .with_system(play_queued_sounds),
            )
            .add_system_set_to_stage(
                CoreStage::Last,
                SystemSet::new()
                    .label(SfxPhase::PrepareQueue)
                    .with_system(proxy_as_sfx_event::<ButtonActiveEvt>(
                        "click".to_owned(),
                        13,
                    )),
            );
    }
}

#[derive(SystemLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SfxPhase {
    PrepareQueue,
    ProcessQueue,
}

pub trait SfxEvt {
    fn get_volume(&self) -> f32;
}

impl SfxEvt for ButtonActiveEvt {
    fn get_volume(&self) -> f32 {
        0.5
    }
}

pub struct QueueSoundEvt {
    volume: f32,
    file_prefix: String,
    range: RangeInclusive<u8>,
    channel_suffix: Option<usize>,
}

impl Default for QueueSoundEvt {
    fn default() -> Self {
        Self {
            volume: 1.,
            file_prefix: String::from(""),
            range: 1..=1,
            channel_suffix: None,
        }
    }
}

fn play_queued_sounds(
    mut ev_r: EventReader<QueueSoundEvt>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
) {
    for ev in ev_r.iter() {
        let index = thread_rng().gen_range(ev.range.clone());
        let path = format!("audio/sfx/{}{index}.ogg", ev.file_prefix);
        let channel_key = if let Some(suffix) = ev.channel_suffix {
            format!("{}{}", ev.file_prefix, suffix)
        } else {
            ev.file_prefix.clone()
        };
        let channel = AudioChannel::new(channel_key);
        audio.set_volume_in_channel(ev.volume, &channel);
        audio.play_in_channel(asset_server.load(&path), &channel);
        trace!("playing {path} at volume {}", ev.volume);
    }
}

fn proxy_as_sfx_event<TEvt: Resource + SfxEvt>(
    file_prefix: String,
    to_index: u8,
) -> impl FnMut(EventReader<TEvt>, EventWriter<QueueSoundEvt>) {
    move |mut ev_r: EventReader<TEvt>, mut sfx_queue_ev_w: EventWriter<QueueSoundEvt>| {
        for ev in ev_r.iter() {
            sfx_queue_ev_w.send(QueueSoundEvt {
                volume: ev.get_volume(),
                file_prefix: file_prefix.to_owned(),
                range: 1..=to_index,
                ..Default::default()
            });
        }
    }
}
