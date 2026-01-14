use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlanError {
    #[error("无法为该查询生成执行计划: {0}")]
    OptimizationFailed(String),
    #[error("字段 `{0}` 在上下文中不存在")]
    ColumnAmbiguous(String),
}
