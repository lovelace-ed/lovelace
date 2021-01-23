# Malvolio

Malvolio is a HTML templating library, which you can use both on your server and in a browser. You
can build complex layouts quickly and efficiently. We've found that Malvolio make it easy to write
composable code, where you can use Rust's built in features for abstraction (functions, traits, etc)
to build more easily maintainable applications. Combined with Rust's strong type system you should
be able to pretty fearlessly refactor your applications into something you'd actually like to work
on again, rather than just handing off to the next unlucky person.

## Usage

Malvolio should be relatively simple to use, especially if you have used other libraries which offer
"builder-syntax" style APIs.

A quick couple of examples (see the examples and documentation for more details).

```rust
malvolio::prelude::Form::new()
    .attribute(Method::Post)
    .child(
        Input::default()
            .attribute(Type::Text)
            .attribute(Name::new("invited-user-identifier")),
    )
    .child(
        Input::default()
            .attribute(Type::Submit)
            .attribute(Value::new("Invite teacher!")),
    )
```

```rust
/* Note that this example DOES NOT COMPILE as is because it is part of a function which it was taken
from and without which all the variables are not correctly defined. */
Html::default()
    .head(default_head("Notifications".to_string()))
    .body({
        let mut body = Body::default();
        if let Some(element) = custom_element {
            body = body.child(element);
        }
        body.child(
            Div::new()
                .attribute(malvolio::prelude::Class::from(LIST))
                .children(data.into_iter().map(|notification| {
                    Div::new()
                        .attribute(malvolio::prelude::Class::from(LIST_ITEM))
                        .child(H3::new(notification.title))
                        .child(P::with_text(notification.contents))
                        .child(
                            A::default()
                                .attribute(Href::new(format!(
                                    "/notifications/mark_read/{}",
                                    notification.id
                                )))
                                .text("Mark as read"),
                        )
                        .child(
                            A::default()
                                .attribute(Href::new(format!(
                                    "/notifications/delete/{}",
                                    notification.id
                                )))
                                .text("Delete this notification"),
                        )
                })),
        )
```

## Examples
We haven't provided any explicit examples per-se, but we are in the process of writing a web
application with Malvolio which might be helpful when trying to work out how to use the library.
The [source code is on Github](https://github.com/lovelace-ed/lovelace/tree/main/main).

## Documentation

Malvolio has API docs which are [hosted on docs.rs](https://docs.rs).

If it's confusing you though please do consider raising an
[issue on our Github repository](https://github.com/lovelace-ed/lovelace/issues) (yes, that is the
correct repository :-) )
