use std::cell::UnsafeCell;

use util::downcast_rs::{impl_downcast, Downcast};

use crate::{BoxedStageLabel, CommandBuffer, Resources, UnsafeResources, World};

pub trait ParRunnable: Runnable + Send + Sync {}
impl<T: Runnable + Send + Sync> ParRunnable for T {}

pub trait Runnable {
    unsafe fn run_unsafe(&mut self, world: &World, resources: &UnsafeResources);

    fn command_buffer_mut(&mut self) -> Option<&mut CommandBuffer>;

    fn stage(&self) -> Option<BoxedStageLabel>;

    fn run(&mut self, world: &World, resources: &mut Resources) {
        unsafe { self.run_unsafe(world, resources.internal()) }
    }
}

pub(crate) struct SystemBox(UnsafeCell<Box<dyn ParRunnable>>);
unsafe impl Send for SystemBox {}
unsafe impl Sync for SystemBox {}

impl SystemBox {
    pub(crate) fn new<S: ParRunnable + 'static>(system: S) -> Self {
        SystemBox(UnsafeCell::new(Box::new(system)))
    }

    pub(crate) unsafe fn get_mut(&self) -> &mut dyn ParRunnable {
        std::ops::DerefMut::deref_mut(&mut *self.0.get())
    }
}

pub(crate) trait Executor: Downcast + Send + Sync {
    fn cache_data(&mut self, systems: &[SystemBox]);
    fn run_systems(
        &mut self,
        systems: &[SystemBox],
        world: &mut World,
        resources: &UnsafeResources,
    );
}
impl_downcast!(Executor);

#[derive(Default)]
pub struct SequenceExecutor {}

impl Executor for SequenceExecutor {
    fn cache_data(&mut self, _systems: &[SystemBox]) {}

    fn run_systems(
        &mut self,
        systems: &[SystemBox],
        world: &mut World,
        resources: &UnsafeResources,
    ) {
        for system in systems {
            let borrow = unsafe { system.get_mut() };
            unsafe { borrow.run_unsafe(world, resources) }
        }
    }
}

#[derive(Default)]
pub struct SequenceOnceExecutor {
    ran: bool,
}

impl Executor for SequenceOnceExecutor {
    fn cache_data(&mut self, _systems: &[SystemBox]) {}

    fn run_systems(
        &mut self,
        systems: &[SystemBox],
        world: &mut World,
        resources: &UnsafeResources,
    ) {
        if self.ran {
            return;
        }

        for system in systems {
            let borrow = unsafe { system.get_mut() };
            unsafe { borrow.run_unsafe(world, resources) }
        }
        self.ran = true;
    }
}
