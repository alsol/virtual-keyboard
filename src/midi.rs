use midir::{MidiOutput, MidiOutputConnection};

#[derive(Debug)]
pub struct Note {
    freq: f32,
    amp: f32,
}

impl Note {
    pub fn new(freq: f32, amp: f32) -> Self {
        Self { freq, amp }
    }

    pub fn is_dead(&self) -> bool {
        self.amp < 0.01
    }

    pub fn is_same(&self, other: &Note) -> bool {
        self.freq == other.freq && self.amp >= other.amp
    }
}

pub struct Midi {
    name: String,
    connection: MidiOutputConnection,
}

pub enum NoteEvent {
    NoteOn,
    NoteOff,
}

impl NoteEvent {
    fn value(&self) -> u8 {
        match *self {
            NoteEvent::NoteOn => 0x90,
            NoteEvent::NoteOff => 0x80,
        }
    }
}

impl Midi {

    fn new(name: String, connection: MidiOutputConnection) -> Self {
        Self {
            name, 
            connection,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn note_on(&mut self, note: &Note) {
        self.send(note, NoteEvent::NoteOn)
    }

    pub fn note_off(&mut self, note: &Note) {
        self.send(note, NoteEvent::NoteOff)
    }

    pub fn send(&mut self, note: &Note, event: NoteEvent) {
        const VELOCITY: u8 = 0x64;

        let note = freq_to_midi_note(note.freq);
        self.connection.send(&[event.value(), note, VELOCITY]).unwrap();
    }
}

pub fn open_midi_output() -> Result<Midi, String> {
    let client_name = "virtual-keyboard midi";
    let midi_out = MidiOutput::new(client_name).unwrap();

    let out_ports = midi_out.ports();
    if out_ports.len() == 0 {
        return Err("No midi output ports found".to_string());
    }

    let midi_port = midi_out.ports().into_iter().next().unwrap();
    
    match midi_out.connect(&midi_port,  "midi-output") {
        Ok(connection) => Ok(Midi::new(client_name.to_string(), connection)),
        Err(err) => Err(format!("Failed to connect to output port: {}", err)),
    }
}

fn freq_to_midi_note(freq: f32) -> u8 {
    let note = 12.0 * (freq / 440.0).log2() + 69.0;
    note.round() as u8
}