use crate::ShaderStages;

#[derive(Debug, Clone)]
pub struct Shader {
    pub source: String,
    pub stage: ShaderStages,
}

impl Shader {
    pub fn new(source: String, stage: ShaderStages) -> Self {
        Self { source, stage }
    }
}
