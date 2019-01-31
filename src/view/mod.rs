//! Rendering to templates.
//!
//! > View is the only module that knows anything about HTML, or JSON, or other "renderings" of the
//! > response. I'm happy to call this "view" in common with traditional stateless MVC, because
//! > it's role is largely the same.

use failure::{Error, Fallible, SyncFailure};
use packer::Packer;
use serde::Serialize;
use tera::Tera;
use warp::{
    http::{header::CONTENT_TYPE, Response},
    reject::custom,
    Rejection,
};

lazy_static::lazy_static! {
    static ref TERA: Tera = {
        #[derive(Packer)]
        #[folder = "src/view/templates"]
        struct Templates;

        let mut tera = Tera::default();
        let templates = Templates::list()
            .map(|name| (name, Templates::get_str(name).unwrap()))
            .collect::<Vec<_>>();
        tera.add_raw_templates(templates).unwrap();
        tera.build_inheritance_chains().unwrap();
        tera
    };
}

/// Renders a template as HTML to a String.
pub fn render<T: Serialize>(name: &str, data: T) -> Fallible<String> {
    TERA.render(name, &data)
        .map_err(|err| SyncFailure::new(err).into())
}

/// Renders a template as HTML to a `warp::Reply`.
pub fn render_html<T: Serialize>(name: &str, data: T) -> Result<Response<String>, Rejection> {
    render(name, data)
        .and_then(|body| {
            Response::builder()
                .header(CONTENT_TYPE, "text/html; charset=utf-8")
                .body(body)
                .map_err(Error::from)
        })
        .map_err(|err| custom(err.compat()))
}
