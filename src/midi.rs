use midir::{MidiOutput, MidiOutputConnection};

pub struct Midi {
    name: String,
    connection: MidiOutputConnection,
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

    pub fn send(&mut self, freq: f32, amp: f32) {
        const NOTE_ON_MSG: u8 = 0x90;
        const NOTE_OFF_MSG: u8 = 0x80;
        const VELOCITY: u8 = 0x64;

        let note = freq_to_midi_note(freq);
        self.connection.send(&[NOTE_ON_MSG, note, VELOCITY]).unwrap();
    }
}

pub fn open_midi_output() -> Result<Midi, String> {
    let midi_out = MidiOutput::new("virtual-keyboard midi").unwrap();

    let out_ports = midi_out.ports();
    if out_ports.len() == 0 {
        return Err("No midi output ports found".to_string());
    }

    let midi_port = midi_out.ports().into_iter().next().unwrap();
    match midi_out.connect(&midi_port,  "midi-output") {
        Ok(connection) => Ok(Midi::new("virtual-keyboard midi".to_string(), connection)),
        Err(err) => Err(format!("Failed to connect to output port: {}", err)),
    }
}

fn freq_to_midi_note(freq: f32) -> u8 {
    let note = 12.0 * (freq / 440.0).log2() + 69.0;
    note.round() as u8
}