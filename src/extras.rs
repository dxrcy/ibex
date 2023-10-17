use crate as ibex;
use ibex::{compose::View, view};

/// Returns `true` if first CLI argument is `local`
///
/// Useful for `url!` and development/production conditions
pub fn is_local() -> bool {
    std::env::args().nth(1) == Some("local".to_string())
}

/// Returns the input appended to `URL_ROOT`
///
/// Note that `URL_ROOT`:
///  - Must be defined
///  - Must be in scope where macro is called
///  - Should BEGIN and END with a slash. Eg. `const URL_ROOT: &str = "/example-site/"`
///  - May be `"/"` (same as development) for sites hosted at root URL path
///
/// This macro will always resolve to a `String`
#[macro_export]
macro_rules! url {
    () => {
        url!(@root)
    };
    ($path:expr) => {
        url!(@root) + &$path
    };
    (@root) => {
        if ::ibex::extras::is_local() { "/" } else { URL_ROOT }.to_string()
    };
}

/// Shorthand to define `struct Meta` with fields and matching builder functions
macro_rules! define_meta {
    ( $(
        #[$($meta:tt)*]
        $name:ident
    ),* $(,)? ) => {
        /// Construct many <meta> tag for `use_meta`
        ///
        /// Includes aliases of similar tags, eg. `url` and `og:url`
        #[derive(Debug, Default, Clone)]
        pub struct Meta { $(
            $name: Option<String>,
        )* }

        impl Meta { $(
            /// Add a <meta> tag group
            ///
            /// Data names:
            #[$($meta)*]
            pub fn $name(mut self, $name: impl Into<String>) -> Self {
                self.$name = Some($name.into());
                self
            }
        )* }
    };
}

define_meta! {
    /// `url`, `og:url`
    url,
    /// `name`, `og:title`, `title`
    title,
    /// `description`, `description`, `og:description`, `twitter:description`
    desc,
    /// `image`, `image`, `og:image`, `twitter:image`
    image,
    /// `author`
    author,
    /// `theme-color`
    color,
}

impl Meta {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Include many <meta> tags in <head>, for charset and SEO
///
/// Includes aliases of similar tags, eg. `url` and `og:url`
pub fn use_meta(meta: Meta) -> View {
    view! {
        HEAD {
            meta [charset="utf-8"]/
            meta [name="viewport", content="width=device-width, initial-scale=1"]/

            [if let Some(url) = meta.url { view!{
                meta [name="url",        content=url]/
                meta [property="og:url", content=url]/
            }} else { view! {}}]

            [if let Some(title) = meta.title { view!{
                meta [itemprop="name",     content=title]/
                meta [property="og:title", content=title]/
                meta [name="title",        content=title]/
            }} else { view! {}}]

            [if let Some(desc) = meta.desc{ view!{
                meta [name="description",         content=desc]/
                meta [itemprop="description",     content=desc]/
                meta [property="og:description",  content=desc]/
                meta [name="twitter:description", content=desc]/
            }} else { view! {}}]

            [if let Some(image) = meta.image { view!{
                meta [name="image",         content=image]/
                meta [itemprop="image",     content=image]/
                meta [property="og:image",  content=image]/
                meta [name="twitter:image", content=image]/
            }} else { view! {}}]

            [if let Some(author) = meta.author { view!{
                meta [name="author", content=author]/
            }} else { view! {}}]

            [if let Some(color) = meta.color { view!{
                meta [name="theme-color", content=color]/
            }} else { view! {}}]

            meta [property="og:type", content="website"]/
            meta [name="twitter:card", content="summary_large_image"]/
        }
    }
}
