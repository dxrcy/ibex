use crate::compose::Attribute;
use crate::dom::{Document, DomNode};

pub fn render(page: Document) -> String {
    format!(
        concat!(
            r"<!DOCTYPE html>",
            r"<html>",
            r"<head>",
            r"{}",
            r"</head>",
            r"<body>",
            r"{}",
            r"</body>",
            r"</html>",
        ),
        // ignores attributes!
        render_nodes(page.head.children),
        render_nodes(page.body.children),
    )
}

fn render_nodes(nodes: Vec<DomNode>) -> String {
    nodes
        .into_iter()
        .map(|node| render_node(node))
        .collect::<Vec<_>>()
        .join("")
}

fn render_node(node: DomNode) -> String {
    match node {
        DomNode::Element(element) => {
            format!(
                "<{tag}{attrs}>{content}</{tag}>",
                tag = element.tag,
                attrs = format_attributes(element.attributes),
                content = render_nodes(element.children),
            )
        }
        DomNode::Text(text) => text,
    }
}

fn format_attributes(attributes: Vec<Attribute>) -> String {
    if attributes.is_empty() {
        return String::new();
    }

    " ".to_string()
        + &attributes
            .into_iter()
            .map(|attribute| match attribute.value {
                None => attribute.name,
                Some(value) => attribute.name + "=\"" + &value + "\"",
            })
            .collect::<Vec<_>>()
            .join(" ")
}
