use crate::compose::Attribute;
use crate::compose::Node;
use crate::compose::Tag;
use crate::compose::View;
use crate::render::render;

#[derive(Clone, Debug)]
pub struct Document {
    pub head: DomElement,
    pub body: DomElement,
}

#[derive(Clone, Debug)]
pub enum DomNode {
    Element(DomElement),
    Text(String),
}

#[derive(Clone, Debug)]
pub struct DomElement {
    pub tag: Tag,
    pub attributes: Vec<Attribute>,
    pub children: Vec<DomNode>,
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

pub fn convert_nodes(view: View, head: &mut DomElement) -> Vec<DomNode> {
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
