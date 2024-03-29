use std::fmt::Display;

use super::{
    expr::{Expr, PhysicalExpr},
    plan::{PhysicalPlan, Plan},
};
use crate::data_types::{record_batch::RecordBatch, schema::Schema};

use anyhow::Result;

/// Execute a projection.
pub struct ProjectionExec {
    input: Box<Plan>,
    schema: Schema,
    expr: Vec<Expr>,
}

impl ProjectionExec {
    pub fn new(input: Plan, schema: Schema, expr: Vec<Expr>) -> Self {
        Self {
            input: Box::new(input),
            schema,
            expr,
        }
    }
}

impl PhysicalPlan for ProjectionExec {
    fn schema(&self) -> Schema {
        self.schema.clone()
    }

    fn execute(&self) -> Result<Box<dyn Iterator<Item = RecordBatch> + '_>> {
        let input = self.input.execute()?;
        Ok(Box::new(input.map(|b| {
            let fields = self
                .expr
                .iter()
                .map(|e| e.evaluate(&b).expect("evaluate expr failed"))
                .collect::<Vec<_>>();
            RecordBatch::new(self.schema.clone(), fields)
        })))
    }

    fn children(&self) -> Vec<&Plan> {
        vec![&self.input]
    }
}

impl Display for ProjectionExec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ProjectionExec: {}",
            self.expr
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        data_source::{csv_data_source::CsvDataSource, Source},
        data_types::{column_array::DataType, schema::Field},
        physical_plan::{expr::Column, scan::ScanExec},
        test_util::rq_test_data,
    };

    #[test]
    fn test_projection_execute() {
        let data_path = rq_test_data("boolean_field.csv");
        let schema = Schema::new(vec![Field::new("c1".to_string(), DataType::Boolean)]);
        let csv_data_source = CsvDataSource::new(data_path, schema.clone(), 3);
        let scan = ScanExec::new(Source::Csv(csv_data_source), vec!["c1".to_string()]);
        let projection =
            ProjectionExec::new(Plan::Scan(scan), schema, vec![Expr::Column(Column::new(0))]);
        assert!(projection.execute().is_ok());
        assert_eq!(projection.execute().unwrap().count(), 1);
        assert!(projection
            .execute()
            .unwrap()
            .next()
            .unwrap()
            .field(0)
            .get_value(0)
            .unwrap()
            .downcast_ref::<bool>()
            .unwrap())
    }

    #[test]
    fn test_display() {
        let data_path = rq_test_data("boolean_field.csv");
        let schema = Schema::new(vec![Field::new("c1".to_string(), DataType::Boolean)]);
        let csv_data_source = CsvDataSource::new(data_path, schema.clone(), 3);
        let scan = ScanExec::new(Source::Csv(csv_data_source), vec!["c1".to_string()]);
        let projection =
            ProjectionExec::new(Plan::Scan(scan), schema, vec![Expr::Column(Column::new(0))]);
        assert_eq!(projection.to_string(), "ProjectionExec: #0");
    }
}
