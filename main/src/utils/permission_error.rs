use malvolio::prelude::*;

use super::default_head;

pub fn permission_error() -> Html {
    Html::new()
        .head(default_head("Permission error".to_string()))
        .body(
            Body::new()
                .child(H1::new("Permission error"))
                .child(P::with_text(
                    "You don't have permission to view this resource. You might need to ask
                    your teacher for an invite to this class, or contact your administrator.",
                )),
        )
}
