use crate::compose::{Attribute, Node, Tag, View};
use crate::render::{render, render_nodes};

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

/// Convert multiple nodes (as a `View`) to DOM nodes
///
/// Panics if any node is `HEAD`
fn convert_nodes_headless(view: View) -> Vec<DomNode> {
    view.0
        .into_iter()
        .map(|node| convert_node_headless(node))
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

/// Convert a `Node` to `DomNode`s
///
/// Panics if any node is `HEAD`
fn convert_node_headless(node: Node) -> Vec<DomNode> {
    match node {
        Node::HeadAppend(_) => panic!("Cannot use `HEAD` without rendering as `Document`"),

        // Recursively convert `Element` to `DomElement`
        Node::Element(element) => vec![DomNode::Element(DomElement {
            tag: element.tag,
            attributes: element.attributes,
            children: convert_nodes_headless(element.children),
        })],

        Node::Fragment(view) => convert_nodes_headless(view),
        Node::Text(text) => vec![DomNode::Text(text)],
    }
}

// ---------------------
// Handy implementations
// ---------------------

impl View {
    /// Convert to a `Document`
    pub fn document(self) -> Document {
        convert(self)
    }
    /// Convert to a `Document` and render to HTML file
    pub fn render_document(self) -> String {
        self.document().render()
    }

    /// Render nodes as string, without converting to `Document`
    ///
    /// Does not include `body`, `head`, or `html` tags.
    /// This only renders the view
    pub fn render_orphan(self) -> String {
        let nodes = convert_nodes_headless(self);
        render_nodes(nodes)
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
