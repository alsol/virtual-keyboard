use midir::{MidiOutput, MidiOutputConnection};

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

    pub fn send(&mut self, freq: f32, _amp: f32, event: NoteEvent) {
        const VELOCITY: u8 = 0x64;

        let note = freq_to_midi_note(freq);
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