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

#[derive(Debug)]
pub struct RouteFile {
    path: String,
    content: String,
}

pub fn build(routes: Vec<Route>) -> io::Result<()> {
    let files = render_routes(routes);
    write_files(files)?;
    copy_static()?;
    Ok(())
}

pub fn render_routes(routes: Vec<Route>) -> Vec<RouteFile> {
    routes.into_iter().map(render_route).collect()
}

pub fn render_route(route: Route) -> RouteFile {
    RouteFile {
        path: url_path_to_filepath(&route.url_path),
        content: route.document.render(),
    }
}

const BUILD_DIR: &str = "build";
const STATIC_DIR: &str = "static";

pub fn write_files(files: Vec<RouteFile>) -> Result<(), io::Error> {
    if Path::new(BUILD_DIR).exists() {
        fs::remove_dir_all(BUILD_DIR)?;
    }
    fs::create_dir_all(BUILD_DIR)?;

    for file in files {
        let path = format!("{BUILD_DIR}/{}", file.path);
        create_parent_folder(&path)?;
        fs::write(path, file.content)?;
    }

    Ok(())
}

pub fn copy_static() -> io::Result<()> {
    copy_folder(
        Path::new(STATIC_DIR),
        Path::new(&format!("{BUILD_DIR}/static")),
    )
}

fn copy_folder(src: &Path, dest: &Path) -> io::Result<()> {
    if src.is_dir() {
        fs::create_dir_all(dest)?;

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let entry_path = entry.path();
            let dest_path = dest.join(entry.file_name());

            if entry_path.is_dir() {
                copy_folder(&entry_path, &dest_path)?;
            } else {
                fs::copy(&entry_path, &dest_path)?;
            }
        }
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
        vec![::ibex::ssg::Route::new(
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
                ::ibex::ssg::Route::new(
                    ::ibex::routes!(@path $($tt)*),
                    $expr,
                )
            )
            .collect::<Vec<::ibex::ssg::Route>>()
    };

    (@path) => { "" };
    (@path
        / $($tt:tt)*
    ) => {
        "/".to_string()
            + &routes!(@path $($tt)*)
    };
    (@path
        $x:literal $($tt:tt)*
    ) => {
        $x.to_string()
            + &routes!(@path $($tt)*)
    };
    (@path
        [$x:expr] $($tt:tt)*
    ) => {
        $x.to_string()
            + &routes!(@path $($tt)*)
    };
}
