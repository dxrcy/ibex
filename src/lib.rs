mod extra;

pub use core::*;
pub use extra::*;
pub use macros::view;

pub mod prelude {
    pub use core::{
        compose::{Attribute, Element, Node, Tag, View},
        dom::Document,
        route, routes,
    };
     
    pub use macros::view;

    pub use crate::extra::{use_meta, Meta};
}
