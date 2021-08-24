use crate::concurrency::{ThreadPool, ThreadPoolContext};
use crate::engine::{RasterResultDescriptor, ResultDescriptor, VectorResultDescriptor};
use crate::error::Error;
use crate::mock::MockDatasetDataSourceLoadingInfo;
use crate::source::{GdalLoadingInfo, OgrSourceDataset};
use crate::util::Result;
use async_trait::async_trait;
use geoengine_datatypes::dataset::DatasetId;
use geoengine_datatypes::raster::GridShape;
use geoengine_datatypes::raster::TilingSpecification;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;

use super::{RasterQueryRectangle, VectorQueryRectangle};

/// A context that provides certain utility access during operator initialization
pub trait ExecutionContext: Send
    + Sync
    + MetaDataProvider<MockDatasetDataSourceLoadingInfo, VectorResultDescriptor, VectorQueryRectangle>
    + MetaDataProvider<OgrSourceDataset, VectorResultDescriptor, VectorQueryRectangle>
    + MetaDataProvider<GdalLoadingInfo, RasterResultDescriptor, RasterQueryRectangle>
{
    fn thread_pool(&self) -> ThreadPoolContext;
    fn tiling_specification(&self) -> TilingSpecification;
}

#[async_trait]
pub trait MetaDataProvider<L, R, Q>
where
    R: ResultDescriptor,
{
    async fn meta_data(&self, dataset: &DatasetId) -> Result<Box<dyn MetaData<L, R, Q>>>;
}

#[async_trait]
pub trait MetaData<L, R, Q>: Debug + Send + Sync
where
    R: ResultDescriptor,
{
    async fn loading_info(&self, query: Q) -> Result<L>;
    async fn result_descriptor(&self) -> Result<R>;

    fn pre_load_hook(&self) -> Option<&dyn PreLoadHook>;

    fn box_clone(&self) -> Box<dyn MetaData<L, R, Q>>;
}

impl<L, R, Q> Clone for Box<dyn MetaData<L, R, Q>>
where
    R: ResultDescriptor,
{
    fn clone(&self) -> Box<dyn MetaData<L, R, Q>> {
        self.box_clone()
    }
}

#[async_trait]
pub trait PreLoadHook: Debug + Send + Sync {
    async fn execute(&self) -> Result<()>;
    fn box_clone(&self) -> Box<dyn PreLoadHook>;
}

impl Clone for Box<dyn PreLoadHook> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

pub struct MockExecutionContext {
    pub thread_pool: ThreadPool,
    pub meta_data: HashMap<DatasetId, Box<dyn Any + Send + Sync>>,
    pub tiling_specification: TilingSpecification,
}

impl Default for MockExecutionContext {
    fn default() -> Self {
        Self {
            thread_pool: ThreadPool::default(),
            meta_data: HashMap::default(),
            tiling_specification: TilingSpecification {
                origin_coordinate: Default::default(),
                tile_size_in_pixels: GridShape {
                    shape_array: [600, 600],
                },
            },
        }
    }
}

impl MockExecutionContext {
    pub fn add_meta_data<L, R, Q>(
        &mut self,
        dataset: DatasetId,
        meta_data: Box<dyn MetaData<L, R, Q>>,
    ) where
        L: Send + Sync + 'static,
        R: Send + Sync + 'static + ResultDescriptor,
        Q: Send + Sync + 'static,
    {
        self.meta_data
            .insert(dataset, Box::new(meta_data) as Box<dyn Any + Send + Sync>);
    }
}

impl ExecutionContext for MockExecutionContext {
    fn thread_pool(&self) -> ThreadPoolContext {
        self.thread_pool.create_context()
    }

    fn tiling_specification(&self) -> TilingSpecification {
        self.tiling_specification
    }
}

#[async_trait]
impl<L, R, Q> MetaDataProvider<L, R, Q> for MockExecutionContext
where
    L: 'static,
    R: 'static + ResultDescriptor,
    Q: 'static,
{
    async fn meta_data(&self, dataset: &DatasetId) -> Result<Box<dyn MetaData<L, R, Q>>> {
        let meta_data = self
            .meta_data
            .get(dataset)
            .ok_or(Error::UnknownDatasetId)?
            .downcast_ref::<Box<dyn MetaData<L, R, Q>>>()
            .ok_or(Error::DatasetLoadingInfoProviderMismatch)?;

        Ok(meta_data.clone())
    }
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaticMetaData<L, R, Q>
where
    L: Debug + Clone + Send + Sync + 'static,
    R: Debug + Send + Sync + 'static + ResultDescriptor,
    Q: Debug + Clone + Send + Sync + 'static,
{
    pub loading_info: L,
    pub result_descriptor: R,
    #[serde(skip)]
    pub phantom: PhantomData<Q>,
}

#[async_trait]
impl<L, R, Q> MetaData<L, R, Q> for StaticMetaData<L, R, Q>
where
    L: Debug + Clone + Send + Sync + 'static,
    R: Debug + Send + Sync + 'static + ResultDescriptor,
    Q: Debug + Clone + Send + Sync + 'static,
{
    async fn loading_info(&self, _query: Q) -> Result<L> {
        Ok(self.loading_info.clone())
    }

    async fn result_descriptor(&self) -> Result<R> {
        Ok(self.result_descriptor.clone())
    }

    fn pre_load_hook(&self) -> Option<&dyn PreLoadHook> {
        None
    }
    fn box_clone(&self) -> Box<dyn MetaData<L, R, Q>> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct StaticMetaDataWithHook<L, R, Q>
where
    L: Debug + Clone + Send + Sync + 'static,
    R: Debug + Send + Sync + 'static + ResultDescriptor,
    Q: Debug + Clone + Send + Sync + 'static,
{
    pub loading_info: L,
    pub result_descriptor: R,
    pub phantom: PhantomData<Q>,
    pub pre_load_hook: Box<dyn PreLoadHook>,
}

#[async_trait]
impl<L, R, Q> MetaData<L, R, Q> for StaticMetaDataWithHook<L, R, Q>
where
    L: Debug + Clone + Send + Sync + 'static,
    R: Debug + Send + Sync + 'static + ResultDescriptor,
    Q: Debug + Clone + Send + Sync + 'static,
{
    async fn loading_info(&self, _query: Q) -> Result<L> {
        Ok(self.loading_info.clone())
    }

    async fn result_descriptor(&self) -> Result<R> {
        Ok(self.result_descriptor.clone())
    }

    fn pre_load_hook(&self) -> Option<&dyn PreLoadHook> {
        Some(self.pre_load_hook.as_ref())
    }
    fn box_clone(&self) -> Box<dyn MetaData<L, R, Q>> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geoengine_datatypes::collections::VectorDataType;
    use geoengine_datatypes::spatial_reference::SpatialReferenceOption;

    #[tokio::test]
    async fn test() {
        let info = StaticMetaData {
            loading_info: 1_i32,
            result_descriptor: VectorResultDescriptor {
                data_type: VectorDataType::Data,
                spatial_reference: SpatialReferenceOption::Unreferenced,
                columns: Default::default(),
            },
            phantom: Default::default(),
        };

        let info: Box<dyn MetaData<i32, VectorResultDescriptor, VectorQueryRectangle>> =
            Box::new(info);

        let info2: Box<dyn Any + Send + Sync> = Box::new(info);

        let info3 = info2
            .downcast_ref::<Box<dyn MetaData<i32, VectorResultDescriptor, VectorQueryRectangle>>>()
            .unwrap();

        assert_eq!(
            info3.result_descriptor().await.unwrap(),
            VectorResultDescriptor {
                data_type: VectorDataType::Data,
                spatial_reference: SpatialReferenceOption::Unreferenced,
                columns: Default::default(),
            }
        );
    }
}
