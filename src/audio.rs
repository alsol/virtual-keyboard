extern crate cpal;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc::Sender;

pub fn init(sender: Sender<f32>) {
    let host = cpal::default_host();

    let input_device = host.default_input_device().expect("No input device found");

    println!("Input device: {}", input_device.name().unwrap());

    let config = input_device
        .default_input_config()
        .expect("Failed to get default input config")
        .into();

    println!("Input device config: {:?}", config);

    let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        for &sample in data {
            sender.send(sample * 5.0).unwrap();
        }
    };
    
    let input_stream = input_device.build_input_stream(
        &config,
        input_data_fn,
        err_fn
    ).expect("Failed to build input stream");

    input_stream.play().expect("Failed to start playing");

    loop {

    }
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}
