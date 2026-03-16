use crate::runtime_paths::RuntimePaths;

#[derive(Debug, Clone)]
pub struct RuntimeService {
    pub paths: RuntimePaths,
}

impl RuntimeService {
    pub fn new(paths: RuntimePaths) -> Self {
        Self { paths }
    }
}
