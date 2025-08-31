//! Criterion benchmark adapter

use std::sync::Arc;
use criterion::{Criterion, black_box};
use crate::test_framework::{
    TestExecutor, ExecutionContext, ExecutionMode, 
    WorkloadClass, TelemetryCollector
};

/// Adapter to run TestExecutor implementations as Criterion benchmarks
pub struct CriterionAdapter<E: TestExecutor> {
    executor: Arc<E>,
    workload: Option<WorkloadClass>,
    custom_config: Option<E::Config>,
}

impl<E: TestExecutor + 'static> CriterionAdapter<E> {
    /// Create a new Criterion adapter
    pub fn new(executor: E) -> Self {
        Self {
            executor: Arc::new(executor),
            workload: None,
            custom_config: None,
        }
    }
    
    /// Set a specific workload class
    pub fn with_workload(mut self, workload: WorkloadClass) -> Self {
        self.workload = Some(workload);
        self
    }
    
    /// Set a custom configuration
    pub fn with_config(mut self, config: E::Config) -> Self {
        self.custom_config = Some(config);
        self
    }
    
    /// Run the benchmark
    pub fn bench(self, c: &mut Criterion, name: &str) {
        let config = self.custom_config.unwrap_or_else(|| self.executor.default_config());
        let workload = self.workload.unwrap_or_else(|| WorkloadClass::from_env());
        
        let context = ExecutionContext {
            config: config.clone(),
            mode: ExecutionMode::Bench,
            telemetry: Arc::new(TelemetryCollector::new()),
            timeout: Some(workload.timeout()),
        };
        
        c.bench_function(name, |b| {
            let executor = self.executor.clone();
            let ctx = context.clone();
            
            b.iter(|| {
                // Create a new runtime for each iteration
                let runtime = tokio::runtime::Runtime::new().unwrap();
                let result = runtime.block_on(executor.execute(ctx.clone()));
                black_box(result);
            });
        });
    }
    
    /// Run the benchmark with custom iterations
    pub fn bench_with_iterations(self, c: &mut Criterion, name: &str, iterations: u64) {
        let config = self.custom_config.unwrap_or_else(|| self.executor.default_config());
        let workload = self.workload.unwrap_or_else(|| WorkloadClass::from_env());
        
        let context = ExecutionContext {
            config: config.clone(),
            mode: ExecutionMode::Bench,
            telemetry: Arc::new(TelemetryCollector::new()),
            timeout: Some(workload.timeout()),
        };
        
        let mut group = c.benchmark_group(name);
        group.sample_size(iterations as usize);
        
        group.bench_function(name, |b| {
            let executor = self.executor.clone();
            let ctx = context.clone();
            
            b.iter(|| {
                let runtime = tokio::runtime::Runtime::new().unwrap();
                let result = runtime.block_on(executor.execute(ctx.clone()));
                black_box(result);
            });
        });
        
        group.finish();
    }
}