//! Utility for creating schema definitions.

pub struct SchemaBuilder {
    pub schema: String,
}

impl SchemaBuilder {
    pub fn new() -> Self {
        Self { schema: String::new() }
    }
    
    pub fn build(self) -> String {
        self.schema
    }
}
