use std::sync::Arc;

pub trait ToArrow {
    fn to_type(&self) -> arrow::datatypes::DataType;
    fn to_array(self) -> Arc<dyn arrow::array::Array>;
}

macro_rules! int_to_arrow_array {
    ($tt:ty, $dtt:expr, $att:ty) => {
        impl ToArrow for Vec<$tt> {
            fn to_type(&self) -> arrow::datatypes::DataType {
                $dtt
            }

            fn to_array(self) -> Arc<dyn arrow::array::Array> {
                // this cast normalizes the table as we only support i64 values
                let v = self.iter().map(|v| *v).collect::<Vec<_>>();
                Arc::new(<$att>::from(v))
            }
        }
    };
}

int_to_arrow_array!(
    i64,
    arrow::datatypes::DataType::Int64,
    arrow::array::Int64Array
);

macro_rules! string_to_arrow_array {
    ($tt:ty, $dtt:expr, $att:ty) => {
        impl ToArrow for Vec<$tt> {
            fn to_type(&self) -> arrow::datatypes::DataType {
                $dtt
            }

            fn to_array(self) -> Arc<dyn arrow::array::Array> {
                Arc::new(<$att>::from(self))
            }
        }
    };
}

string_to_arrow_array!(
    &str,
    arrow::datatypes::DataType::Utf8,
    arrow::array::StringArray
);
string_to_arrow_array!(
    String,
    arrow::datatypes::DataType::Utf8,
    arrow::array::StringArray
);

/// Utility macro to simplify the creation of RecordBatches
#[macro_export]
macro_rules! record_batch {
    ($($col_name:expr => $slice:expr), + $(,)?) => {
        {
            use std::sync::Arc;
            use arrow::datatypes::Field;
            use arrow::datatypes::Schema;
            use arrow::record_batch::RecordBatch;
            use $crate::base::database::ToArrow;

            let schema = Arc::new(Schema::new(
                vec![$(
                    Field::new(&$col_name.to_string(), $slice.to_vec().to_type(), false)
                ,)+]));

            let arrays = vec![$($slice.to_vec().to_array(),)+];

            RecordBatch::try_new(schema, arrays).unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::record_batch;

    use arrow::datatypes::Field;
    use arrow::datatypes::Schema;
    use arrow::record_batch::RecordBatch;
    use std::sync::Arc;

    #[test]
    fn test_record_batch_macro() {
        let batch = record_batch!(
            "f" => ["abc", "t", "fg"],
            "ghisi" => [-99, 1230, 222]
        );

        let arrays: Vec<Arc<dyn arrow::array::Array>> = vec![
            Arc::new(arrow::array::StringArray::from(["abc", "t", "fg"].to_vec())),
            Arc::new(arrow::array::Int64Array::from([-99, 1230, 222].to_vec())),
        ];

        let schema = Arc::new(Schema::new(vec![
            Field::new("f", arrow::datatypes::DataType::Utf8, false),
            Field::new("ghisi", arrow::datatypes::DataType::Int64, false),
        ]));

        let expected_batch = RecordBatch::try_new(schema, arrays).unwrap();

        assert_eq!(batch, expected_batch);
    }
}
