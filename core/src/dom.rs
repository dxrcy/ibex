use crate::compose::{Attribute, Node, Tag, View};
use crate::render::render;

/// HTML document to render to string for `.html` file
#[derive(Clone, Debug)]
pub struct Document {
    /// <head>
    pub(super) head: DomElement,
    /// <body>
    pub(super) body: DomElement,
}

/// HTML node
#[derive(Clone, Debug)]
pub(super) enum DomNode {
    /// `DomElement`
    Element(DomElement),
    /// Text node
    Text(String),
}

/// HTML element
#[derive(Clone, Debug)]
pub(super) struct DomElement {
    pub(super) tag: Tag,
    pub(super) attributes: Vec<Attribute>,
    pub(super) children: Vec<DomNode>,
}

/// Convert a `View` to a `Document`
pub fn convert(view: View) -> Document {
    // Empty <head> to push elements onto
    let mut head = DomElement {
        tag: Tag::Head,
        attributes: vec![],
        children: vec![],
    };

    // Convert <body>
    let body = DomElement {
        tag: Tag::Body,
        attributes: vec![],
        children: convert_nodes(view, &mut head),
    };

    Document { head, body }
}

/// Convert multiple nodes (as a `View`) to DOM nodes
fn convert_nodes(view: View, head: &mut DomElement) -> Vec<DomNode> {
    view.0
        .into_iter()
        .map(|node| convert_node(node, head))
        .collect::<Vec<_>>()
        .concat()
}
/// Convert a `Node` to `DomNode`s
fn convert_node(node: Node, head: &mut DomElement) -> Vec<DomNode> {
    match node {
        // Add nodes to <head>
        // Return nothing
        Node::HeadAppend(view) => {
            for node in view.0 {
                let mut node = convert_node(node, head);
                head.children.append(&mut node)
            }
            vec![]
        }

        // Recursively convert `Element` to `DomElement`
        Node::Element(element) => vec![DomNode::Element(DomElement {
            tag: element.tag,
            attributes: element.attributes,
            children: convert_nodes(element.children, head),
        })],

        Node::Fragment(view) => convert_nodes(view, head),
        Node::Text(text) => vec![DomNode::Text(text)],
    }
}

// ---------------------
// Handy implementations
// ---------------------

impl View {
    pub fn document(self) -> Document {
        convert(self)
    }
    pub fn render(self) -> String {
        self.document().render()
    }
}
impl From<View> for Document {
    fn from(value: View) -> Self {
        value.document()
    }
}
impl Document {
    pub fn render(self) -> String {
        render(self)
    }
}
