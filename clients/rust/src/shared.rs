pub struct KVMetadata {
    pub key: String,
    pub value: String,
}

pub struct MetadataFindInput {
    pub limit: usize,
    pub skip: usize,
    pub metadata: KVMetadata,
}

impl MetadataFindInput {
    pub(crate) fn to_query(&self) -> Vec<(String, String)> {
        vec![
            ("skip".to_string(), self.skip.to_string()),
            ("limit".to_string(), self.limit.to_string()),
            ("key".to_string(), self.metadata.key.clone()),
            ("value".to_string(), self.metadata.value.clone()),
        ]
    }
}
