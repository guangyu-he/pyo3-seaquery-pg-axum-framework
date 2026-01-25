pub mod health;
pub mod py_example;

use utoipa::OpenApi;

use crate::endpoints::health::__path_health;
use crate::endpoints::py_example::__path_py_example;

#[derive(OpenApi)]
#[openapi(paths(py_example, health), components(schemas()))]
pub struct ApiDoc;
