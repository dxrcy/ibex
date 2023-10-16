# Ibex

Ibex provides ergonomic HTML templating in Rust for SSG/SSR, using procedural macros.

Still a work in progress.

## Example

```rs
fn blog_page(blog: BlogPost) -> Document {
    // Creates a `View` and converts to `Document`
    view! {
        // Calls a function `header(true)`
        @header[true]

        // Html elements
        h2 { [blog.title] }
        h3 { i {[blog.author]} }

        // Include any variables in scope
        p {
            [blog.body]
        }

        // Use debug formatting to put string in quotes
        img [src=blog.image]/
    }
    .into()
}
```

![svg design of an ibex](./ibex.png)

