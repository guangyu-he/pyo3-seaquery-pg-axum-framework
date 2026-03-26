pub mod health;
pub mod py_example;

use utoipa::OpenApi;

use crate::endpoints::health::__path_health;
use crate::endpoints::py_example::__path_handle_py_example_cls;
use crate::endpoints::py_example::__path_handle_py_example_func;

#[derive(OpenApi)]
#[openapi(
    paths(handle_py_example_cls, handle_py_example_func, health),
    components(schemas())
)]
pub struct ApiDoc;
