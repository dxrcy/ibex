/// Some built in features
pub mod extras;
/// Routing and file management for SSG apps
pub mod ssg;

pub use core::*;
pub use extras::{is_local, use_meta, Meta};
pub use macros::view;

pub mod prelude {
    pub use core::{
        compose::{Attribute, Element, Node, Tag, View},
        dom::Document,
    };

    pub use macros::view;

    pub use crate::extras::{is_local, use_meta, Meta};
    pub use crate::url;
}
