use crate::contexts::MockableSession;
use crate::datasets::listing::{DatasetListOptions, DatasetListing, DatasetProvider, OrderBy};
use crate::datasets::provenance::{ProvenanceOutput, ProvenanceProvider};
use crate::datasets::storage::{
    AddDataset, Dataset, DatasetDb, DatasetProviderDb, DatasetProviderDefinition,
    DatasetProviderListOptions, DatasetProviderListing, DatasetStore, DatasetStorer,
    MetaDataDefinition,
};
use crate::datasets::upload::{Upload, UploadDb, UploadId};
use crate::error;
use crate::error::Result;
use crate::pro::users::UserSession;
use crate::util::user_input::Validated;
use async_trait::async_trait;
use geoengine_datatypes::{
    dataset::{DatasetId, DatasetProviderId, InternalDatasetId},
    util::Identifier,
};
use geoengine_operators::engine::{
    MetaData, MetaDataProvider, RasterQueryRectangle, RasterResultDescriptor, StaticMetaData,
    TypedResultDescriptor, VectorQueryRectangle, VectorResultDescriptor,
};
use geoengine_operators::source::{GdalLoadingInfo, GdalMetaDataRegular, OgrSourceDataset};
use geoengine_operators::{mock::MockDatasetDataSourceLoadingInfo, source::GdalMetaDataStatic};
use std::collections::HashMap;

#[derive(Default)]
pub struct ProHashMapDatasetDb {
    datasets: Vec<Dataset>,
    ogr_datasets: HashMap<
        InternalDatasetId,
        StaticMetaData<OgrSourceDataset, VectorResultDescriptor, VectorQueryRectangle>,
    >,
    mock_datasets: HashMap<
        InternalDatasetId,
        StaticMetaData<
            MockDatasetDataSourceLoadingInfo,
            VectorResultDescriptor,
            VectorQueryRectangle,
        >,
    >,
    gdal_datasets: HashMap<
        InternalDatasetId,
        Box<dyn MetaData<GdalLoadingInfo, RasterResultDescriptor, RasterQueryRectangle>>,
    >,
    uploads: HashMap<UploadId, Upload>,
    external_providers: HashMap<DatasetProviderId, Box<dyn DatasetProviderDefinition>>,
}

impl DatasetDb<UserSession> for ProHashMapDatasetDb {}

#[async_trait]
impl DatasetProviderDb<UserSession> for ProHashMapDatasetDb {
    async fn add_dataset_provider(
        &mut self,
        _session: &UserSession,
        provider: Box<dyn DatasetProviderDefinition>,
    ) -> Result<DatasetProviderId> {
        let id = provider.id();
        self.external_providers.insert(id, provider);
        Ok(id)
    }

    async fn list_dataset_providers(
        &self,
        _session: &UserSession,
        _options: Validated<DatasetProviderListOptions>,
    ) -> Result<Vec<DatasetProviderListing>> {
        // TODO: use options
        Ok(self
            .external_providers
            .iter()
            .map(|(id, d)| DatasetProviderListing {
                id: *id,
                type_name: d.type_name(),
                name: d.name(),
            })
            .collect())
    }

    async fn dataset_provider(
        &self,
        _session: &UserSession,
        provider: DatasetProviderId,
    ) -> Result<Box<dyn DatasetProvider>> {
        self.external_providers
            .get(&provider)
            .cloned()
            .ok_or(error::Error::UnknownProviderId)?
            .initialize()
            .await
    }
}

pub trait ProHashMapStorable: Send + Sync {
    fn store(&self, id: InternalDatasetId, db: &mut ProHashMapDatasetDb) -> TypedResultDescriptor;
}

impl DatasetStorer for ProHashMapDatasetDb {
    type StorageType = Box<dyn ProHashMapStorable>;
}

impl ProHashMapStorable for MetaDataDefinition {
    fn store(&self, id: InternalDatasetId, db: &mut ProHashMapDatasetDb) -> TypedResultDescriptor {
        match self {
            MetaDataDefinition::MockMetaData(d) => d.store(id, db),
            MetaDataDefinition::OgrMetaData(d) => d.store(id, db),
            MetaDataDefinition::GdalMetaDataRegular(d) => d.store(id, db),
            MetaDataDefinition::GdalStatic(d) => d.store(id, db),
        }
    }
}

impl ProHashMapStorable
    for StaticMetaData<OgrSourceDataset, VectorResultDescriptor, VectorQueryRectangle>
{
    fn store(&self, id: InternalDatasetId, db: &mut ProHashMapDatasetDb) -> TypedResultDescriptor {
        db.ogr_datasets.insert(id, self.clone());
        self.result_descriptor.clone().into()
    }
}

impl ProHashMapStorable
    for StaticMetaData<
        MockDatasetDataSourceLoadingInfo,
        VectorResultDescriptor,
        VectorQueryRectangle,
    >
{
    fn store(&self, id: InternalDatasetId, db: &mut ProHashMapDatasetDb) -> TypedResultDescriptor {
        db.mock_datasets.insert(id, self.clone());
        self.result_descriptor.clone().into()
    }
}

impl ProHashMapStorable for GdalMetaDataRegular {
    fn store(&self, id: InternalDatasetId, db: &mut ProHashMapDatasetDb) -> TypedResultDescriptor {
        db.gdal_datasets.insert(id, Box::new(self.clone()));
        self.result_descriptor.clone().into()
    }
}

impl ProHashMapStorable for GdalMetaDataStatic {
    fn store(&self, id: InternalDatasetId, db: &mut ProHashMapDatasetDb) -> TypedResultDescriptor {
        db.gdal_datasets.insert(id, Box::new(self.clone()));
        self.result_descriptor.clone().into()
    }
}

#[async_trait]
impl DatasetStore<UserSession> for ProHashMapDatasetDb {
    async fn add_dataset(
        &mut self,
        _session: &UserSession,
        dataset: Validated<AddDataset>,
        meta_data: Box<dyn ProHashMapStorable>,
    ) -> Result<DatasetId> {
        let dataset = dataset.user_input;
        let id = dataset
            .id
            .unwrap_or_else(|| InternalDatasetId::new().into());
        let result_descriptor = meta_data.store(id.internal().expect("from AddDataset"), self);

        let d: Dataset = Dataset {
            id: id.clone(),
            name: dataset.name,
            description: dataset.description,
            result_descriptor,
            source_operator: dataset.source_operator,
            symbology: dataset.symbology,
            provenance: dataset.provenance,
        };
        self.datasets.push(d);

        Ok(id)
    }

    fn wrap_meta_data(&self, meta: MetaDataDefinition) -> Self::StorageType {
        Box::new(meta)
    }
}

#[async_trait]
impl DatasetProvider for ProHashMapDatasetDb {
    async fn list(
        &self,
        // _session: &UserSession,
        options: Validated<DatasetListOptions>,
    ) -> Result<Vec<DatasetListing>> {
        // TODO: permissions

        // TODO: include datasets from external dataset providers
        let options = options.user_input;

        let mut list: Vec<_> = if let Some(filter) = &options.filter {
            self.datasets
                .iter()
                .filter(|d| d.name.contains(filter) || d.description.contains(filter))
                .collect()
        } else {
            self.datasets.iter().collect()
        };

        match options.order {
            OrderBy::NameAsc => list.sort_by(|a, b| a.name.cmp(&b.name)),
            OrderBy::NameDesc => list.sort_by(|a, b| b.name.cmp(&a.name)),
        };

        let list = list
            .into_iter()
            .skip(options.offset as usize)
            .take(options.limit as usize)
            .map(Dataset::listing)
            .collect();

        Ok(list)
    }

    async fn load(
        &self,
        //  _session: &UserSession,
        dataset: &DatasetId,
    ) -> Result<Dataset> {
        // TODO: permissions

        self.datasets
            .iter()
            .find(|d| d.id == *dataset)
            .cloned()
            .ok_or(error::Error::UnknownDatasetId)
    }
}

#[async_trait]
impl
    MetaDataProvider<MockDatasetDataSourceLoadingInfo, VectorResultDescriptor, VectorQueryRectangle>
    for ProHashMapDatasetDb
{
    async fn meta_data(
        &self,
        dataset: &DatasetId,
    ) -> Result<
        Box<
            dyn MetaData<
                MockDatasetDataSourceLoadingInfo,
                VectorResultDescriptor,
                VectorQueryRectangle,
            >,
        >,
        geoengine_operators::error::Error,
    > {
        Ok(Box::new(
            self.mock_datasets
                .get(&dataset.internal().ok_or(
                    geoengine_operators::error::Error::DatasetMetaData {
                        source: Box::new(error::Error::DatasetIdTypeMissMatch),
                    },
                )?)
                .ok_or(geoengine_operators::error::Error::DatasetMetaData {
                    source: Box::new(error::Error::UnknownDatasetId),
                })?
                .clone(),
        ))
    }
}

#[async_trait]
impl MetaDataProvider<OgrSourceDataset, VectorResultDescriptor, VectorQueryRectangle>
    for ProHashMapDatasetDb
{
    async fn meta_data(
        &self,
        dataset: &DatasetId,
    ) -> Result<
        Box<dyn MetaData<OgrSourceDataset, VectorResultDescriptor, VectorQueryRectangle>>,
        geoengine_operators::error::Error,
    > {
        Ok(Box::new(
            self.ogr_datasets
                .get(&dataset.internal().ok_or(
                    geoengine_operators::error::Error::DatasetMetaData {
                        source: Box::new(error::Error::DatasetIdTypeMissMatch),
                    },
                )?)
                .ok_or(geoengine_operators::error::Error::DatasetMetaData {
                    source: Box::new(error::Error::UnknownDatasetId),
                })?
                .clone(),
        ))
    }
}

#[async_trait]
impl MetaDataProvider<GdalLoadingInfo, RasterResultDescriptor, RasterQueryRectangle>
    for ProHashMapDatasetDb
{
    async fn meta_data(
        &self,
        dataset: &DatasetId,
    ) -> Result<
        Box<dyn MetaData<GdalLoadingInfo, RasterResultDescriptor, RasterQueryRectangle>>,
        geoengine_operators::error::Error,
    > {
        let id = dataset
            .internal()
            .ok_or(geoengine_operators::error::Error::DatasetMetaData {
                source: Box::new(error::Error::DatasetIdTypeMissMatch),
            })?;

        Ok(self
            .gdal_datasets
            .get(&id)
            .ok_or(geoengine_operators::error::Error::DatasetMetaData {
                source: Box::new(error::Error::UnknownDatasetId),
            })?
            .clone())
    }
}

#[async_trait]
impl ProvenanceProvider for ProHashMapDatasetDb {
    async fn provenance(&self, dataset: &DatasetId) -> Result<ProvenanceOutput> {
        match dataset {
            DatasetId::Internal { dataset_id: _ } => self
                .datasets
                .iter()
                .find(|d| d.id == *dataset)
                .map(|d| ProvenanceOutput {
                    dataset: d.id.clone(),
                    provenance: d.provenance.clone(),
                })
                .ok_or(error::Error::UnknownDatasetId),
            DatasetId::External(id) => {
                self.dataset_provider(&UserSession::mock(), id.provider_id) // TODO: get correct session into dataset provider
                    .await?
                    .provenance(dataset)
                    .await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contexts::{Context, MockableSession};
    use crate::datasets::listing::OrderBy;
    use crate::pro::contexts::ProInMemoryContext;
    use crate::util::user_input::UserInput;
    use geoengine_datatypes::collections::VectorDataType;
    use geoengine_datatypes::spatial_reference::SpatialReferenceOption;
    use geoengine_operators::source::OgrSourceErrorSpec;

    #[tokio::test]
    async fn add_ogr_and_list() -> Result<()> {
        let ctx = ProInMemoryContext::default();

        let session = UserSession::mock(); // TODO: find suitable way for public data

        let descriptor = VectorResultDescriptor {
            data_type: VectorDataType::Data,
            spatial_reference: SpatialReferenceOption::Unreferenced,
            columns: Default::default(),
        };

        let ds = AddDataset {
            id: None,
            name: "OgrDataset".to_string(),
            description: "My Ogr dataset".to_string(),
            source_operator: "OgrSource".to_string(),
            symbology: None,
            provenance: None,
        };

        let meta = StaticMetaData {
            loading_info: OgrSourceDataset {
                file_name: Default::default(),
                layer_name: "".to_string(),
                data_type: None,
                time: Default::default(),
                columns: None,
                force_ogr_time_filter: false,
                force_ogr_spatial_filter: false,
                on_error: OgrSourceErrorSpec::Ignore,
            },
            result_descriptor: descriptor.clone(),
            phantom: Default::default(),
        };

        let id = ctx
            .dataset_db_ref_mut()
            .await
            .add_dataset(&session, ds.validated()?, Box::new(meta))
            .await?;

        let exe_ctx = ctx.execution_context(session.clone())?;

        let meta: Box<
            dyn MetaData<OgrSourceDataset, VectorResultDescriptor, VectorQueryRectangle>,
        > = exe_ctx.meta_data(&id).await?;

        assert_eq!(
            meta.result_descriptor().await?,
            VectorResultDescriptor {
                data_type: VectorDataType::Data,
                spatial_reference: SpatialReferenceOption::Unreferenced,
                columns: Default::default()
            }
        );

        let ds = ctx
            .dataset_db_ref()
            .await
            .list(
                DatasetListOptions {
                    filter: None,
                    order: OrderBy::NameAsc,
                    offset: 0,
                    limit: 1,
                }
                .validated()?,
            )
            .await?;

        assert_eq!(ds.len(), 1);

        assert_eq!(
            ds[0],
            DatasetListing {
                id,
                name: "OgrDataset".to_string(),
                description: "My Ogr dataset".to_string(),
                tags: vec![],
                source_operator: "OgrSource".to_string(),
                result_descriptor: descriptor.into(),
                symbology: None,
            }
        );

        Ok(())
    }
}

#[async_trait]
impl UploadDb<UserSession> for ProHashMapDatasetDb {
    async fn get_upload(&self, _session: &UserSession, upload: UploadId) -> Result<Upload> {
        // TODO: user permission
        self.uploads
            .get(&upload)
            .map(Clone::clone)
            .ok_or(error::Error::UnknownUploadId)
    }

    async fn create_upload(&mut self, _session: &UserSession, upload: Upload) -> Result<()> {
        // TODO: user permission
        self.uploads.insert(upload.id, upload);
        Ok(())
    }
}
