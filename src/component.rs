
use core::alloc::Layout;
use alloc::rc::Rc;
use hashbrown::HashMap;
use crate::storage::Slot;

pub trait Component: Sized + 'static {}

pub(crate) type ComponentID = core::any::TypeId;

#[derive(Clone, Copy)]
pub(crate) struct ComponentInfo {
	pub(crate) name: &'static str,
	pub(crate) layout: Layout,
	pub(crate) drop: unsafe fn(*mut u8),
}

pub struct ComponentRegistry {
	pub(crate) components: HashMap<ComponentID, ComponentInfo>,
}

impl ComponentRegistry {
	pub fn new() -> Self {
		ComponentRegistry { components: HashMap::new() }
	}

	pub fn register<C: Component>(&mut self) {
		let id = ComponentID::of::<C>();
		let component_info = ComponentInfo {
			name: core::any::type_name::<C>(),
			layout: Layout::new::<Slot<C>>(),
			drop: unsafe { core::mem::transmute(core::ptr::drop_in_place::<Slot<C>> as *mut u8) },
		};
		self.components.insert(id, component_info);
	}

	pub fn finalize(self) -> Rc<Self> { Rc::new(self) }
}
