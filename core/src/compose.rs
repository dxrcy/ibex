#[derive(Clone, Debug)]
pub struct View(pub Vec<Node>);

#[derive(Clone, Debug)]
pub enum Node {
    HeadAppend(View),
    Element(Element),
    Fragment(View),
    Text(String),
}

#[derive(Clone, Debug)]
pub struct Element {
    pub tag: Tag,
    pub attributes: Vec<Attribute>,
    pub children: View,
}

macro_rules! element_tag {
    ( $( $ident:ident $str:literal ),* $(,)? ) => {
        #[derive(Clone, Copy, Debug)]
        pub enum Tag {
            $( $ident, )*
        }

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

#[derive(Clone, Debug)]
pub struct Attribute {
    pub name: String,
    pub value: Option<String>,
}

element_tag! {
    H1    "h1",
    H2    "h2",
    H3    "h3",
    P     "p",
    Span  "span",
    Div   "div",
    Body  "body",
    Head  "head",
    Link  "link",
    Title "title",
    Ul    "ul",
    Li    "li",
    A     "a",
    Br    "br",
    Input "input",
    B     "b",
    I     "i",
    Small "small",
    Image "image",
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string: &str = (*self).into();
        write!(f, "{}", string)
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
impl From<()> for Node {
    fn from(_: ()) -> Self {
        Self::Fragment(View(vec![]))
    }
}
impl From<View> for Node {
    fn from(value: View) -> Self {
        Self::Fragment(value)
    }
}

impl<T> From<Vec<T>> for Node
where
    T: Into<Node>,
{
    fn from(value: Vec<T>) -> Self {
        Node::Fragment(View(value.into_iter().map(Into::into).collect()))
    }
}

impl<T> From<Option<T>> for Node
where
    T: Into<Node>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => value.into(),
            None => Node::Fragment(View(vec![])),
        }
    }
}
