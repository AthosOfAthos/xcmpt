
use core::alloc::Layout;
use hashbrown::HashMap;
use crate::storage::Slot;

pub trait Component: Sized + 'static {}

pub(crate) type ComponentID = core::any::TypeId;

#[derive(Clone, Copy)]
pub(crate) struct ComponentInfo {
	pub(crate) layout: Layout,
	pub(crate) stride: usize,
	pub(crate) drop: unsafe fn(*mut u8),
}

impl ComponentInfo {
	pub(crate) const fn new<C: Component>() -> Self {
		let layout = Layout::new::<Slot<C>>();
		let stride = layout.size() + (layout.size() % layout.align());
		let drop = unsafe { core::mem::transmute(core::ptr::drop_in_place::<Slot<C>> as *mut u8) };
		ComponentInfo { layout, stride, drop }
	}
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
		let component_info = ComponentInfo::new::<C>();
		self.components.insert(id, component_info);
	}
}
