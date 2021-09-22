use crate::{
    entity::{Entities, Entity},
    Component, Components,
};

#[derive(Default)]
pub struct World {
    components: Components,
    entities: Entities,
}

pub struct WorldExt<'a> {
    components: &'a mut Components,
    entity: Entity,
}

impl<'a> WorldExt<'a> {
    pub fn with<C: Component>(&mut self, component: C) -> &mut Self {
        self.components.insert(self.entity, component);
        self
    }

    pub fn build(&mut self) -> Entity {
        self.entity
    }
}

impl World {
    pub fn spawn(&mut self) -> WorldExt {
        WorldExt {
            components: &mut self.components,
            entity: self.entities.alloc(),
        }
    }

    pub fn components(&self) -> &Components {
        &self.components
    }
}
