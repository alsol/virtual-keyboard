use ringbuf::RingBuffer;
use std::sync::mpsc;
use std::thread;
use midi::Note;
use clap::arg;

mod audio;
mod fft;
mod midi;

struct Props {
    audio_config: audio::AudioConfig,
    buffer_size: usize,
    threshold: f32,
    debug: bool
}

impl Props {
    fn from_args() -> Self {
        let app = clap::Command::new("forward")
            .arg(arg!([IN] "The input device to use [default: default]"))
            .arg(arg!(-c --channel [CHANNEL] "The input channel for input device [default: 1]"))
            .arg(arg!(-t --threshold [THRESHOLD] "Specify the threshold for input signal [default: 250]"))
            .arg(arg!(-d --debug "Produce execution debug output [default: false]"));

        let matches = app.get_matches();

        let input_device = matches.value_of("IN").unwrap_or("Scarlett Solo USB").to_string();
        let input_channel: usize = matches
            .value_of("channel")
            .unwrap_or("2")
            .parse()
            .unwrap();
        let threshold: f32 = matches
            .value_of("threshold")
            .unwrap_or("250")
            .parse()
            .unwrap();

        let debug: bool = matches.is_present("debug");
        let buffer_size = 8_192;

        Self { 
            audio_config: audio::AudioConfig {
                input_device,
                input_channel
            },
             buffer_size,
             threshold, 
             debug 
        }
    }
}

fn main() {
    let props = Props::from_args();

    let (sample_tx, sample_rx) = mpsc::channel();
    let (note_tx, note_rx) = mpsc::channel();

    let ring_buffer = RingBuffer::<f32>::new(props.buffer_size);

    let (mut prod, mut cons) = ring_buffer.split();

    thread::spawn(move || {
        audio::init(sample_tx, &props.audio_config);
    });

    thread::spawn(move || loop {
        let sample = sample_rx.recv().unwrap();
        match prod.push(sample) {
            Ok(_) => {}
            Err(_) => {}
        }
    });

    thread::spawn(move || {
        let mut sample: Vec<f32> = vec![0.0; props.buffer_size];

        loop {
            if cons.is_full() {
                cons.pop_slice(&mut sample);
                let fft = fft::calculate_fft(&sample);
                let maxima = fft::find_maxima(&fft, props.threshold);

                let note = Note::new(
                    fft::recalculate_to_freq(maxima.1, props.buffer_size, 48_000.0),
                    maxima.0,
                );

                if props.debug && !&note.is_dead() {
                    println!("{:?}", &note)
                }

                note_tx.send(note).unwrap();
            }
        }
    });

    let midi_output = midi::open_midi_output();

    match midi_output {
        Ok(mut midi) => {
            println!(
                "🚀 Successfully connected as a midi device, forwarding audio input to {}",
                midi.name()
            );
            let mut currently_playing_note: Option<Note> = None;
            loop {
                let note = note_rx.recv().unwrap();

                // Dead note received, stop playing current note
                if note.is_dead() && currently_playing_note.is_some() {
                    let current_note = currently_playing_note.unwrap();
                    midi.note_off(&current_note);
                    currently_playing_note = None;
                    continue;
                } else if note.is_dead() {
                    continue;
                }

                let next_note: Note = match &currently_playing_note {
                    Some(current_note) => {
                        // It means that we are still holding the same note -> do nothing
                        if current_note.is_same(&note) {
                            continue;
                        }

                        midi.note_off(current_note);
                        midi.note_on(&note);
                        note
                    }
                    None => {
                        midi.note_on(&note);
                        note
                    }
                };

                currently_playing_note = Some(next_note);
            }
        }
        Err(e) => {
            eprintln!("Failed to open midi output: {}", e);
            std::process::exit(1);
        }
    }
}
