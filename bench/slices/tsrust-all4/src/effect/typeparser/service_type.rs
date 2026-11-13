// Go: internal/typeparser/service_type.go

use crate::effect::typeparser::*;
use crate::prelude::*;

/// ServiceTypeId is the property key for the newer Context.Service variance struct.
pub const SERVICE_TYPE_ID: &str = "~effect/Context/Service";

/// Service represents parsed v4 service type parameters.
#[derive(Clone, Debug)]
pub struct Service {
    /// The service identifier/tag type
    pub identifier: TypeId,
    /// The service implementation shape
    pub shape: TypeId,
}

