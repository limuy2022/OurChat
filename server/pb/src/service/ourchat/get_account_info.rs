pub mod v1 {
    use std::{collections::HashSet, sync::LazyLock};

    include!("../../generated/service.ourchat.get_account_info.v1.rs");

    pub static OWNER_PRIVILEGE: LazyLock<HashSet<QueryValues>> = LazyLock::new(|| {
        collection_literals::collection! {
            QueryValues::Sessions,
            QueryValues::Friends,
            QueryValues::UpdatedTime,
            QueryValues::Email,
        }
    });
}
