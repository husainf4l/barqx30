use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "objects")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub bucket_id: i32,
    #[sea_orm(unique)]
    pub key: String,
    pub size: i64,
    pub etag: String,
    pub content_type: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::buckets::Entity",
        from = "Column::BucketId",
        to = "super::buckets::Column::Id"
    )]
    Buckets,
}

impl Related<super::buckets::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Buckets.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
