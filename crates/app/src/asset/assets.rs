use std::{collections::HashMap, fmt::Debug};

use util::crossbeam_channel::Sender;

use crate::{AppStage, Asset, Events, Handle, HandleId, ParRunnable, RefChange, SystemBuilder};

#[derive(Debug)]
pub enum AssetEvent<T: Asset> {
    Created { handle: Handle<T> },
    Modified { handle: Handle<T> },
    Removed { handle: Handle<T> },
}

#[derive(Debug)]
pub struct Assets<T: Asset> {
    assets: HashMap<HandleId, T>,
    events: Events<AssetEvent<T>>,
    pub(crate) ref_change_sender: Sender<RefChange>,
}

impl<T: Asset> Assets<T> {
    pub(crate) fn new(ref_change_sender: Sender<RefChange>) -> Self {
        Self {
            assets: Default::default(),
            events: Default::default(),
            ref_change_sender,
        }
    }

    pub fn get_handle<H: Into<HandleId>>(&self, handle: H) -> Handle<T> {
        Handle::strong(handle.into(), self.ref_change_sender.clone())
    }

    pub fn get<H: Into<HandleId>>(&self, handle: H) -> Option<&T> {
        self.assets.get(&handle.into())
    }

    pub fn add(&mut self, asset: T) -> Handle<T> {
        let id = HandleId::random::<T>();
        self.assets.insert(id, asset);
        self.events.send(AssetEvent::Created {
            handle: Handle::weak(id),
        });
        self.get_handle(id)
    }

    pub fn set<H: Into<HandleId>>(&mut self, handle: H, asset: T) -> Handle<T> {
        let id = handle.into();
        if self.assets.insert(id, asset).is_some() {
            self.events.send(AssetEvent::Modified {
                handle: Handle::weak(id),
            });
        } else {
            self.events.send(AssetEvent::Created {
                handle: Handle::weak(id),
            });
        }
        self.get_handle(id)
    }

    pub fn remove<H: Into<HandleId>>(&mut self, handle: H) -> Option<T> {
        let id = handle.into();
        let asset = self.assets.remove(&id);
        if asset.is_some() {
            self.events.send(AssetEvent::Removed {
                handle: Handle::weak(id),
            })
        }
        asset
    }

    pub fn has<H: Into<HandleId>>(&self, handle: H) -> bool {
        self.assets.contains_key(&handle.into())
    }

    pub fn clear(&mut self) {
        self.assets.clear();
    }

    fn update_events(&mut self, events: &mut Events<AssetEvent<T>>) {
        if !self.events.is_empty() {
            events.extend(self.events.drain())
        }
    }
}

pub(crate) fn asset_event_sys<T: Asset>() -> impl ParRunnable {
    SystemBuilder::new()
        .on_stage(AppStage::AssetEvents)
        .write_resource::<Events<AssetEvent<T>>>()
        .write_resource::<Assets<T>>()
        .build(|_, _, (events, assets), _| assets.update_events(events))
}
