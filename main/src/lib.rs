/// Some built in features
pub mod extras;
/// Routing and file management for SSG apps
pub mod ssg;

pub use extras::{is_local, use_meta, Meta};
pub use ibex_core::*;
pub use ibex_macros::{document, view};

pub mod prelude {
    pub use ibex_core::{
        compose::{Attribute, Element, Node, Tag, View},
        dom::Document,
    };

    pub use ibex_macros::{document, view};

    pub use crate::extras::{is_local, use_meta, Meta};
    pub use crate::url;
}
