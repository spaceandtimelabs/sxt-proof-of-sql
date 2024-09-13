use super::curve_25519_scalar::Curve25519Scalar;
use crate::base::database::{
    test_accessor::TestAccessor, Column, ColumnRef, ColumnType, CommitmentAccessor, DataAccessor,
    MetadataAccessor, SchemaAccessor, TableRef,
};
use curve25519_dalek::RistrettoPoint;
use proof_of_sql_parser::Identifier;

#[derive(Clone, Default)]
/// An inner product test accessor that leaves all of the required methods except `new` `unimplemented!()`.
pub struct UnimplementedInnerProductTestAccessor;
impl TestAccessor<RistrettoPoint> for UnimplementedInnerProductTestAccessor {
    type Table = ();

    fn new_empty() -> Self {
        Default::default()
    }

    fn add_table(&mut self, _table_ref: TableRef, _data: (), _table_offset: usize) {
        unimplemented!()
    }

    fn get_column_names(&self, _table_ref: TableRef) -> Vec<&str> {
        unimplemented!()
    }

    fn update_offset(&mut self, _table_ref: TableRef, _new_offset: usize) {
        unimplemented!()
    }
}
impl DataAccessor<Curve25519Scalar> for UnimplementedInnerProductTestAccessor {
    fn get_column(&self, _column: ColumnRef) -> Column<Curve25519Scalar> {
        unimplemented!()
    }
}
impl CommitmentAccessor<RistrettoPoint> for UnimplementedInnerProductTestAccessor {
    fn get_commitment(&self, _column: ColumnRef) -> RistrettoPoint {
        unimplemented!()
    }
}
impl MetadataAccessor for UnimplementedInnerProductTestAccessor {
    fn get_length(&self, _table_ref: TableRef) -> usize {
        unimplemented!()
    }

    fn get_offset(&self, _table_ref: TableRef) -> usize {
        unimplemented!()
    }
}
impl SchemaAccessor for UnimplementedInnerProductTestAccessor {
    fn lookup_column(&self, _table_ref: TableRef, _column_id: Identifier) -> Option<ColumnType> {
        unimplemented!()
    }

    fn lookup_schema(&self, _table_ref: TableRef) -> Vec<(Identifier, ColumnType)> {
        unimplemented!()
    }
}
