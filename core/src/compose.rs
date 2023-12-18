/// List of component nodes
#[derive(Clone, Debug, Default)]
pub struct View(pub Vec<Node>);

/// Abstract component node
#[derive(Clone, Debug)]
pub enum Node {
    HeadAppend(View),
    Element(Element),
    Fragment(View),
    Text(String),
}

/// Html-like element
#[derive(Clone, Debug)]
pub struct Element {
    pub tag: Tag,
    pub attributes: Vec<Attribute>,
    pub children: View,
}

/// Shorthand to define `enum Tag` with methods to convert into and from a string
macro_rules! define_tag {
    ( $( $ident:ident $str:literal ),* $(,)? ) => {
        /// Html tag for `Element` and `DomElement`
        #[derive(Clone, Copy, Debug)]
        pub enum Tag { $(
            /// Html tag
            $ident,
        )* }

        impl Into<&'static str> for Tag {
            fn into(self) -> &'static str {
                match self {
                    $( Self::$ident => $str,)*
                }
            }
        }
        impl TryFrom<&str> for Tag {
            type Error = ();
            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Ok(match value {
                    $( $str => Self::$ident, )*
                    _ => return Err(()),
                })
            }
        }
    };
}

define_tag! {
    A "a", Abbr "abbr", Address "address", Article "article", Aside "aside", Audio "audio", B "b", Base "base", Bdi "bdi", Bdo "bdo", Big "big", Blockquote "blockquote", Body "body", Br "br", Button "button", Caption "caption", Center "center", Cite "cite", Code "code", Col "col", Colgroup "colgroup", Data "data", Datalist "datalist", Dd "dd", Del "del", Details "details", Dfn "dfn", Dialog "dialog", Div "div", Dl "dl", Dt "dt", Em "em", Embed "embed", Fieldset "fieldset", Figcaption "figcaption", Figure "figure", Footer "footer", Form "form", H1 "h1", H2 "h2", H3 "h3", H4 "h4", H5 "h5", H6 "h6", Head "head", Header "header", Hr "hr", Html "html", I "i", Iframe "iframe", Img "img", Input "input", Ins "ins", Kbd "kbd", Label "label", Legend "legend", Li "li", Link "link", Main "main", Map "map", Mark "mark", Meta "meta", Meter "meter", Nav "nav", Noscript "noscript", Object "object", Ol "ol", Optgroup "optgroup", Option "option", Output "output", P "p", Param "param", Picture "picture", Pre "pre", Progress "progress", Q "q", Rp "rp", Rt "rt", Ruby "ruby", S "s", Samp "samp", Script "script", Section "section", Select "select", Small "small", Source "source", Span "span", Strong "strong", Style "style", Sub "sub", Summary "summary", Sup "sup", Svg "svg", Table "table", Tbody "tbody", Td "td", Template "template", Textarea "textarea", Tfoot "tfoot", Th "th", Thead "thead", Time "time", Title "title", Tr "tr", Track "track", U "u", Ul "ul", Var "var", Video "video", Wbr "wbr"
}

/// Html attribute for `Element` and `DomElement`
#[derive(Clone, Debug)]
pub struct Attribute {
    pub name: String,
    pub value: String,
}

// ---------------------
// Handy implementations
// ---------------------

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string: &str = (*self).into();
        write!(f, "{}", string)
    }
}

impl From<View> for Node {
    fn from(value: View) -> Self {
        Self::Fragment(value)
    }
}

impl From<String> for Node {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}
impl From<&String> for Node {
    fn from(value: &String) -> Self {
        Self::Text(value.to_owned())
    }
}
impl From<&str> for Node {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}
impl From<char> for Node {
    fn from(value: char) -> Self {
        Self::Text(value.to_string())
    }
}
impl From<&char> for Node {
    fn from(value: &char) -> Self {
        Self::Text(value.to_string())
    }
}
impl From<()> for Node {
    fn from(_: ()) -> Self {
        Self::Fragment(View(vec![]))
    }
}

impl<T> From<Vec<T>> for Node
where
    T: Into<Node>,
{
    fn from(value: Vec<T>) -> Self {
        Self::Fragment(View(value.into_iter().map(Into::into).collect()))
    }
}
impl<T> From<Option<T>> for Node
where
    T: Into<Node>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => value.into(),
            None => Self::Fragment(View(vec![])),
        }
    }
}

impl From<String> for View {
    fn from(value: String) -> Self {
        Self(vec![value.into()])
    }
}
impl From<&String> for View {
    fn from(value: &String) -> Self {
        Self(vec![value.into()])
    }
}
impl From<&str> for View {
    fn from(value: &str) -> Self {
        Self(vec![value.into()])
    }
}
impl From<()> for View {
    fn from(_: ()) -> Self {
        View(vec![])
    }
}

impl<T> From<Vec<T>> for View
where
    T: Into<Node>,
{
    fn from(value: Vec<T>) -> Self {
        View(value.into_iter().map(Into::into).collect())
    }
}
impl<T> From<Option<T>> for View
where
    T: Into<Node>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => View(vec![value.into()]),
            None => View(vec![]),
        }
    }
}

impl View {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Implement `Into<Node>` for some select types which implement `Display` (`ToString`)
/// Cannot use T: Display, because `Vec<T>` conflict (`Vec<T>` pseudo-implements `Display` for
///   future-proofing)
macro_rules! impl_with_display {
    ( $( $ty:ty ),* ) => {
        $(
            impl From<$ty> for Node {
                fn from(value: $ty) -> Self {
                    Node::Text(value.to_string())
                }
            }
            impl From<$ty> for View {
                fn from(value: $ty) -> Self {
                    View(vec![Node::Text(value.to_string())])
                }
            }
        )*
    };
}
impl_with_display![u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize];
