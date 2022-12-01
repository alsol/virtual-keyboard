extern crate cpal;

use cpal::{traits::{DeviceTrait, HostTrait}, Stream, StreamConfig};
use std::sync::mpsc::Sender;

pub struct AudioConfig {
    pub input_device: String,
    pub input_channel: usize
}

pub struct AudioDeviceMeta {
    pub name: String,
    pub default: bool
}

pub struct AudioDevice {
    pub input_stream: Stream,
    pub buffer_size: usize
}

pub fn list() -> Vec<AudioDeviceMeta> {
    let host = cpal::default_host();

    let default_device_name = &host.default_input_device()
        .map(|d| d.name().unwrap())
        .unwrap();

    host.input_devices().unwrap()
        .map(|d| AudioDeviceMeta {
            name: d.name().unwrap(),
            default: d.name().map(|name| name.eq(default_device_name)).unwrap_or(false)
        })
        .collect()
}

pub fn init(sender: Sender<f32>, config: &AudioConfig) -> AudioDevice {
    let host = cpal::default_host();
    
    let input_device = if config.input_device == "default" {
        host.default_input_device()
    } else {
        host.input_devices().unwrap()
            .find(|d| d.name().map(|name| name == config.input_device).unwrap_or(false))
    }
    .expect("Failed to find input device");

    println!("🔊 Input device: {}", input_device.name().unwrap());

    let input_config = input_device.supported_input_configs().unwrap().enumerate()
        .find(|(index, _)| index == &(config.input_channel - 1))
        .map(|(_, c)| c.with_max_sample_rate())
        .expect("Failed to get input config");

    let buffer_size: u32 = match input_config.buffer_size() {
        cpal::SupportedBufferSize::Range { min: _, max } => *max,
        cpal::SupportedBufferSize::Unknown => panic!("💀 Unable to calculate buffer size for selected device"),
    };

    let stream_config = StreamConfig {
        channels: input_config.channels(),
        sample_rate: input_config.sample_rate(),
        buffer_size: cpal::BufferSize::Fixed(buffer_size)
    };

    println!("🔨 Selected config: {:?}", stream_config);

    let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        for &sample in data {
            sender.send(sample * 5.0).unwrap();
        }
    };
    
    let input_stream = input_device.build_input_stream(
        &stream_config,
        input_data_fn,
        err_fn
    ).expect("Failed to build input stream");

    AudioDevice {
        input_stream,
        buffer_size: buffer_size as usize
    }
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}
