
use core::alloc::Layout;
use hashbrown::HashMap;
use crate::storage::Slot;

pub trait Component: Sized + 'static {}

pub(crate) type ComponentID = core::any::TypeId;

#[derive(Clone, Copy)]
pub(crate) struct ComponentInfo {
	pub(crate) name: &'static str,
	pub(crate) layout: Layout,
	pub(crate) stride: usize,
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
		let layout = Layout::new::<Slot<C>>();
		let stride = layout.size() + (layout.size() % layout.align());
		let component_info = ComponentInfo {
			name: core::any::type_name::<C>(),
			layout,
			stride,
			drop: unsafe { core::mem::transmute(core::ptr::drop_in_place::<Slot<C>> as *mut u8) },
		};
		self.components.insert(id, component_info);
	}
}
