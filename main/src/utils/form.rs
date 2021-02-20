use malvolio::prelude::*;
use portia::render::Render;

use super::{default_head, error::LovelaceError};

pub trait FormProducer {
    fn produce(self) -> Form;
}

pub struct FormErrorMsg<P>(pub LovelaceError, pub P);

impl<P> Render<Div> for FormErrorMsg<P>
where
    P: FormProducer,
{
    fn render(self) -> Div {
        let (error, form) = (self.0, self.1);
        Div::new()
            .child(Render::<Div>::render(error))
            .child(form.produce())
    }
}

impl<P> Render<Html> for FormErrorMsg<P>
where
    P: FormProducer,
{
    fn render(self) -> Html {
        Html::new()
            .status(400)
            .head(default_head("Bad request"))
            .body(
                Body::new()
                    .child(H3::new(match self.0 {
                        LovelaceError::PermissionError => {
                            "Error – you don't have permission to do this."
                        }
                        LovelaceError::DatabaseError => {
                            "Encountered a database error when trying to fulfil this request."
                        }
                    }))
                    .child(Render::<Div>::render(self)),
            )
    }
}
