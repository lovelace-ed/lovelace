/// Render an object.
pub trait Render<TO> {
    /// Renders this item as type `TO`.
    fn render(self) -> TO;
}

impl<ANYTHING, TO> RenderCtx<TO> for ANYTHING
where
    ANYTHING: Render<TO>,
{
    type Ctx = ();

    fn render(self, _: ()) -> TO {
        self.render()
    }
}

/// Render an object, using the context provided. Use this method when writing UI pieces which
/// display application data.
pub trait RenderCtx<TO> {
    type Ctx;

    /// Renders this item as type `TO`.

    fn render(self, ctx: Self::Ctx) -> TO;
}
