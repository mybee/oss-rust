#[derive(Clone, Debug)]
pub struct ListBuckets {
    prefix: String,
    marker: String,
    max_keys: String,
    is_truncated: bool,
    next_marker: String,

    id: String,
    display_name: String,

    buckets: Vec<Bucket>,
}

impl ListBuckets {
    pub fn new(
        prefix: String,
        marker: String,
        max_keys: String,
        is_truncated: bool,
        next_marker: String,
        id: String,
        display_name: String,
        buckets: Vec<Bucket>,
    ) -> Self {
        ListBuckets {
            prefix,
            marker,
            max_keys,
            is_truncated,
            next_marker,
            id,
            display_name,
            buckets,
        }
    }

    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    pub fn marker(&self) -> &str {
        &self.marker
    }

    pub fn max_keys(&self) -> &str {
        &self.max_keys
    }

    pub fn is_truncated(&self) -> bool {
        self.is_truncated
    }

    pub fn next_marker(&self) -> &str {
        &self.next_marker
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn buckets(&self) -> &Vec<Bucket> {
        &self.buckets
    }
}

#[derive(Clone, Debug)]
pub struct Bucket {
    name: String,
    create_date: String,
    location: String,
    extranet_endpoint: String,
    intranet_endpoint: String,
    storage_class: String,
}

impl Bucket {
    pub fn new(
        name: String,
        create_date: String,
        location: String,
        extranet_endpoint: String,
        intranet_endpoint: String,
        storage_class: String,
    ) -> Self {
        Bucket {
            name,
            create_date,
            location,
            extranet_endpoint,
            intranet_endpoint,
            storage_class,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn create_data(&self) -> &str {
        &self.create_date
    }

    pub fn location(&self) -> &str {
        &self.location
    }

    pub fn extranet_endpoint(&self) -> &str {
        &self.extranet_endpoint
    }

    pub fn intranet_endpoint(&self) -> &str {
        &self.intranet_endpoint
    }

    pub fn storage_class(&self) -> &str {
        &self.storage_class
    }
}
