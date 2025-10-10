use crate::io::midi::MidiEvent;

pub struct Scheduler;

impl Scheduler {
    pub fn new() -> Self {
        Self
    }

    pub fn enqueue(&mut self, _event: MidiEvent) {
        todo!("add timestamped event to timeline")
    }

    pub fn tick(&mut self, _frames: usize) {
        todo!("advance scheduler state and dispatch events")
    }
}
