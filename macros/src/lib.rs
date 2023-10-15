#![allow(unused)]

use proc_macro as pm1;

use proc_macro2::{Delimiter, Punct, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use syn::{braced, parse::Parse, parse_macro_input, token::Token, Item, Lit, Token};

use core::compose::Tag;

#[proc_macro]
pub fn view(input: pm1::TokenStream) -> pm1::TokenStream {
    let view = parse_view(input.into());
    quote! { #view }.into()
}

#[derive(Debug)]
struct View(Vec<Node>);

#[derive(Debug)]
enum Node {
    HeadAppend(View),
    Element(Element),
    Literal(String),
    Expression(TokenStream),
    Function(Function),
    If(TokenStream, View, Option<View>),
    For(TokenStream, TokenStream, View),
}

#[derive(Debug)]
struct Element {
    tag: Tag,
    attributes: Vec<Attribute>,
    children: View,
}

#[derive(Debug)]
struct Function {
    name: String,
    arguments: TokenStream,
    children: Option<View>,
}

#[derive(Debug)]
struct Attribute {
    name: String,
    value: Option<TokenStream>,
}

impl ToTokens for View {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut vec_tokens = TokenStream::new();
        for node in &self.0 {
            vec_tokens.extend(quote! {
                #node,
            });
        }
        tokens.extend(quote! {
            ibex::compose::View(vec![#vec_tokens])
        });
    }
}

impl ToTokens for Node {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Node::HeadAppend(view) => {
                tokens.extend(quote! { ibex::compose::Node::HeadAppend(#view) })
            }
            Node::Element(element) => {
                tokens.extend(quote! { ibex::compose::Node::Element(#element) })
            }
            Node::Literal(string) => {
                tokens.extend(quote! { ibex::compose::Node::Text(#string.to_string()) })
            }
            Node::Expression(content) => {
                let expr = quote! { #content };
                tokens.extend(quote! { ibex::compose::Node::from(#expr) })
            }
            Node::Function(Function {
                name,
                arguments,
                children,
            }) => {
                // Convert string of ident into ident
                let name = quote::format_ident!("{}", format!("{}", name));
                let call = match children {
                    Some(children) => {
                        if arguments.is_empty() {
                            quote! { #name(#children) }
                        } else {
                            quote! { #name(#arguments, #children) }
                        }
                    }
                    None => {
                        quote! { #name(#arguments) }
                    }
                };
                tokens.extend(quote! {ibex::compose::Node::Fragment(#call)})
            }
            Node::If(condition, then, otherwise) => match otherwise {
                Some(otherwise) => tokens.extend(quote! {
                ibex::compose::Node::Fragment(
                    if #condition {
                        #then
                    } else {
                        #otherwise
                    }
                )}),
                None => tokens.extend(quote! {
                ibex::compose::Node::Fragment(
                    if #condition {
                        #then
                    } else {
                        view! {}
                    }
                )}),
            },
            Node::For(item, source, block) => tokens.extend(quote! {
                ibex::compose::Node::Fragment(ibex::compose::View(
                    #source.map(|(#item)| {
                        ibex::compose::Node::Fragment(#block)
                    }).collect::<Vec<_>>()
            ))}),
        }
    }
}

impl ToTokens for Element {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Element {
            tag,
            attributes,
            children,
        } = self;

        // Convert string of ident into ident
        let tag = quote::format_ident!("{}", format!("{:?}", tag));

        let mut children_tokens = TokenStream::new();
        children.to_tokens(&mut children_tokens);

        let mut attributes_tokens = TokenStream::new();
        for Attribute { name, value } in attributes {
            let value = match value {
                Some(value) => quote! { Some((#value).to_string()) },
                None => quote! { None },
            };
            attributes_tokens.extend(quote! {
                ibex::compose::Attribute {
                    name: #name.to_string(),
                    value: #value,
                },
            });
        }

        tokens.extend(quote! {
            ibex::compose::Element {
                tag: ibex::compose::Tag::#tag,
                attributes: vec![ #attributes_tokens ],
                children: #children_tokens,
            },
        });
    }
}

fn parse_view(input: TokenStream) -> View {
    let mut nodes = Vec::new();

    let mut tokens = input.into_iter().peekable();

    while let Some(token) = tokens.next() {
        match token {
            TokenTree::Ident(ident) => {
                let tag = ident.to_string();

                let tag = if tag == "HEAD" {
                    if !nodes.is_empty() {
                        panic!("HEAD must be first element in group");
                    }
                    None
                } else {
                    let Ok(tag) = tag.as_str().try_into() else {
                        panic!("Invalid tag name '{}'", tag);
                    };
                    Some(tag)
                };

                let attributes = match tokens.peek() {
                    Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Bracket => {
                        let mut group = group.stream().into_iter().peekable();
                        let mut attributes = Vec::new();

                        loop {
                            let Some(name) = group.next() else {
                                panic!("Missing attribute name");
                            };
                            let name = name.to_string();

                            let value = match group.next() {
                                Some(TokenTree::Punct(punct)) if punct.to_string() == "=" => {
                                    let Some(value) = group.next() else {
                                        panic!("Missing attribute value");
                                    };
                                    let value = parse_value(value.to_token_stream());
                                    Some(value)
                                }
                                Some(TokenTree::Punct(punct)) if punct.to_string() == "," => None,
                                None => None,
                                Some(token) => panic!(
                                    "Unexpected token `{}`. Expected one of `]`, `,`, or `=`",
                                    token
                                ),
                            };

                            attributes.push(Attribute { name, value });

                            if let Some(TokenTree::Punct(punct)) = group.peek() {
                                if punct.to_string() == "," {
                                    group.next();
                                }
                            }

                            if group.peek().is_none() {
                                break;
                            }
                        }

                        tokens.next();
                        attributes
                    }
                    _ => Vec::new(),
                };

                let Some(next) = tokens.next() else {
                    panic!("Unexpected end. Missing braces or single slash.");
                };

                let children = match next {
                    TokenTree::Group(group) => {
                        if group.delimiter() != Delimiter::Brace {
                            panic!("Group must have braces: {{...}}");
                        }

                        parse_view(group.stream())
                    }

                    TokenTree::Punct(punct) if punct.to_string() == "/" => View(Vec::new()),

                    _ => panic!("Expected group or single slash"),
                };

                match tag {
                    None => nodes.push(Node::HeadAppend(children)),
                    Some(tag) => nodes.push(Node::Element(Element {
                        tag,
                        attributes,
                        children,
                    })),
                }
            }

            TokenTree::Literal(literal) => match Lit::new(literal) {
                Lit::Str(string) => {
                    nodes.push(Node::Literal(string.value()));
                }
                _ => panic!("Only string literals are allowed"),
            },

            TokenTree::Punct(punct) if punct.to_string() == "@" => {
                let Some(TokenTree::Ident(name)) = tokens.next() else {
                    panic!("Missing name for node function");
                };
                let name = name.to_string();

                let arguments = match tokens.peek() {
                    Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Bracket => {
                        let arguments = group.stream();
                        tokens.next();
                        arguments
                    }
                    _ => TokenStream::new(),
                };

                // Peek next token, don't consume unless matched
                let children = match tokens.peek() {
                    Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                        // Parse, and then consume iterator item, then returned parsed value
                        let children = Some(parse_view(group.stream()));
                        tokens.next();
                        children
                    }
                    _ => None,
                };

                nodes.push(Node::Function(Function {
                    name,
                    arguments,
                    children,
                }));
            }

            TokenTree::Group(group) if group.delimiter() == Delimiter::Bracket => {
                let mut stream = group.stream().into_iter().peekable();

                // Special `if` and `for` statements
                if let Some(TokenTree::Punct(punct)) = stream.peek() {
                    if punct.to_string() == "*" {
                        stream.next();

                        let Some(TokenTree::Ident(statement)) = stream.next() else {
                            panic!("Missing or invalid statement");
                        };
                        println!("'{}'", statement);
                        match statement.to_string().as_str() {
                            "if" => {
                                let condition = match stream.next() {
                                    Some(TokenTree::Group(group))
                                        if group.delimiter() == Delimiter::Parenthesis =>
                                    {
                                        group.stream()
                                    }
                                    _ => panic!(
                                        "`if` statement condition must be a group with parenthesis"
                                    ),
                                };

                                // if block
                                let then = match stream.next() {
                                    Some(TokenTree::Group(group))
                                        if group.delimiter() == Delimiter::Brace =>
                                    {
                                        parse_view(group.stream())
                                    }
                                    _ => panic!("`if` statement block must be a group with braces"),
                                };

                                // else block
                                let otherwise = match stream.next() {
                                    Some(TokenTree::Ident(ident))
                                        if ident.to_string() == "else" =>
                                    {
                                        match stream.next() {
                                            Some(TokenTree::Group(group))
                                                if group.delimiter() == Delimiter::Brace =>
                                            {
                                                Some(parse_view(group.stream(), ))
                                            }
                                            _=> panic!(
                                                "`if-else` statement block must be a group with braces"
                                            ),
                                        }
                                    }
                                    Some(token) => panic!(
                                        "Unexpected token `{}` after `if` statement block",
                                        token
                                    ),
                                    None => None,
                                };

                                nodes.push(Node::If(condition, then, otherwise));
                            }

                            "for" => {
                                // item
                                let item = match stream.next() {
                                    Some(TokenTree::Group(group))
                                        if group.delimiter() == Delimiter::Parenthesis =>
                                    {
                                        group.stream()
                                    }
                                    _ => panic!(
                                        "`for` statement item must be a group with parenthesis"
                                    ),
                                };

                                // `in`
                                match stream.next() {
                                    Some(TokenTree::Ident(ident)) if ident.to_string() == "in" => {}
                                    _ => panic!("`for` statement must have `in` keyword"),
                                }

                                // source
                                let source = match stream.next() {
                                    Some(TokenTree::Group(group))
                                        if group.delimiter() == Delimiter::Parenthesis =>
                                    {
                                        group.stream()
                                    }
                                    _ => panic!(
                                        "`for` statement source must be a group with parenthesis"
                                    ),
                                };

                                // for block
                                let block = match stream.next() {
                                    Some(TokenTree::Group(group))
                                        if group.delimiter() == Delimiter::Brace =>
                                    {
                                        parse_view(group.stream())
                                    }
                                    _ => {
                                        panic!("`for` statement block must be a group with braces")
                                    }
                                };

                                nodes.push(Node::For(item, source, block));
                            }

                            _ => panic!("Invalid statement"),
                        }
                        continue;
                    }
                }

                nodes.push(Node::Expression(stream.collect()));
            }

            TokenTree::Punct(punct) if punct.to_string() == "~" => {
                nodes.push(Node::Literal(" ".to_string()));
            }

            _ => {
                panic!("Unexpected token {:#?}", token);
            }
        }
    }

    View(nodes)
}

/// If input is a square bracket group starting with tokens `:?`,
///     wrap the rest of the group in a debug format
/// If not, return input string
fn parse_value(input: TokenStream) -> TokenStream {
    let mut tokens = input.clone().into_iter();

    // (awful code)
    if let Some(TokenTree::Group(group)) = tokens.next() {
        let mut stream = group.stream().into_iter();
        if let Some(TokenTree::Punct(punct)) = stream.next() {
            if punct.to_string() == ":" {
                if let Some(TokenTree::Punct(punct)) = stream.next() {
                    if punct.to_string() == "?" {
                        let rest: TokenStream = stream.collect();
                        return quote! { format!("{:?}", #rest) };
                    }
                }
            }
        }
    }

    input
}
