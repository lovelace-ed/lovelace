/// Render an object.
pub trait Render<TO> {
    fn render(self) -> TO;
}
