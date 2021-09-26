use std::marker::PhantomData;

use crate::{ParRunnable, SystemBuilder};

#[derive(Debug)]
enum State {
    A,
    B,
}

#[derive(Debug)]
pub struct Events<T> {
    events_a: Vec<T>,
    events_b: Vec<T>,
    start_a: usize,
    start_b: usize,
    count: usize,
    state: State,
}

impl<T> Default for Events<T> {
    fn default() -> Self {
        Events {
            events_a: Vec::new(),
            events_b: Vec::new(),
            start_a: 0,
            start_b: 0,
            count: 0,
            state: State::A,
        }
    }
}

impl<T> Events<T> {
    pub fn send(&mut self, event: T) {
        match self.state {
            State::A => self.events_a.push(event),
            State::B => self.events_b.push(event),
        }
        self.count += 1;
    }

    pub fn update(&mut self) {
        match self.state {
            State::A => {
                self.events_b.clear();
                self.state = State::B;
                self.start_b = self.count;
            }
            State::B => {
                self.events_a.clear();
                self.state = State::A;
                self.start_b = self.count;
            }
        }
    }

    pub fn drain(&mut self) -> impl Iterator<Item = T> + '_ {
        self.reset_start();
        match self.state {
            State::A => self.events_b.drain(..).chain(self.events_a.drain(..)),
            State::B => self.events_a.drain(..).chain(self.events_b.drain(..)),
        }
    }

    #[inline]
    fn reset_start(&mut self) {
        self.start_a = self.count;
        self.start_b = self.count;
    }

    #[inline]
    pub fn clear(&mut self) {
        self.reset_start();
        self.events_a.clear();
        self.events_b.clear();
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.events_a.is_empty() && self.events_b.is_empty()
    }
}

pub(crate) fn update_event_sys<T: 'static>() -> impl ParRunnable {
    SystemBuilder::new()
        .write_resource::<Events<T>>()
        .build(|_, _, events, _| events.update())
}

impl<T> std::iter::Extend<T> for Events<T> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        let mut count = self.count;
        let events = iter.into_iter().map(|event| {
            count += 1;
            event
        });

        match self.state {
            State::A => self.events_a.extend(events),
            State::B => self.events_b.extend(events),
        }
        self.count = count;
    }
}

pub struct EventReader<T> {
    count: usize,
    _marker: PhantomData<T>,
}

impl<T> Default for EventReader<T> {
    fn default() -> Self {
        EventReader {
            count: 0,
            _marker: PhantomData,
        }
    }
}

impl<T> EventReader<T> {
    pub fn iter<'a>(&mut self, events: &'a Events<T>) -> impl DoubleEndedIterator<Item = &'a T> {
        let a_index = if self.count > events.start_a {
            self.count - events.start_a
        } else {
            0
        };
        let b_index = if self.count > events.start_b {
            self.count - events.start_b
        } else {
            0
        };
        self.count = events.count;
        match events.state {
            State::A => events
                .events_b
                .get(b_index..)
                .unwrap_or_else(|| &[])
                .iter()
                .chain(events.events_a.get(a_index..).unwrap_or_else(|| &[]).iter()),
            State::B => events
                .events_a
                .get(a_index..)
                .unwrap_or_else(|| &[])
                .iter()
                .chain(events.events_b.get(b_index..).unwrap_or_else(|| &[]).iter()),
        }
    }
}
