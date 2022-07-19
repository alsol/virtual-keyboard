use std::sync::mpsc;
use std::thread;
use ringbuf::RingBuffer;

mod audio;
mod fft;

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
    Self {
      freq,
      amp,
    }
  }
}

fn main() {
    let props = Props::new();

    let (tx, rx) = mpsc::channel();
    let (note_tx, note_rx) = mpsc::channel();

    let ring_buffer = RingBuffer::<f32>::new(props.buffer_size);

    let (mut prod,  mut cons) = ring_buffer.split();

    thread::spawn(move || {
        audio::init(tx);
    });

    thread::spawn(move || {
        println!("Started thread");
        loop {
          let sample = rx.recv().unwrap();
          if !prod.is_full() {
            prod.push(sample).unwrap();
          }
        }
      });

    thread::spawn(move || {
        println!("Started thread");
        let mut sample: Vec<f32> = vec![0.0; props.buffer_size];

        loop {
          if cons.is_full() {
            cons.pop_slice(&mut sample);
            let fft = fft::calculate_fft(&sample);
            let maxima = fft::find_maxima(&fft, props.threshold);

            if maxima.1 > 0 {
                let note = Note::new(fft::recalculate_to_freq(maxima.1, props.buffer_size, 44_100.0), maxima.0);
                note_tx.send(note).unwrap();
            }
          }
        }  
      });

      thread::spawn(move || {
        println!("Started thread");
        loop {
          match note_rx.recv() {
            Ok(note) =>  println!("Note: {:?}", note),
            Err(_) => {},
          }
        }
      });

      loop {
        thread::sleep(std::time::Duration::from_millis(100));
      }
    
}