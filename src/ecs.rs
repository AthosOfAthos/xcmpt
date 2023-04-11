use crate::component::{ComponentID, ComponentInfo};
use crate::storage::{Slot, ComponentMap};
use crate::{Component, ComponentRegistry};
use alloc::vec::Vec;
use runtime_id::RuntimeID;

type Index = usize;
type Generation = usize;

#[derive(Clone, Copy)]
pub struct EntityID {
	scene_id: RuntimeID,
	index: Index,
	generation: Generation,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Entity {
	alive: bool,
	generation: Generation,
}

pub type GrowFn = fn(usize) -> usize;

pub struct ECS {
	scene_id: RuntimeID,
	capacity: usize,
	entity_count: usize,
	grow_fn: Option<GrowFn>,
	entities: Vec<Entity>,
	components: ComponentMap,
}

impl ECS {
	pub fn new(capacity: usize) -> Self {
		let mut entities = Vec::with_capacity(capacity);
		entities.resize(capacity, Entity { alive: false, generation: 0 });

		ECS {
			scene_id: RuntimeID::new(),
			capacity,
			entity_count: 0,
			grow_fn: None,
			entities,
			components: ComponentMap::new(),
		}
	}
	
	pub fn from_registry(registry: &ComponentRegistry, capacity: usize) -> Self {
		let mut ecs = ECS::new(capacity);

		for (id, info) in &registry.components {
			ecs.components.register(*id, *info, capacity);
		}

		return ecs;
	}
	
	pub fn register<C: Component>(&mut self) {
		self.components.register(ComponentID::of::<C>(), ComponentInfo::new::<C>(), self.capacity);
	}

	pub const fn get_capacity(&self) -> usize { self.capacity }

	pub fn set_grow_fn(&mut self, grow: Option<GrowFn>) { self.grow_fn = grow }

	pub fn grow_capacity(&mut self) {
		let new_capacity = self.grow_fn.unwrap()(self.capacity);
		self.grow_capacity_to_size(new_capacity);
	}

	pub fn grow_capacity_to_size(&mut self, new_capacity: usize) {
		assert!(new_capacity > self.capacity, "new capacity must be larget than previous");
		self.entities.resize(new_capacity, Entity { alive: false, generation: 0 });
		self.components.resize(new_capacity);
		self.capacity = new_capacity;
	}

	pub const fn get_entity_count(&self) -> usize { self.entity_count }

	pub fn is_valid(&self, entity: &EntityID) -> bool {
		if entity.scene_id != self.scene_id { return false; }
		self.entities[entity.index] == Entity { alive: true, generation: entity.generation }
	}

	fn allocate_entity(&mut self) -> Option<EntityID> {
		for index in 0..self.capacity {
			let entity = &mut self.entities[index];
			if !entity.alive {
				entity.alive = true;
				entity.generation += 1;
				return Some(EntityID { scene_id: self.scene_id, index, generation: entity.generation });
			}
		}
		None
	}
	
	pub fn create_entity(&mut self) -> Option<EntityID> {
		let entity = match self.allocate_entity() {
			Some(entity) => entity,
			None => {
				if self.grow_fn == None {
					return None;
				} else {
					self.grow_capacity();
					self.allocate_entity().unwrap()
				}
			},
		};

		self.entity_count += 1;
		return Some(entity);
	}

	pub fn destroy_entity(&mut self, entity: EntityID) {
		if self.is_valid(&entity) {
			self.entity_count -= 1;
			self.components.delete_index(entity.index);
			self.entities[entity.index].alive = false;
		}
	}

	pub fn has_component<C: Component>(&self, entity: &EntityID) -> bool {
		if !self.is_valid(entity) { return false; }
		return match self.components.get_array::<C>() {
		    Some(array) => array[entity.index].is_filled(),
		    None => false,
		}
	}

	pub fn add_component<C: Component>(&mut self, entity: &EntityID, component: C) {
		if !self.is_valid(entity) { return; }
		match unsafe { self.components.get_array_mut::<C>() } {
		    Some(array) => array[entity.index] = Slot::Filled(component),
		    None => todo!(),
		}
	}

	pub fn remove_component<C: Component>(&mut self, entity: &EntityID) {
		if !self.is_valid(entity) { return; }
		match unsafe { self.components.get_array_mut::<C>() } {
			Some(array) => array[entity.index] = Slot::Empty,
			None => todo!(),
		}
	}

	pub fn get_component<C: Component>(&self, entity: &EntityID) -> Option<&C> {
		if !self.is_valid(entity) { return None; }
		match self.components.get_array::<C>() {
		    Some(array) => array[entity.index].as_option(),
		    None => None,
		}
	}

	pub fn get_component_mut<C: Component>(&mut self, entity: &EntityID) -> Option<&mut C> {
		if !self.is_valid(entity) { return None; }
		match unsafe { self.components.get_array_mut::<C>() } {
		    Some(array) => array[entity.index].as_option_mut(),
		    None => None,
		}
	}
}

#[cfg(test)]
mod test {
	use crate::{ComponentRegistry, ECS, Component};
	
	#[derive(PartialEq, Eq)]
	struct TestComponent(usize);
	impl Component for TestComponent {}

	#[test]
	fn basic() {
		let registry = ComponentRegistry::new();
		let mut ecs = ECS::from_registry(&registry, 512);

		let entity = ecs.create_entity().unwrap();
		assert!(ecs.is_valid(&entity));
		ecs.destroy_entity(entity);
		assert!(!ecs.is_valid(&entity));
	}

	#[test]
	fn count() {
		const CAPACITY: usize = 64;
		const ENTITY_COUNT: usize = 47;
		
		let mut ecs = ECS::new(CAPACITY);
		
		assert_eq!(ecs.get_entity_count(), 0);
		
		for _ in 0..ENTITY_COUNT {
			ecs.create_entity().unwrap();
		}
		assert_eq!(ecs.get_entity_count(), ENTITY_COUNT);
	}

	#[test]
	fn grow_to_size() {
		const STARTING_CAPACITY: usize = 64;
		const NEW_CAPACITY: usize = 1024;
		
		let mut ecs = ECS::new(STARTING_CAPACITY);
		ecs.register::<TestComponent>();
		
		assert_eq!(ecs.capacity, STARTING_CAPACITY);
		
		ecs.grow_capacity_to_size(NEW_CAPACITY);
		assert_eq!(ecs.capacity, NEW_CAPACITY);
	}

	#[test]
	fn grow_fn() {
		const STARTING_CAPACITY: usize = 32;
		let grow = |capacity: usize| -> usize { capacity * 2 };

		let mut ecs = ECS::new(STARTING_CAPACITY);
		ecs.set_grow_fn(Some(grow));
		
		ecs.grow_capacity();
		assert_eq!(ecs.get_capacity(), STARTING_CAPACITY * 2);
	}

	#[test]
	fn component_basic() {
		let mut ecs = ECS::new(512);
		ecs.register::<TestComponent>();

		let entity = ecs.create_entity().unwrap();
		assert!(!ecs.has_component::<TestComponent>(&entity));
		let test_component = TestComponent(128);
		ecs.add_component(&entity, test_component);
		assert!(ecs.has_component::<TestComponent>(&entity));

		ecs.get_component_mut::<TestComponent>(&entity).unwrap().0 = 72;
		let component = ecs.get_component::<TestComponent>(&entity).unwrap();
		assert_eq!(component.0, 72);
		
		ecs.remove_component::<TestComponent>(&entity);
		assert!(!ecs.has_component::<TestComponent>(&entity));
	}

	mod drop {
		use crate::{ECS, Component};
		use core::sync::atomic::{AtomicUsize, Ordering};

		static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);
		struct Dropper {}
		impl Component for Dropper {}
		impl Drop for Dropper {
		    fn drop(&mut self) {
				DROP_COUNT.fetch_add(1, Ordering::Relaxed);
		    }
		}

		#[test]
		fn drop_test() {
			const COUNT: usize = 231;
			
			let mut ecs = ECS::new(512);
			ecs.register::<Dropper>();
			for _ in 0..(COUNT - 1) {
				let entity = ecs.create_entity().unwrap();
				ecs.add_component::<Dropper>(&entity, Dropper {});
			}

			let entity = ecs.create_entity().unwrap();
			ecs.add_component::<Dropper>(&entity, Dropper {});
			ecs.remove_component::<Dropper>(&entity);
			drop(ecs);
			
			assert_eq!(DROP_COUNT.load(Ordering::Relaxed), COUNT);
		}
	}
}
