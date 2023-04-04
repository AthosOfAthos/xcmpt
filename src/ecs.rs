
use crate::{Component, ComponentRegistry};
use alloc::rc::Rc;
use runtime_id::RuntimeID;

pub struct EntityID {
	scene_id: RuntimeID,
	index: usize,
	generation: usize,
}

pub struct ECS {
	registry: Rc<ComponentRegistry>,
	scene_id: RuntimeID,
}

impl ECS {
	pub fn new(registry: &Rc<ComponentRegistry>) -> Self {
		unimplemented!()
	}

	pub fn create_entity(&mut self) -> EntityID {
		unimplemented!()
	}

	pub fn destroy_entity(&mut self) {
		unimplemented!()
	}

	pub fn has_component<C: Component>(&self, entity: &EntityID) -> bool {
		unimplemented!()
	}

	pub fn add_component<C: Component>(&mut self, entity: &EntityID, component: C) {
		unimplemented!()
	}

	pub fn remove_component<C: Component>(&mut self, entity: &EntityID) -> Option<C> {
		unimplemented!()
	}

	pub fn get_component<C: Component>(&self, entity: &EntityID) -> Option<&C> {
		unimplemented!()
	}

	pub fn get_component_mut<C: Component>(&mut self, entity: &EntityID) -> Option<&mut C> {
		unimplemented!()
	}
}
