/// Some built in features
pub mod extras;

pub use core::*;
pub use extras::*;
pub use macros::view;

pub mod prelude {
    pub use core::{
        compose::{Attribute, Element, Node, Tag, View},
        dom::Document,
        route, routes,
    };

    pub use macros::view;

    pub use crate::extras::{is_local, use_meta, Meta};
    pub use crate::url;
}
