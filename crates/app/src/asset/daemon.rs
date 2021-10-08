use std::{any::TypeId, collections::HashMap, sync::Arc};

use util::{
    crossbeam_channel::{self, Receiver, Sender, TryRecvError},
    downcast_rs::{impl_downcast, Downcast},
    parking_lot::RwLock,
};

use crate::{AppStage, Asset, AssetDyn, Assets, HandleId, ParRunnable, SystemBuilder};

pub(crate) enum RefChange {
    Increment(HandleId),
    Decrement(HandleId),
}

#[derive(Clone)]
pub(crate) struct RefChangeChannel {
    pub sender: Sender<RefChange>,
    pub receiver: Receiver<RefChange>,
}

impl Default for RefChangeChannel {
    fn default() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        RefChangeChannel { sender, receiver }
    }
}

pub enum AssetLifecycleEvent<T> {
    Create {
        asset: Box<T>,
        id: HandleId,
        version: usize,
    },
    Free(HandleId),
}

trait AssetLifecycle: Downcast + Send + Sync + 'static {
    fn create_asset(&self, id: HandleId, asset: Box<dyn AssetDyn>, version: usize);
    fn free_asset(&self, id: HandleId);
}
impl_downcast!(AssetLifecycle);

pub(crate) struct AssetLifecycleChanel<T> {
    pub sender: Sender<AssetLifecycleEvent<T>>,
    pub receiver: Receiver<AssetLifecycleEvent<T>>,
}

impl<T> Default for AssetLifecycleChanel<T> {
    fn default() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        Self { sender, receiver }
    }
}

impl<T: Asset> AssetLifecycle for AssetLifecycleChanel<T> {
    fn create_asset(&self, id: HandleId, asset: Box<dyn AssetDyn>, version: usize) {
        if let Ok(asset) = asset.downcast::<T>() {
            self.sender
                .send(AssetLifecycleEvent::Create { asset, id, version })
                .unwrap()
        } else {
            panic!(
                "Failed to downcast asset to {}.",
                std::any::type_name::<T>()
            );
        }
    }

    fn free_asset(&self, id: HandleId) {
        self.sender.send(AssetLifecycleEvent::Free(id)).unwrap();
    }
}

#[derive(Default)]
struct AssetRefCounter {
    channel: RefChangeChannel,
    ref_counts: Arc<RwLock<HashMap<HandleId, usize>>>,
}

#[derive(Default)]
pub struct AssetDaemon {
    ref_counter: AssetRefCounter,
    asset_lifecycles: Arc<RwLock<HashMap<TypeId, Box<dyn AssetLifecycle>>>>,
}

impl AssetDaemon {
    pub(crate) fn register_asset<T: Asset>(&mut self) -> Assets<T> {
        self.asset_lifecycles.write().insert(
            TypeId::of::<T>(),
            Box::new(AssetLifecycleChanel::<T>::default()),
        );
        Assets::new(self.ref_counter.channel.sender.clone())
    }

    fn free_unused_assets(&self) {
        let receiver = &self.ref_counter.channel.receiver;
        let mut ref_counts = self.ref_counter.ref_counts.write();
        let asset_lifecycles = self.asset_lifecycles.read();

        loop {
            let ref_change = match receiver.try_recv() {
                Ok(ref_change) => ref_change,
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => panic!("RefChange channel disconnected"),
            };

            match ref_change {
                RefChange::Increment(id) => *ref_counts.entry(id).or_insert(0) += 1,
                RefChange::Decrement(id) => {
                    let entry = ref_counts.entry(id).or_insert(0);
                    *entry -= 1;
                    if *entry == 0 {
                        let type_id = match id {
                            HandleId::Id(type_id, _) => Some(type_id),
                        };

                        if let Some(type_id) = type_id {
                            if let Some(asset_lifecycle) = asset_lifecycles.get(&type_id) {
                                asset_lifecycle.free_asset(id)
                            }
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn update_asset_storage<T: Asset>(&self, assets: &mut Assets<T>) {
        let asset_lifecycles = self.asset_lifecycles.read();
        let asset_lifecycle = asset_lifecycles.get(&TypeId::of::<T>()).unwrap();
        let channel = asset_lifecycle
            .downcast_ref::<AssetLifecycleChanel<T>>()
            .unwrap();

        loop {
            match channel.receiver.try_recv() {
                Ok(AssetLifecycleEvent::Create { .. }) => {}
                Ok(AssetLifecycleEvent::Free(id)) => {
                    assets.remove(id);
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => panic!("AssetChannel disconnected"),
            }
        }
    }
}

pub(crate) fn free_unused_assets_sys() -> impl ParRunnable {
    SystemBuilder::new()
        .on_stage(AppStage::PreUpdate)
        .read_resource::<AssetDaemon>()
        .build(|_, _, daemon, _| daemon.free_unused_assets())
}

pub(crate) fn update_asset_storage_sys<T: Asset>() -> impl ParRunnable {
    SystemBuilder::new()
        .on_stage(AppStage::LoadAssets)
        .read_resource::<AssetDaemon>()
        .write_resource::<Assets<T>>()
        .build(|_, _, (daemon, assets), _| daemon.update_asset_storage(assets))
}
