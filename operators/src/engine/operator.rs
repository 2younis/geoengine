use serde::{Deserialize, Serialize};

use crate::error;
use crate::util::Result;

use super::{
    query_processor::{TypedRasterQueryProcessor, TypedVectorQueryProcessor},
    CloneablePlotOperator, CloneableRasterOperator, CloneableVectorOperator, ExecutionContext,
    PlotResultDescriptor, QueryProcessor, RasterResultDescriptor, ResultDescriptor,
    TypedPlotQueryProcessor, VectorResultDescriptor,
};

/// Common methods for `RasterOperator`s
#[typetag::serde(tag = "type")]
pub trait RasterOperator: CloneableRasterOperator + Send + Sync + std::fmt::Debug {
    fn initialize(
        self: Box<Self>,
        context: &dyn ExecutionContext,
    ) -> Result<Box<InitializedRasterOperator>>;

    /// Wrap a box around a `RasterOperator`
    fn boxed(self) -> Box<dyn RasterOperator>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

/// Common methods for `VectorOperator`s
#[typetag::serde(tag = "type")]
pub trait VectorOperator: CloneableVectorOperator + Send + Sync + std::fmt::Debug {
    fn initialize(
        self: Box<Self>,
        context: &dyn ExecutionContext,
    ) -> Result<Box<InitializedVectorOperator>>;

    /// Wrap a box around a `VectorOperator`
    fn boxed(self) -> Box<dyn VectorOperator>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

/// Common methods for `PlotOperator`s
#[typetag::serde(tag = "type")]
pub trait PlotOperator: CloneablePlotOperator + Send + Sync + std::fmt::Debug {
    fn initialize(
        self: Box<Self>,
        context: &dyn ExecutionContext,
    ) -> Result<Box<InitializedPlotOperator>>;

    /// Wrap a box around a `PlotOperator`
    fn boxed(self) -> Box<dyn PlotOperator>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

pub trait InitializedOperatorBase {
    type Descriptor: ResultDescriptor + Clone;

    /// Get the result descriptor of the `Operator`
    fn result_descriptor(&self) -> &Self::Descriptor;

    /// Get the sources of the `Operator`
    fn raster_sources(&self) -> &[Box<InitializedRasterOperator>];

    /// Get the sources of the `Operator`
    fn vector_sources(&self) -> &[Box<InitializedVectorOperator>];

    /// Get the sources of the `Operator`
    fn raster_sources_mut(&mut self) -> &mut [Box<InitializedRasterOperator>];

    /// Get the sources of the `Operator`
    fn vector_sources_mut(&mut self) -> &mut [Box<InitializedVectorOperator>];
}

pub type InitializedVectorOperator =
    dyn InitializedOperator<VectorResultDescriptor, TypedVectorQueryProcessor>;

pub type InitializedRasterOperator =
    dyn InitializedOperator<RasterResultDescriptor, TypedRasterQueryProcessor>;

pub type InitializedPlotOperator =
    dyn InitializedOperator<PlotResultDescriptor, TypedPlotQueryProcessor>;

pub trait InitializedOperator<R, Q>: InitializedOperatorBase<Descriptor = R> + Send + Sync
where
    R: ResultDescriptor,
{
    /// Instantiate a `TypedVectorQueryProcessor` from a `RasterOperator`
    fn query_processor(&self) -> Result<Q>;

    /// Wrap a box around a `RasterOperator`
    fn boxed(self) -> Box<dyn InitializedOperator<R, Q>>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

impl<R> InitializedOperatorBase for Box<dyn InitializedOperatorBase<Descriptor = R>>
where
    R: ResultDescriptor + std::clone::Clone,
{
    type Descriptor = R;

    fn result_descriptor(&self) -> &Self::Descriptor {
        self.as_ref().result_descriptor()
    }
    fn raster_sources(&self) -> &[Box<InitializedRasterOperator>] {
        self.as_ref().raster_sources()
    }
    fn vector_sources(&self) -> &[Box<InitializedVectorOperator>] {
        self.as_ref().vector_sources()
    }
    fn raster_sources_mut(&mut self) -> &mut [Box<InitializedRasterOperator>] {
        self.as_mut().raster_sources_mut()
    }
    fn vector_sources_mut(&mut self) -> &mut [Box<InitializedVectorOperator>] {
        self.as_mut().vector_sources_mut()
    }
}

impl<R, Q> InitializedOperatorBase for Box<dyn InitializedOperator<R, Q>>
where
    R: ResultDescriptor,
    Q: QueryProcessor,
{
    type Descriptor = R;
    fn result_descriptor(&self) -> &Self::Descriptor {
        self.as_ref().result_descriptor()
    }
    fn raster_sources(&self) -> &[Box<InitializedRasterOperator>] {
        self.as_ref().raster_sources()
    }
    fn vector_sources(&self) -> &[Box<InitializedVectorOperator>] {
        self.as_ref().vector_sources()
    }
    fn raster_sources_mut(&mut self) -> &mut [Box<InitializedRasterOperator>] {
        self.as_mut().raster_sources_mut()
    }
    fn vector_sources_mut(&mut self) -> &mut [Box<InitializedVectorOperator>] {
        self.as_mut().vector_sources_mut()
    }
}

impl<R, Q> InitializedOperator<R, Q> for Box<dyn InitializedOperator<R, Q>>
where
    R: ResultDescriptor,
    Q: QueryProcessor,
{
    fn query_processor(&self) -> Result<Q> {
        self.as_ref().query_processor()
    }
}

/// An enum to differentiate between `Operator` variants
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "operator")]
pub enum TypedOperator {
    Vector(Box<dyn VectorOperator>),
    Raster(Box<dyn RasterOperator>),
    Plot(Box<dyn PlotOperator>),
}

impl TypedOperator {
    pub fn get_vector(self) -> Result<Box<dyn VectorOperator>> {
        if let TypedOperator::Vector(o) = self {
            return Ok(o);
        }
        Err(error::Error::InvalidOperatorType)
    }

    pub fn get_raster(self) -> Result<Box<dyn RasterOperator>> {
        if let TypedOperator::Raster(o) = self {
            return Ok(o);
        }
        Err(error::Error::InvalidOperatorType)
    }

    pub fn get_plot(self) -> Result<Box<dyn PlotOperator>> {
        if let TypedOperator::Plot(o) = self {
            return Ok(o);
        }
        Err(error::Error::InvalidOperatorType)
    }
}

impl From<Box<dyn VectorOperator>> for TypedOperator {
    fn from(operator: Box<dyn VectorOperator>) -> Self {
        Self::Vector(operator)
    }
}

impl From<Box<dyn RasterOperator>> for TypedOperator {
    fn from(operator: Box<dyn RasterOperator>) -> Self {
        Self::Raster(operator)
    }
}

impl From<Box<dyn PlotOperator>> for TypedOperator {
    fn from(operator: Box<dyn PlotOperator>) -> Self {
        Self::Plot(operator)
    }
}

/// An enum to differentiate between `InitializedOperator` variants
pub enum TypedInitializedOperator {
    Vector(Box<InitializedVectorOperator>),
    Raster(Box<InitializedRasterOperator>),
    Plot(Box<InitializedPlotOperator>),
}

impl From<Box<InitializedVectorOperator>> for TypedInitializedOperator {
    fn from(operator: Box<InitializedVectorOperator>) -> Self {
        TypedInitializedOperator::Vector(operator)
    }
}

impl From<Box<InitializedRasterOperator>> for TypedInitializedOperator {
    fn from(operator: Box<InitializedRasterOperator>) -> Self {
        TypedInitializedOperator::Raster(operator)
    }
}

impl From<Box<InitializedPlotOperator>> for TypedInitializedOperator {
    fn from(operator: Box<InitializedPlotOperator>) -> Self {
        TypedInitializedOperator::Plot(operator)
    }
}

#[macro_export]
macro_rules! call_on_typed_operator {
    ($typed_operator:expr, $operator_var:ident => $function_call:expr) => {
        match $typed_operator {
            $crate::engine::TypedOperator::Vector($operator_var) => $function_call,
            $crate::engine::TypedOperator::Raster($operator_var) => $function_call,
            $crate::engine::TypedOperator::Plot($operator_var) => $function_call,
        }
    };
}
