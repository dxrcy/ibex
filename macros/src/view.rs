use proc_macro2::{Delimiter, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use syn::Lit;

use ibex_core::compose::Tag;

#[derive(Debug)]
pub struct View(Vec<Node>);

#[derive(Debug)]
enum Node {
    HeadAppend(View),
    Element(Element),
    Literal(String),
    Expression(TokenStream),
    Function(Function),
    If(TokenStream, View, Option<View>),
    For(TokenStream, View),
    With(TokenStream, View),
}

#[derive(Debug)]
struct Element {
    tag: Tag,
    id: Option<TokenStream>,
    class: Option<TokenStream>,
    attributes: Vec<Attribute>,
    children: View,
}

#[derive(Debug)]
struct Function {
    name: TokenStream,
    arguments: TokenStream,
    children: Option<View>,
}

#[derive(Debug)]
enum Attribute {
    Pair {
        name: String,
        value: TokenStream,
    },
    Conditional {
        name: String,
        condition: TokenStream,
    },
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
                tokens.extend(quote! { ibex::compose::Node::HeadAppend(#view.into()) })
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
                // let name = quote::format_ident!("{}", format!("{}", name));
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
                tokens.extend(quote! {ibex::compose::Node::Fragment(#call.into())})
            }
            // conditions cannot be wrapped in brackets, because `if-let` statements will break
            // this, however, makes errors look ugly with `else-if` chaining
            Node::If(condition, then, otherwise) => match otherwise {
                Some(otherwise) => tokens.extend(quote! {
                    ibex::compose::Node::Fragment(
                        if #condition {
                            #then
                        } else {
                            #otherwise
                        }
                    )
                }),
                None => tokens.extend(quote! {
                    ibex::compose::Node::Fragment(
                        if #condition {
                            #then
                        } else {
                            view! {}
                        }
                    )
                }),
            },
            // must use `for` loop inside block, as opposed to `.map`, because `#source` is a
            // tokenstream, which does not separate tokens before and after `in` keyword
            Node::For(source, block) => tokens.extend(quote! {
                ibex::compose::Node::Fragment({
                    let mut items = Vec::new();
                    for #source {
                        items.push( ibex::compose::Node::Fragment(#block) );
                    }
                    ibex::compose::View(items)
                })
            }),

            Node::With(scope, view) => tokens.extend(quote! {
                {
                    #scope;
                    ibex::compose::Node::Fragment(#view)
                }
            }),
        }
    }
}

impl ToTokens for Element {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Element {
            tag,
            id,
            class,
            attributes,
            children,
        } = self;

        // Convert string of ident into ident
        let tag = quote::format_ident!("{}", format!("{:?}", tag));

        let mut children_tokens = TokenStream::new();
        children.to_tokens(&mut children_tokens);

        let mut attribute_pushes = TokenStream::new();

        if let Some(id) = id {
            attribute_pushes.extend(quote! {
                attributes.push(ibex::compose::Attribute {
                    name: "id".to_string(),
                    value: Some((#id).to_string()),
                });
            });
        }
        if let Some(class) = class {
            attribute_pushes.extend(quote! {
                attributes.push(ibex::compose::Attribute {
                    name: "class".to_string(),
                    value: Some((#class).to_string()),
                });
            });
        }

        for attribute in attributes {
            match attribute {
                Attribute::Pair { name, value } => attribute_pushes.extend(quote! {
                    attributes.push(ibex::compose::Attribute {
                        name: #name.to_string(),
                        value: Some((#value).to_string()),
                    });
                }),
                Attribute::Conditional { name, condition } => attribute_pushes.extend(quote! {
                    if #condition {
                        attributes.push(ibex::compose::Attribute {
                            name: #name.to_string(),
                            value: None,
                        })
                    };
                }),
            };
        }

        tokens.extend(quote! {
            ibex::compose::Element {
                tag: ibex::compose::Tag::#tag,
                attributes: {
                    let mut attributes = Vec::new();
                    #attribute_pushes;
                    attributes
                },
                children: #children_tokens,
            },
        });
    }
}

pub fn parse_view(input: TokenStream) -> View {
    let mut nodes = Vec::new();

    let mut tokens = input.into_iter().peekable();

    while let Some(token) = tokens.next() {
        match token {
            TokenTree::Ident(ident) => {
                let tag = ident.to_string();

                let tag: Option<Tag> = if tag == "HEAD" {
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

                // Id and class
                let mut id = None;
                let mut class = None;
                if let Some(TokenTree::Punct(punct)) = tokens.peek() {
                    if punct.to_string() == "#" {
                        tokens.next();
                        let Some(token) = tokens.next() else {
                            panic!("Expected id name");
                        };
                        match token {
                            TokenTree::Literal(literal) => {
                                id = Some(quote! { #literal });
                            }
                            TokenTree::Group(group) if group.delimiter() == Delimiter::Bracket => {
                                id = Some(group.stream());
                            }
                            _ => panic!("Id name must be literal or group"),
                        }
                    }
                }
                if let Some(TokenTree::Punct(punct)) = tokens.peek() {
                    if punct.to_string() == "." {
                        tokens.next();
                        let Some(token) = tokens.next() else {
                            panic!("Expected class name");
                        };
                        match token {
                            TokenTree::Literal(literal) => {
                                class = Some(quote! { #literal });
                            }
                            TokenTree::Group(group) if group.delimiter() == Delimiter::Bracket => {
                                class = Some(group.stream());
                            }
                            _ => panic!("Class name must be literal or group"),
                        }
                    }
                }

                // Disallow id after class
                if let Some(TokenTree::Punct(punct)) = tokens.peek() {
                    if punct.to_string() == "#" {
                        panic!("Id must not come after class name");
                    }
                }

                // Attributes
                let mut attributes = Vec::new();
                if let Some(TokenTree::Group(group)) = tokens.peek() {
                    if group.delimiter() == Delimiter::Bracket {
                        let mut group = group.stream().into_iter().peekable();
                        tokens.next();

                        loop {
                            let Some(name) = group.next() else {
                                panic!("Missing attribute name");
                            };
                            let name = name.to_string();

                            let mut value = TokenStream::new();
                            let mut is_conditional = false;

                            if matches!(group.peek(), Some(TokenTree::Punct(punct)) if punct.to_string() == "!")
                            {
                                // always `true` no-value attribute
                                // same as `name?=true` (conditional with literal)
                                group.next();
                                attributes.push(Attribute::Conditional {
                                    name,
                                    condition: quote!(true),
                                });
                            } else {
                                if matches!(group.peek(), Some(TokenTree::Punct(punct)) if punct.to_string() == "?")
                                {
                                    is_conditional = true;
                                    group.next();
                                }

                                match group.next() {
                                    Some(TokenTree::Punct(punct)) if punct.to_string() == "=" => {}
                                    Some(TokenTree::Punct(punct)) => {
                                        panic!(
                                            "Unexpected punctuation token `{}`. Expected value",
                                            punct
                                        );
                                    }
                                    _ => {
                                        panic!("Unexpected end of attributes. Expected value");
                                    }
                                }

                                loop {
                                    let next = group.peek();
                                    match next {
                                        Some(TokenTree::Punct(punct))
                                            if punct.to_string() == "," =>
                                        {
                                            group.next();
                                            break;
                                        }
                                        None => break,
                                        Some(token) => {
                                            value.extend(token.to_token_stream());
                                            group.next();
                                        }
                                    }
                                }

                                attributes.push(if is_conditional {
                                    Attribute::Conditional {
                                        name,
                                        condition: value,
                                    }
                                } else {
                                    Attribute::Pair { name, value }
                                });
                            }

                            if let Some(TokenTree::Punct(punct)) = group.peek() {
                                if punct.to_string() == "," {
                                    group.next();
                                }
                            }

                            if group.peek().is_none() {
                                break;
                            }
                        }
                    }
                }

                let Some(next) = tokens.next() else {
                    panic!("Unexpected end. Missing braces or single slash.");
                };

                let children = match next {
                    TokenTree::Group(group) => {
                        if group.delimiter() != Delimiter::Brace {
                            panic!("Group must have braces: {{...}}");
                        }

                        if let Some(tag) = tag {
                            if tag.is_void() {
                                panic!("Void tag <{}> cannot contain children. Use `/` to close.", tag);
                            }
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
                        id,
                        class,
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
                enum PrevToken {
                    Nothing,
                    Ident,
                    FirstColon,
                    SecondColon,
                }

                let mut name = TokenStream::new();
                let mut prev_token = PrevToken::Nothing;
                while let Some(token) = tokens.peek() {
                    match token {
                        TokenTree::Ident(ident) => match prev_token {
                            PrevToken::Nothing => {
                                name.extend(quote! { #ident });
                                prev_token = PrevToken::Ident;
                                tokens.next();
                            }
                            PrevToken::Ident => break,
                            PrevToken::FirstColon => {
                                panic!("please use two colons please");
                            }
                            PrevToken::SecondColon => {
                                name.extend(quote! { :: #ident });
                                prev_token = PrevToken::Ident;
                                tokens.next();
                            }
                        },

                        TokenTree::Punct(punct) if punct.to_string() == ":" => match prev_token {
                            PrevToken::Nothing => {
                                name.extend(quote! { :: });
                                prev_token = PrevToken::SecondColon;
                                tokens.next();
                            }
                            PrevToken::Ident => {
                                prev_token = PrevToken::FirstColon;
                                tokens.next();
                            }
                            PrevToken::FirstColon => {
                                prev_token = PrevToken::SecondColon;
                                tokens.next();
                            }
                            PrevToken::SecondColon => break,
                        },
                        _ => break,
                    }
                }

                let arguments = match tokens.peek() {
                    Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Bracket => {
                        let arguments = group.stream();
                        tokens.next();

                        // Remove trailing comma
                        // Because comma is automatically inserted between 'attribute-like params'
                        // and 'child-like View param'
                        // Stream is reversed, first item removed (if is comma), and reversed back
                        let mut stream_rev: Vec<TokenTree> = arguments.into_iter().collect();
                        stream_rev.reverse();
                        let mut stream_rev = stream_rev.into_iter().peekable();
                        match stream_rev.peek() {
                            Some(TokenTree::Punct(punct)) if punct.to_string() == "," => {
                                stream_rev.next();
                            }
                            _ => (),
                        }
                        let mut arguments: Vec<TokenTree> = stream_rev.collect();
                        arguments.reverse();
                        Some(arguments.into_iter().collect())
                    }
                    _ => None,
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

                let arguments = match arguments {
                    Some(arguments) => arguments,
                    None => {
                        if children.is_some() {
                            TokenStream::new()
                        } else {
                            panic!("function call must include argument group (even empty), if does not children");
                        }
                    }
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
                    if punct.to_string() == ":" {
                        stream.next();

                        let Some(TokenTree::Ident(statement)) = stream.next() else {
                            panic!("Missing or invalid statement");
                        };
                        match statement.to_string().as_str() {
                            "if" => {
                                // reverse tokenstream to consume from end
                                // cannot use `.rev` as TokenStream as an iterator is not
                                // double-ended
                                let mut stream_rev: Vec<TokenTree> = stream.collect();
                                stream_rev.reverse();
                                let mut stream_rev = stream_rev.into_iter().peekable();

                                // match last block
                                // this could be the block after `if` or `else`
                                let last_block = match stream_rev.next() {
                                    Some(TokenTree::Group(group))
                                        if group.delimiter() == Delimiter::Brace =>
                                    {
                                        parse_view(group.stream())
                                    }
                                    _ => panic!("`if` block must be group (matching last token)"),
                                };

                                // match `else` keyword before last block
                                // if found, last block must be `else` block
                                // otherwise, last block must be `if` block
                                let (then, otherwise) = match stream_rev.peek() {
                                    Some(TokenTree::Ident(ident))
                                        if ident.to_string() == "else" =>
                                    {
                                        stream_rev.next();
                                        let then = match stream_rev.next() {
                                            Some(TokenTree::Group(group))
                                                if group.delimiter() == Delimiter::Brace =>
                                            {
                                                parse_view(group.stream())
                                            }
                                            _ => panic!("`if` block must be group (matching third last token)"),
                                        };
                                        (then, Some(last_block))
                                    }
                                    _ => (last_block, None),
                                };

                                // reverse back and return to tokenstream
                                // everything before blocks matched above, is part of `if`
                                // condition
                                let mut condition: Vec<TokenTree> = stream_rev.collect();
                                condition.reverse();
                                let condition: TokenStream = condition.into_iter().collect();

                                nodes.push(Node::If(condition, then, otherwise));
                            }

                            "for" => {
                                // reverse tokenstream to consume from end
                                // cannot use `.rev` as TokenStream as an iterator is not
                                // double-ended
                                let mut stream_rev: Vec<TokenTree> = stream.collect();
                                stream_rev.reverse();
                                let mut stream_rev = stream_rev.into_iter().peekable();

                                // match last block
                                // this must be the block of the `for` loop (obviously)
                                let block = match stream_rev.next() {
                                    Some(TokenTree::Group(group))
                                        if group.delimiter() == Delimiter::Brace =>
                                    {
                                        parse_view(group.stream())
                                    }
                                    _ => panic!("`if` block must be group (matching last token)"),
                                };

                                // reverse back and return to tokenstream
                                // this must be the 'source' of the `for` loop (between `for` and
                                // block)
                                let mut source: Vec<TokenTree> = stream_rev.collect();
                                source.reverse();
                                let source: TokenStream = source.into_iter().collect();

                                nodes.push(Node::For(source, block));
                            }

                            "where" => {
                                // reverse tokenstream to consume from end
                                // cannot use `.rev` as TokenStream as an iterator is not
                                // double-ended
                                let mut stream_rev: Vec<TokenTree> = stream.collect();
                                stream_rev.reverse();
                                let mut stream_rev = stream_rev.into_iter().peekable();

                                // match last block
                                // this must be the block of the `where` statement (obviously)
                                let block = match stream_rev.next() {
                                    Some(TokenTree::Group(group))
                                        if group.delimiter() == Delimiter::Brace =>
                                    {
                                        parse_view(group.stream())
                                    }
                                    _ => {
                                        panic!("`where` block must be group (matching last token)")
                                    }
                                };

                                // reverse back and return to tokenstream
                                // this is the `let` statements for the `where` statement
                                let mut statements: Vec<TokenTree> = stream_rev.collect();
                                statements.reverse();
                                let statements: TokenStream = statements.into_iter().collect();

                                nodes.push(Node::With(statements, block));
                            }

                            _ => panic!("Invalid statement"),
                        }
                        continue;
                    }
                }

                nodes.push(Node::Expression(stream.collect()));
            }

            // Tilde for whitespace
            TokenTree::Punct(punct) if punct.to_string() == "~" => {
                // Double tilde for linebreak
                if let Some(TokenTree::Punct(punct)) = tokens.peek() {
                    if punct.to_string() == "~" {
                        tokens.next();
                        nodes.push(Node::Literal("\n".to_string()));
                        continue;
                    }
                }
                nodes.push(Node::Literal(" ".to_string()));
            }

            // Html entity
            TokenTree::Punct(punct) if punct.to_string() == "&" => {
                let code = match tokens.next() {
                    Some(TokenTree::Ident(ident)) => ident.to_string(),
                    Some(TokenTree::Literal(literal))
                        if literal.to_string().parse::<i32>().is_ok() =>
                    {
                        literal.to_string()
                    }
                    _ => panic!("Expected html entity code, as ident or codepoint integer literal"),
                };
                // optional semicolon after code
                match tokens.peek() {
                    Some(TokenTree::Punct(punct)) if punct.to_string() == ";" => {
                        tokens.next();
                    }
                    _ => (),
                }
                nodes.push(Node::Literal(format!("&{};", code)));
            }

            _ => {
                panic!("Unexpected token {:#?}", token);
            }
        }
    }

    View(nodes)
}
