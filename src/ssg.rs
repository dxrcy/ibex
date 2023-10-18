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

const BUILD_DIR: &str = "build";
const STATIC_DIR: &str = "static";
const SCSS_DIR: &str = "src/scss";

pub fn quick_build(routes: Vec<Route>) -> io::Result<()> {
    let files = render_routes(routes);
    write_files(files)?;
    if Path::new(STATIC_DIR).exists() {
        copy_static()?;
    }
    if Path::new(SCSS_DIR).exists() {
        convert_scss()?;
    }
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
    // For development mode
    // symlink::symlink_dir(
    //     Path::new(STATIC_DIR),
    //     Path::new(&format!("{BUILD_DIR}/static")),
    // )
    copy_folder(
        Path::new(STATIC_DIR),
        Path::new(&format!("{BUILD_DIR}/static")),
    )
}

pub fn convert_scss() -> io::Result<()> {
    convert_scss_folder(Path::new(SCSS_DIR), Path::new(&format!("{BUILD_DIR}/css")))
}

fn convert_scss_folder(src: &Path, dest: &Path) -> io::Result<()> {
    if src.is_dir() {
        fs::create_dir_all(dest)?;

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let entry_path = entry.path();
            let filename_dest = replace_scss_extension(&entry.file_name().to_string_lossy());
            let dest_path = dest.join(filename_dest);

            if entry_path.is_dir() {
                copy_folder(&entry_path, &dest_path)?;
            } else {
                let scss = fs::read_to_string(entry_path)?;
                let css = grass::from_string(scss, &Default::default())
                    .expect("Failed to compile scss to css");
                fs::write(dest_path, css)?;
            }
        }
    }
    Ok(())
}

fn replace_scss_extension(filename: &str) -> String {
    let mut split: Vec<_> = filename.split(".").collect();
    if split.last() == Some(&"scss") {
        split.pop();
        split.join(".") + ".css"
    } else {
        filename.to_string()
    }
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
