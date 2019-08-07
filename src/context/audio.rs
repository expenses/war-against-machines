// Audio Playback

use resources::AUDIO;

use rodio::*;

use std::io::Cursor;
use std::rc::Rc;

// An audio player struct
pub struct Player {
    sources: Vec<Rc<Vec<u8>>>,
    sinks: Vec<Sink>,
    endpoint: Device,
}

impl Player {
    pub fn new() -> Player {
        Player {
            // Use reference-counting to avoid cloning the source each time
            sources: AUDIO.iter().map(|sound| Rc::new(sound.to_vec())).collect(),
            sinks: Vec::new(),
            endpoint: default_output_device().unwrap(),
        }
    }

    // Play a sound with a certain volume
    pub fn play(&mut self, index: usize, volume: f32) {
        // Get the source
        let source = Decoder::new(Cursor::new(self.sources[index].as_ref().clone())).unwrap();

        // Try to find an empty sink and append the source to that
        for sink in &mut self.sinks {
            if sink.empty() {
                sink.append(source);
                sink.set_volume(volume);
                return;
            }
        }

        // Or create a new sink
        let sink = Sink::new(&self.endpoint);
        sink.append(source);
        sink.set_volume(volume);

        // Append the sink
        self.sinks.push(sink);
    }
}
