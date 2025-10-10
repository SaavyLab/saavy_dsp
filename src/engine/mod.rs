pub mod allocator;
pub mod scheduler;

use self::{allocator::VoiceAllocator, scheduler::Scheduler};

pub struct EngineComponents {
    pub scheduler: Scheduler,
    pub allocator: Box<dyn VoiceAllocator + Send>,
}
