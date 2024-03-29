use crate::{
    data_source::{csv_data_source::CsvDataSource, Source},
    data_types::schema::Schema,
    logical_plan::{data_frame::DataFrame, plan::Plan as LogicalPlan, scan::Scan},
    optimizer::Optimizer,
    physical_plan::plan::Plan as PhysicalPlan,
    query_planner::planner::QueryPlanner,
};

use anyhow::Result;

pub struct ExecutionContext {
    batch_size: usize,
}

impl ExecutionContext {
    pub fn new(batch_size: usize) -> Self {
        ExecutionContext { batch_size }
    }

    pub fn csv(&self, file_path: String, schema: Schema) -> DataFrame {
        let csv_data_source = CsvDataSource::new(file_path.clone(), schema, self.batch_size);
        let scan_plan = Scan::new(file_path, Source::Csv(csv_data_source), vec![]);
        DataFrame::new(LogicalPlan::Scan(scan_plan))
    }

    pub fn create_physical_plan(&self, df: &DataFrame) -> Result<PhysicalPlan> {
        let optimized_plan = Optimizer::optimize(&df.logical_plan());
        QueryPlanner::create_physical_plan(&optimized_plan)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        data_types::{column_array::DataType, schema::Field},
        logical_plan::expr_fn::{col, lit},
        physical_plan::plan::PhysicalPlan,
        test_util::rq_test_data,
    };

    #[test]
    fn test_execute_data_frame() {
        let ctx = ExecutionContext::new(3);
        let data_path = rq_test_data("primitive_field.csv");
        let schema = Schema::new(vec![
            Field::new("c1".to_string(), DataType::Int32),
            Field::new("c2".to_string(), DataType::Int32),
            Field::new("c3".to_string(), DataType::Int64),
            Field::new("c4".to_string(), DataType::Int64),
        ]);
        let df = ctx
            .csv(data_path, schema)
            .filter(col("c1").eq(lit(1_i32)))
            .project(vec![col("c1"), col("c2"), col("c3")]);
        let physical_plan = ctx.create_physical_plan(&df).unwrap();
        let batches = physical_plan.execute();
        assert!(batches.is_ok());
        let mut batches = batches.unwrap();
        let first = batches.next().unwrap();
        assert_eq!(first.row_count(), 1);
        assert_eq!(first.column_count(), 3);
        assert_eq!(
            first
                .field(0)
                .get_value(0)
                .unwrap()
                .downcast_ref::<i32>()
                .unwrap(),
            &1
        )
    }
}
