use crate::compose::Attribute;
use crate::dom::{Document, DomElement, DomNode};

/// Render a `Document` to a HTML string
pub fn render(page: Document) -> String {
    if !page.head.attributes.is_empty() || !page.body.attributes.is_empty() {
        panic!("Cannot use attributes on <head> or <body> tags (how did you even get this error?)");
    }

    format!(
        concat!(
            r"<!DOCTYPE html>",
            r"<html{lang}>",
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
        lang = match page.lang {
            Some(lang) => format!(" lang=\"{}\"", lang),
            None => "".to_string(),
        },
    )
}

/// Render multiple DOM nodes
pub(super) fn render_nodes(nodes: Vec<DomNode>) -> String {
    nodes
        .into_iter()
        .map(render_node)
        .collect::<Vec<_>>()
        .join("")
}
/// Render a single DOM node
fn render_node(node: DomNode) -> String {
    match node {
        DomNode::Element(element) => render_element(element),
        DomNode::Text(text) => text,
    }
}

/// Render a DOM element to HTML string
fn render_element(element: DomElement) -> String {
    if element.tag.is_void() {
        if !element.children.is_empty() {
            panic!("Void tag <{}> cannot contain children", element.tag);
        }
        format!(
            "<{tag}{attrs}>",
            tag = element.tag,
            attrs = format_attributes(element.attributes),
        )
    } else {
        format!(
            "<{tag}{attrs}>{content}</{tag}>",
            tag = element.tag,
            attrs = format_attributes(element.attributes),
            content = render_nodes(element.children),
        )
    }
}

/// Render attributes in key="value" format
fn format_attributes(attributes: Vec<Attribute>) -> String {
    if attributes.is_empty() {
        return String::new();
    }
    // Space to separate from tag name
    " ".to_string()
        + &attributes
            .into_iter()
            .map(format_attribute_value)
            .collect::<Vec<_>>()
            .join(" ")
}

fn format_attribute_value(attribute: Attribute) -> String {
    match attribute.value {
        Some(value) => attribute.name + "=\"" + &value + "\"",
        None => attribute.name,
    }
}
