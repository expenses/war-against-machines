// Audio Playback

use rodio::{Decoder, Sink, Device};
use rodio::default_output_device;

use std::rc::Rc;
use std::io::Cursor;

// An audio player struct
pub struct Player {
    sources: [Rc<Vec<u8>>; 3],
    sinks: Vec<Sink>,
    endpoint: Device
}

impl Player {
    pub fn new(sounds: [&[u8]; 3]) -> Player {
        Player {
            // Use reference-counting to avoid cloning the source each time
            sources: [Rc::new(sounds[0].to_vec()), Rc::new(sounds[1].to_vec()), Rc::new(sounds[2].to_vec())],
            sinks: Vec::new(),
            endpoint: default_output_device().unwrap()
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
        let mut sink = Sink::new(&self.endpoint);
        sink.append(source);
        sink.set_volume(volume);

        // Append the sink
        self.sinks.push(sink);
    }
}