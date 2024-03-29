use super::DataSource;
use crate::data_types::{record_batch::RecordBatch, schema::Schema};

use anyhow::Result;

#[derive(Clone)]
pub struct MemoryDataSource {
    schema: Schema,
    data: Vec<RecordBatch>,
}

impl DataSource for MemoryDataSource {
    fn get_schema(&self) -> &Schema {
        &self.schema
    }

    fn scan(&self, projection: Vec<&str>) -> Result<Box<dyn Iterator<Item = RecordBatch> + '_>> {
        let projection_indices = projection
            .iter()
            .filter_map(|name| self.schema.fields.iter().position(|f| f.name == *name))
            .collect::<Vec<_>>();

        Ok(Box::new(self.data.iter().map(move |batch| {
            RecordBatch {
                schema: self.schema.clone(),
                fields: projection_indices
                    .iter()
                    .map(|i| batch.field(*i).clone())
                    .collect(),
            }
        })))
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::data_types::{
        arrow_field_array::ArrowFieldArray,
        column_array::{ArrayRef, DataType},
        schema::Field,
    };

    use arrow::array::Int32Array;

    #[test]
    fn test_get_schema() {
        let schema = Schema::new(vec![
            Field::new("a".to_string(), DataType::Int32),
            Field::new("b".to_string(), DataType::Int32),
        ]);
        let data_source = MemoryDataSource {
            schema: schema.clone(),
            data: vec![],
        };
        assert_eq!(data_source.get_schema(), &schema);
    }

    #[test]
    fn test_scan() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Int32)]);
        let id = Int32Array::from(vec![1, 2, 3, 4, 5]);
        let id_array = vec![Rc::new(ArrowFieldArray::new(Box::new(id))) as ArrayRef];

        let records = vec![RecordBatch::new(schema.clone(), id_array)];
        let data_source = MemoryDataSource {
            schema,
            data: records,
        };

        // None exists in the schema, so we should get an empty iterator.
        let projection = vec!["a"];
        let result: Vec<RecordBatch> = data_source.scan(projection).unwrap().collect();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].fields.len(), 0);

        let projection = vec!["id"];
        let result: Vec<RecordBatch> = data_source.scan(projection).unwrap().collect();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].fields.len(), 1);
        assert_eq!(result[0].fields[0].size(), 5);
    }
}
