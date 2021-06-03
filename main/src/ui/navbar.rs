use malvolio::prelude::*;
use mercutio::{compose, Apply};
use portia::{
    colour::YellowBackground,
    levels::{LayoutAxis, LayoutStrategy, Level},
    padding::DefaultPadding,
    render::RenderCtx,
};

use crate::auth::OptionAuthCookie;

pub struct Navbar;

impl RenderCtx<Div> for Navbar {
    type Ctx = OptionAuthCookie;

    fn render(self, ctx: Self::Ctx) -> Div {
        Level::new()
            .strategy(LayoutStrategy::new().axis(LayoutAxis::Horizontal))
            .apply(|navbar| {
                if ctx.0.is_some() {
                    todo!()
                } else {
                    navbar
                        .child(a().href("/").text("Lovelace").apply(DefaultPadding))
                        .child(a().href("/auth/login").text("Login").apply(DefaultPadding))
                        .child(
                            a().href("/auth/register")
                                .text("Register")
                                .apply(DefaultPadding),
                        )
                }
            })
            .into_div()
            .apply(compose(YellowBackground, DefaultPadding))
    }
}
