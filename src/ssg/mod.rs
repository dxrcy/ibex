use std::{
    fs, io,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{compose::View, dom::Document};

pub fn use_autoreload() -> View {
    const AUTORELOAD_SCRIPT: &str = include_str!("autoreload.html");
    if crate::is_local() {
        AUTORELOAD_SCRIPT.into()
    } else {
        View::new()
    }
}

#[derive(Clone, Debug)]
pub struct Route {
    url_paths: Vec<String>,
    content: RouteContent,
}

impl Route {
    pub fn new(url_paths: Vec<String>, content: impl Into<RouteContent>) -> Self {
        Self {
            url_paths,
            content: content.into(),
        }
    }
    pub fn new_document(url_paths: Vec<String>, document: Document) -> Self {
        Self {
            url_paths,
            content: RouteContent::Document(document),
        }
    }
    pub fn new_raw(url_paths: Vec<String>, content: String) -> Self {
        Self {
            url_paths,
            content: RouteContent::Raw(content),
        }
    }
    pub fn render(self) -> RouteFile {
        let paths = self
            .url_paths
            .into_iter()
            .map(|url_path| {
                if self.content.is_document() {
                    url_path_to_filepath(&url_path)
                } else {
                    url_path
                }
            })
            .collect();

        RouteFile {
            paths,
            content: self.content.render(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum RouteContent {
    Document(Document),
    Raw(String),
}

impl RouteContent {
    pub fn render(self) -> String {
        match self {
            Self::Document(document) => document.render(),
            Self::Raw(content) => content,
        }
    }
    fn is_document(&self) -> bool {
        matches!(self, Self::Document(_))
    }
}

impl From<Document> for RouteContent {
    fn from(value: Document) -> Self {
        Self::Document(value)
    }
}
impl From<Raw> for RouteContent {
    fn from(value: Raw) -> Self {
        Self::Raw(value.0)
    }
}

pub struct Raw(String);
impl From<String> for Raw {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl From<&String> for Raw {
    fn from(value: &String) -> Self {
        Self(value.to_string())
    }
}
impl From<&str> for Raw {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

pub fn raw(content: impl Into<Raw>) -> Raw {
    content.into()
}

#[derive(Debug)]
pub struct RouteFile {
    paths: Vec<String>,
    content: String,
}

// (re)create empty
const BUILD_DIR: &str = "build";
// copy/symlink
const STATIC_DIR: &str = "static";
// move/symlink after copy/symlink of `static`
const STATIC_ROOT_DIR: &str = "build/static/_root";
// recreate with converting to css
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
    write_build_timestamp()?;
    Ok(())
}

pub fn write_build_timestamp() -> Result<(), io::Error> {
    let path = format!("{BUILD_DIR}/__timestamp");
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();
    fs::write(path, current_time)
}

pub fn render_routes(routes: Vec<Route>) -> Vec<RouteFile> {
    routes.into_iter().map(Route::render).collect()
}
pub fn render_route(route: Route) -> RouteFile {
    route.render()
}

pub fn write_files(files: Vec<RouteFile>) -> Result<(), io::Error> {
    if Path::new(BUILD_DIR).exists() {
        fs::remove_dir_all(BUILD_DIR)?;
    }
    fs::create_dir_all(BUILD_DIR)?;

    for file in files {
        for path in file.paths {
            let path = format!("{BUILD_DIR}/{}", path);
            create_parent_folder(&path)?;
            fs::write(path, &file.content)?;
        }
    }

    Ok(())
}

pub fn copy_static() -> io::Result<()> {
    // Symlink instead of copy for development mode
    // Does not work for non-unix systems (nbd)
    copy_or_symlink_dir(
        Path::new(STATIC_DIR),
        Path::new(&format!("{BUILD_DIR}/static")),
        crate::is_local(),
    )?;
    if Path::new(STATIC_ROOT_DIR).exists() {
        move_static_root()?;
    }
    Ok(())
}

/// Move each file and directory from 'static root' directory to build directory
///
/// `build/static/_root/XYZ` -> `build/XYZ`
fn move_static_root() -> io::Result<()> {
    let static_root = Path::new(STATIC_ROOT_DIR);
    let build = Path::new(BUILD_DIR);

    let children = fs::read_dir(static_root)?.flatten();
    for child in children {
        let dest_path = Path::new(build).join(child.file_name());
        move_or_symlink_item(&child.path(), &dest_path, crate::is_local())?;
    }

    if !crate::is_local() {
        fs::remove_dir(static_root)?;
    }

    Ok(())
}

/// Copy or symlink a file or directory, depending on argument
#[cfg(target_family = "unix")]
fn move_or_symlink_item(src: &Path, dest: &Path, do_symlink: bool) -> io::Result<()> {
    if do_symlink {
        // Source path must be absolute (or relative to BUILD_DIR, but thats worse)
        let src = fs::canonicalize(src)?;
        std::os::unix::fs::symlink(src, dest)
    } else {
        fs::rename(src, dest)
    }
}
#[cfg(not(target_family = "unix"))]
fn move_or_symlink_dir(src: &Path, dest: &Path, _do_symlink: bool) -> io::Result<()> {
    fs::rename(src, dest)
}

/// Copy or symlink a directory, depending on argument
#[cfg(target_family = "unix")]
fn copy_or_symlink_dir(src: &Path, dest: &Path, do_symlink: bool) -> io::Result<()> {
    if do_symlink {
        // Source path must be absolute (or relative to BUILD_DIR, but thats worse)
        let src = fs::canonicalize(src)?;
        std::os::unix::fs::symlink(src, dest)
    } else {
        copy_folder(src, dest)
    }
}
#[cfg(not(target_family = "unix"))]
fn copy_or_symlink_dir(src: &Path, dest: &Path, _do_symlink: bool) -> io::Result<()> {
    copy_folder(src, dest)
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
    // 404 needs a special case, for github pages to work
    let filepath = if path == "404" {
        "404.html".to_string()
    } else {
        format!("{}/index.html", path)
    };
    remove_leading_slash(&filepath).to_string()
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

/// Define a vector of `Route`s
#[macro_export]
macro_rules! routes {
    ( $(
        $( ($($tt:tt)*) )|*
        $( for $args:pat in $src:expr )?
        => $expr:expr
    ),* $(,)? ) => {{
        vec![ $(
            ::ibex::routes!(@one
                $( ( $($tt)* ) )|*
                $( for $args in $src)?
                => $expr
            ),
        )* ].concat()
    }};

    // A single route

    (@one
        $( ($($tt:tt)*) )|*
        => $expr:expr
    ) => {
        vec![
            ::ibex::ssg::Route::new(
                vec![ $(
                    ::ibex::routes!(@path_full $($tt)*),
                )* ],
                $expr,
            )
        ]
    };

    (@one
        $( ($($tt:tt)*) )|*
        for $args:pat in $src:expr
        => $expr:expr
    ) => {
        $src
            .into_iter()
            .map(|$args| {
                $( ::ibex::routes!(@check_missing_expr $($tt)*); )*;
                ::ibex::routes!(@one $(($($tt)*))|* => $expr)
            })
            .flatten()
            .collect::<Vec<::ibex::ssg::Route>>()
    };

    // Check for missing expression (required for loop routes)

    (@check_missing_expr
        [$x:expr] $($tt:tt)*
    ) => {{ }};
    (@check_missing_expr
        $_:tt $($tt:tt)*
    ) => {
        ::ibex::routes!(@check_missing_expr $($tt)*)
    };
    (@check_missing_expr) => {
        compile_error!("no variables included in loop route.\nthis will use the same url path for every iteration");
    };

    // Resolve a full route path

    (@path_full
        / $($tt:tt)*
    ) => {
        ::ibex::routes!(@path $($tt)*).to_string()
    };
    (@path_full
        $($tt:tt)*
    ) => {
        compile_error!("please start route with slash");
    };

    // Take a part from a route path and keep going

    (@path) => { "" };
    (@path /) => {
        compile_error!("no trailing slash please");
    };
    (@path
        / / $($tt:tt)*
    ) => {
        compile_error!("dont do two slashes in a row");
    };
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
