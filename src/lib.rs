#![no_std]
extern crate alloc;

mod component;
pub use component::{Component, ComponentRegistry};

mod ecs;
pub use ecs::{EntityID, ECS};

mod query;
pub use query::{Query, QueryMut, QueryIter, QueryMutIter};

mod storage;