pub(self) mod error;
mod feature_collection;
#[macro_use]
mod geo_feature_collection;
#[macro_use]
mod feature_collection_impl;

mod data_collection;
mod multi_line_collection;
mod multi_point_collection;
mod multi_polygon_collection;

pub(crate) use error::FeatureCollectionError;
pub use feature_collection::FeatureCollection;
pub(self) use feature_collection_impl::FeatureCollectionImplHelpers;
pub use geo_feature_collection::{IntoGeometryIterator, IntoGeometryOptionsIterator};

pub use data_collection::DataCollection;
pub use multi_line_collection::MultiLineCollection;
pub use multi_point_collection::{MultiPointCollection, MultiPointCollectionBuilder};
pub use multi_polygon_collection::MultiPolygonCollection;
