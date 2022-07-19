use ringbuf::RingBuffer;
use std::sync::mpsc;
use std::thread;

mod audio;
mod fft;
mod midi;

struct Props {
    buffer_size: usize,
    threshold: f32,
}

impl Props {
    fn new() -> Self {
        Self {
            buffer_size: 8_192,
            threshold: 20.0,
        }
    }
}

#[derive(Debug)]
struct Note {
    freq: f32,
    amp: f32,
}

impl Note {
    fn new(freq: f32, amp: f32) -> Self {
        Self { freq, amp }
    }
}

fn main() {
    let props = Props::new();

    let (sample_tx, sample_rx) = mpsc::channel();
    let (note_tx, note_rx) = mpsc::channel();

    let ring_buffer = RingBuffer::<f32>::new(props.buffer_size);

    let (mut prod, mut cons) = ring_buffer.split();

    thread::spawn(move || {
        audio::init(sample_tx);
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

                if maxima.1 > 0 {
                    let note = Note::new(
                        fft::recalculate_to_freq(maxima.1, props.buffer_size, 44_100.0),
                        maxima.0,
                    );
                    note_tx.send(note).unwrap();
                }
            }
        }
    });

    let midi_output = midi::open_midi_output();

    match midi_output {
        Ok(mut midi) => {
            println!(
                "Successfully connected as a midi device, forwarding audio input to {}",
                midi.name()
            );
            let mut currently_playing_note: Option<Note> = None;
            loop {
                let note = note_rx.recv().unwrap();

                let next_note:Note = match &currently_playing_note {
                    Some(current_note) => {
                        // It means that we are still holding the same note -> do nothing
                        if current_note.freq == note.freq && current_note.amp > note.amp {
                            continue;
                        }

                        midi.send(current_note.freq, current_note.amp, midi::NoteEvent::NoteOff);
                        midi.send(note.freq, note.amp, midi::NoteEvent::NoteOn);
                        note
                    }
                    None => {
                        midi.send(note.freq, note.amp, midi::NoteEvent::NoteOn);
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
