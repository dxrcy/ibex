use crate::compose::{Attribute, Tag};
use crate::dom::{Document, DomElement, DomNode};

pub fn render(page: Document) -> String {
    if !page.head.attributes.is_empty() || !page.body.attributes.is_empty() {
        panic!("Cannot use attributes on <head> or <body> tags (how did you even get this error?)");
    }

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
        DomNode::Element(element) => render_element(element),
        DomNode::Text(text) => text,
    }
}

fn render_element(element: DomElement) -> String {
    match element.tag {
        Tag::Br => {
            if !element.attributes.is_empty() || !element.children.is_empty() {
                panic!("Cannot use attributes or children on <br> tag");
            }
            "<br>".to_string()
        }
        _ => format!(
            "<{tag}{attrs}>{content}</{tag}>",
            tag = element.tag,
            attrs = format_attributes(element.attributes),
            content = render_nodes(element.children),
        ),
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
