use crate::compose::{Attribute, Node, Tag, View};
use crate::render::render;

#[derive(Clone, Debug)]
pub struct Document {
    pub(super) head: DomElement,
    pub(super) body: DomElement,
}

#[derive(Clone, Debug)]
pub(super) enum DomNode {
    Element(DomElement),
    Text(String),
}

#[derive(Clone, Debug)]
pub(super) struct DomElement {
    pub(super) tag: Tag,
    pub(super) attributes: Vec<Attribute>,
    pub(super) children: Vec<DomNode>,
}

impl From<View> for Document {
    fn from(value: View) -> Self {
        convert(value)
    }
}

impl Document {
    pub fn render(self) -> String {
        render(self)
    }
}

pub fn convert(view: View) -> Document {
    let mut head = DomElement {
        tag: Tag::Head,
        attributes: vec![],
        children: vec![],
    };

    let body = DomElement {
        tag: Tag::Body,
        attributes: vec![],
        children: convert_nodes(view, &mut head),
    };

    Document { head, body }
}

fn convert_nodes(view: View, head: &mut DomElement) -> Vec<DomNode> {
    view.0
        .into_iter()
        .map(|node| convert_node(node, head))
        .collect::<Vec<_>>()
        .concat()
}

fn convert_node(node: Node, head: &mut DomElement) -> Vec<DomNode> {
    match node {
        Node::HeadAppend(view) => {
            for node in view.0 {
                let mut node = convert_node(node, head);
                head.children.append(&mut node)
            }
            vec![]
        }

        Node::Element(element) => vec![DomNode::Element(DomElement {
            tag: element.tag,
            attributes: element.attributes,
            children: convert_nodes(element.children, head),
        })],

        Node::Fragment(view) => convert_nodes(view, head),

        Node::Text(text) => vec![DomNode::Text(text)],
    }
}
