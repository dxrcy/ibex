use std::{fs, io, path::Path};

use crate::dom::Document;

#[derive(Clone, Debug)]
pub struct Route {
    url_path: String,
    document: Document,
}

impl Route {
    pub fn new(url_path: String, document: Document) -> Self {
        Self { url_path, document }
    }
}

pub struct File {
    path: String,
    content: String,
}

pub fn render_routes(routes: Vec<Route>) -> Vec<File> {
    routes.into_iter().map(render_route).collect()
}

pub fn render_route(route: Route) -> File {
    File {
        path: url_path_to_filepath(&route.url_path),
        content: route.document.render(),
    }
}

pub fn write_files(files: Vec<File>) -> Result<(), io::Error> {
    let folder = "build";

    if Path::new(folder).exists() {
        fs::remove_dir_all(folder)?;
    }
    fs::create_dir_all(folder)?;

    for file in files {
        let path = format!("{folder}/{}", file.path);
        create_parent_folder(&path)?;
        fs::write(path, file.content)?;
    }

    Ok(())
}

fn url_path_to_filepath(path: &str) -> String {
    remove_leading_slash(&format!("{}/index.html", path)).to_string()
}

fn remove_leading_slash(path: &str) -> &str {
    let mut chars = path.chars();
    while chars.as_str().starts_with('/') {
        chars.next();
    }
    chars.as_str()
}

fn create_parent_folder(path: &str) -> Result<(), io::Error> {
    let mut path = path.split('/');
    path.next_back();
    let path: Vec<_> = path.collect();
    if path.len() < 1 {
        return Ok(());
    }
    fs::create_dir_all(path.join("/"))
}

#[macro_export]
macro_rules! routes {
    ( $(
        ( $($tt:tt)* )
        $( for $args:pat in $src:expr )?
        => $expr:expr
    ),* $(,)? ) => {{
        vec![ $(
            ::ibex::routes!(@one
                ( $($tt)* )
                $( for $args in $src)?
                => $expr
            ),
        )* ].concat()
    }};

    (@one
        ( $($tt:tt)* )
        => $expr:expr
    ) => {
        vec![::ibex::route::Route::new(
            ::ibex::routes!(@path $($tt)*),
            $expr,
        )]
    };

    (@one
        ( $($tt:tt)* )
        for $args:pat in $src:expr
        => $expr:expr
    ) => {
        $src
            .map(|$args|
                ::ibex::route::Route::new(
                    ::ibex::routes!(@path $($tt)*),
                    $expr,
                )
            )
            .collect::<Vec<::ibex::route::Route>>()
    };

    (@path) => { "" };
    (@path
        / $($tt:tt)*
    ) => {
        "/".to_string()
            + &routes!(@path $($tt)*)
    };
    (@path
        $x:ident $($tt:tt)*
    ) => {
        stringify!($x).to_string()
            + &routes!(@path $($tt)*)
    };
    (@path
        [$x:expr]
        $($tt:tt)*
    ) => {
        $x.to_string()
            + &routes!(@path $($tt)*)
    };
}
