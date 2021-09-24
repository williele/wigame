use std::cell::UnsafeCell;

use crate::{CommandBuffer, QueryEntry, World};

#[derive(Default)]
pub struct Scheduler {
    systems: Vec<SystemBox>,
}

pub trait System: 'static + Send + Sync {
    fn can_run(&mut self, _query: &QueryEntry) -> bool {
        true
    }

    fn run(&mut self, command: &mut CommandBuffer, query: &QueryEntry);
}

struct SystemBox(UnsafeCell<Box<dyn System>>);
unsafe impl Send for SystemBox {}
unsafe impl Sync for SystemBox {}

impl Scheduler {
    pub fn add(&mut self, system: impl System) -> &mut Self {
        self.systems
            .push(SystemBox(UnsafeCell::new(Box::new(system))));
        self
    }

    pub fn execute(&mut self, world: &mut World) {
        let entry = QueryEntry::new(world);
        let mut command = CommandBuffer::new();

        self.systems.iter_mut().for_each(|system| {
            let system = system.0.get_mut();
            if system.can_run(&entry) {
                system.run(&mut command, &entry);
            }
        });
        command.flush(world);
    }
}
