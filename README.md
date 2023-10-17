# Ibex

Ibex provides ergonomic HTML templating in Rust for SSG/SSR, using procedural macros.

Still a work in progress.

## Example

```rs
fn at_blog(blog: BlogPost) -> Document {
    // Creates a `View` and converts to `Document`
    view! {
        // Calls a function `header(true)`
        @header[true]

        // Some html elements
        h2 { [blog.title] }
        h3 { i {[blog.author]} }

        // Include any variables in scope
        p {
            [blog.body]
        }

        // Variables can be used in attributes
        // Use a slash to signify an empty element body
        img [src=blog.image]/
    }
    .into()
}
```

![Ibex logo](./ibex.png)

