//! Utility for creating schema definitions.

pub struct SchemaBuilder {
    pub schema: String,
}

impl SchemaBuilder {
    pub fn new() -> Self {
        Self { schema: String::new() }
    }
    
    pub fn with_field(mut self, field: &str) -> Self {
        self.schema.push_str(field);
        self
    }
    
    pub fn build(self) -> String {
        self.schema
    }
}
