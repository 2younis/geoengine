use crate::datasets::listing::{DataSetListOptions, DataSetListing, DataSetProvider};
use crate::datasets::storage::{
    AddDataSet, AddDataSetProvider, DataSetDB, DataSetPermission, DataSetProviderListOptions,
    DataSetProviderListing, ImportDataSet, RasterLoadingInfo, VectorLoadingInfo,
};
use crate::error::Result;
use crate::users::user::UserId;
use crate::util::user_input::Validated;
use async_trait::async_trait;
use futures::stream::BoxStream;
use geoengine_datatypes::collections::{FeatureCollection, TypedFeatureCollection};
use geoengine_datatypes::dataset::{
    DataSetId, DataSetProviderId, InternalDataSetId, StagingDataSetId,
};
use geoengine_datatypes::primitives::Geometry;
use geoengine_datatypes::raster::{Pixel, RasterTile2D};
use geoengine_operators::engine::{
    LoadingInfo, LoadingInfoProvider, RasterResultDescriptor, VectorResultDescriptor,
};
use geoengine_operators::mock::MockDataSetDataSourceLoadingInfo;
use geoengine_operators::source::OgrSourceDataset;

#[derive(Debug)]
pub struct PostgresDataSetDB {}

#[async_trait]
impl DataSetDB for PostgresDataSetDB {
    async fn add_raster_data(
        &mut self,
        _user: UserId,
        _data_set_info: Validated<AddDataSet>,
        _loading_info: RasterLoadingInfo,
        _result: RasterResultDescriptor,
    ) -> Result<InternalDataSetId> {
        todo!()
    }

    async fn add_vector_data(
        &mut self,
        _user: UserId,
        _data_set_info: Validated<AddDataSet>,
        _loading_info: VectorLoadingInfo,
        _result: VectorResultDescriptor,
    ) -> Result<InternalDataSetId> {
        todo!()
    }

    async fn stage_raster_data(
        &mut self,
        _user: UserId,
        _loading_info: RasterLoadingInfo,
        _result: RasterResultDescriptor,
    ) -> Result<StagingDataSetId> {
        todo!()
    }

    async fn stage_vector_data(
        &mut self,
        _user: UserId,
        _loading_info: VectorLoadingInfo,
        _result: VectorResultDescriptor,
    ) -> Result<StagingDataSetId> {
        todo!()
    }

    async fn unstage_data(&mut self, _user: UserId, _data_set: StagingDataSetId) -> Result<()> {
        todo!()
    }

    async fn import_raster_data<T: Pixel>(
        &mut self,
        _user: UserId,
        _data_set: Validated<ImportDataSet>,
        _stream: BoxStream<'_, geoengine_operators::util::Result<RasterTile2D<T>>>,
        _meta: RasterResultDescriptor,
    ) -> Result<InternalDataSetId> {
        todo!()
    }

    async fn import_vector_data<G: Geometry>(
        &mut self,
        _user: UserId,
        _data_set: Validated<ImportDataSet>,
        _stream: BoxStream<'_, geoengine_operators::util::Result<FeatureCollection<G>>>,
        _meta: VectorResultDescriptor,
    ) -> Result<InternalDataSetId>
    where
        FeatureCollection<G>: Into<TypedFeatureCollection>,
    {
        todo!()
    }

    async fn add_data_set_permission(
        &mut self,
        _data_set: InternalDataSetId,
        _user: UserId,
        _permission: DataSetPermission,
    ) -> Result<()> {
        todo!()
    }

    async fn remove_data_set_permission(
        &mut self,
        _data_set: InternalDataSetId,
        _user: UserId,
        _permission: DataSetPermission,
    ) -> Result<()> {
        todo!()
    }

    async fn add_data_set_provider(
        &mut self,
        _user: UserId,
        _provider: Validated<AddDataSetProvider>,
    ) -> Result<DataSetProviderId> {
        todo!()
    }

    async fn list_data_set_providers(
        &self,
        _user: UserId,
        _options: Validated<DataSetProviderListOptions>,
    ) -> Result<Vec<DataSetProviderListing>> {
        todo!()
    }

    async fn data_set_provider(
        &self,
        _user: UserId,
        _provider: DataSetProviderId,
    ) -> Result<&dyn DataSetProvider> {
        todo!()
    }
}

impl LoadingInfoProvider<MockDataSetDataSourceLoadingInfo, VectorResultDescriptor>
    for PostgresDataSetDB
{
    fn loading_info(
        &self,
        _data_set: &DataSetId,
    ) -> Result<
        Box<dyn LoadingInfo<MockDataSetDataSourceLoadingInfo, VectorResultDescriptor>>,
        geoengine_operators::error::Error,
    > {
        todo!()
    }
}

impl LoadingInfoProvider<OgrSourceDataset, VectorResultDescriptor> for PostgresDataSetDB {
    fn loading_info(
        &self,
        _data_set: &DataSetId,
    ) -> Result<
        Box<dyn LoadingInfo<OgrSourceDataset, VectorResultDescriptor>>,
        geoengine_operators::error::Error,
    > {
        todo!()
    }
}

#[async_trait]
impl DataSetProvider for PostgresDataSetDB {
    async fn list(
        &self,
        _user: UserId,
        _options: Validated<DataSetListOptions>,
    ) -> Result<Vec<DataSetListing>> {
        todo!()
    }
}
