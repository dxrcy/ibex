// Re-export all modules (no items)
pub use core::*;

pub mod prelude {
    pub use core::compose::{Attribute, Element, Node, Tag, View};
    pub use core::dom::Document;
    pub use core::route;
    pub use core::routes;
    pub use macros::view;
}
