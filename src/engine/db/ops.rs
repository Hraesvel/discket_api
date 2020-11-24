use arangors::document::options::{InsertOptions, UpdateOptions};
use async_trait::async_trait;

use crate::engine::db::Db;
use crate::engine::EngineError;
use crate::io::{read::EngineGet, write::EngineWrite};
use crate::models::{DocDetail, RequiredTraits};

#[async_trait]
impl EngineGet for Db {
    type E = EngineError;

    async fn get_all<T>(&self) -> Result<Vec<T>, Self::E>
        where
            T: RequiredTraits,
    {
        use arangors::{AqlQuery, Cursor};
        use crate::engine::db::arangodb::aql_snippet;

        let query = AqlQuery::builder()
            .query(aql_snippet::GET_ALL)
            .bind_var("@collection", T::collection_name())
            .batch_size(25)
            .build();

        let cursor: Cursor<T> = self.db().aql_query_batch(query).await?;
        let mut collection: Vec<T> = cursor.result;

        /// Collecting via pagination.
        if let Some(mut i) = cursor.id {
            while let Ok(c) = self.db().aql_next_batch(&i).await {
                let mut r: Vec<T> = c.result;
                collection.append(&mut r);
                if let Some(next_id) = c.id {
                    i = next_id;
                } else {
                    break;
                }
            }
        };

        Ok(collection)
    }

    async fn get<T>(&self, id: &str) -> Result<T, Self::E> {
        unimplemented!()
    }
}

#[async_trait]
impl EngineWrite for Db {
    type E = EngineError;

    async fn insert<T: RequiredTraits>(&self, doc: T) -> Result<(), Self::E>
    {
        let io = InsertOptions::builder().overwrite(false).build();
        let col = self.db().collection(T::collection_name()).await?;
        let _doc = col.create_document(doc, io).await?;
        Ok(())
    }

    async fn update<T: RequiredTraits>(&self, doc: T) -> Result<(), Self::E> {
        let col = self.db().collection(T::collection_name()).await?;
        let _doc = col.update_document(&doc.key(), doc, UpdateOptions::default()).await?;
        Ok(())
    }
}