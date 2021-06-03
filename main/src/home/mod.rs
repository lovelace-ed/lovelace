use malvolio::prelude::*;
use mercutio::{compose, Apply};
use portia::{colour::GreyBackground, levels::Level, padding::DefaultPadding, render::RenderCtx};
use rocket::response::Redirect;

use crate::{
    auth::OptionAuthCookie,
    ui::page::Page,
    utils::{default_head, html_or_redirect::HtmlOrRedirect},
};

#[get("/")]
pub fn home(auth_cookie: OptionAuthCookie) -> HtmlOrRedirect {
    if auth_cookie.0.is_some() {
        HtmlOrRedirect::Redirect(Redirect::to("/dashboard"))
    } else {
        HtmlOrRedirect::Html(Html::default()
            .head(
                default_head("Home").child(
                    Meta::default()
                        .attribute(MetaName::Charset)
                        .attribute(Content::new("utf-8")),
                ),
            )
            .body(
                Page::new()
                    .child(Level::new()
                        .child(P::with_text(
                            "IMPORTANT: This site is in very early stages. Please don't use this
                            for anything serious at this point in time.",
                        ))
                        .child(P::with_text(
                            "Lovelace is a digital platform for learning. It's also quite an
                            incomplete one at the moment, but we're adding features relatively
                            quickly. Updates to this site are rolled out as soon as they're ready,
                            so check back soon for more.",
                        ))
                        .into_div()
                        .apply(compose(GreyBackground, DefaultPadding))
                    )
                    .child(
                        Level::new().child(
                            A::default()
                                .href("https://github.com/lovelace-ed/lovelace")
                                .text("Click me to view the source code.")
                        )
                        .into_div()
                        .apply(DefaultPadding)
                    )
                    .render(auth_cookie)
            )
        )
    }
}
