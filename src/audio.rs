extern crate cpal;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc::Sender;

pub struct AudioConfig {
    pub input_device: String,
    pub input_channel: usize
}

pub fn init(sender: Sender<f32>, config: &AudioConfig) {
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
        .expect("Failed to get input config")
        .into();

    println!("🔨 Selected config: {:?}", input_config);

    let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        for &sample in data {
            sender.send(sample * 5.0).unwrap();
        }
    };
    
    let input_stream = input_device.build_input_stream(
        &input_config,
        input_data_fn,
        err_fn
    ).expect("Failed to build input stream");

    input_stream.play().expect("Failed to open input stream");

    loop {

    }
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}
