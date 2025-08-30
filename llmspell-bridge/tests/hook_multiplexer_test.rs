//! Test the hook multiplexer system that allows multiple debug hooks to coexist

use llmspell_bridge::lua::hook_multiplexer::{
    HookHandler, HookMultiplexer, HookPriority, ProfilerHook,
};
use mlua::{Debug, DebugEvent, HookTriggers, Lua, Result as LuaResult};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

/// Custom memory profiler hook
struct MemoryProfiler {
    allocations: Arc<AtomicU64>,
}

impl HookHandler for MemoryProfiler {
    fn handle_event(&mut self, _lua: &Lua, _ar: &Debug, event: DebugEvent) -> LuaResult<()> {
        if matches!(event, DebugEvent::Call) {
            self.allocations.fetch_add(1, Ordering::Relaxed);
        }
        Ok(())
    }

    fn interested_events(&self) -> HookTriggers {
        HookTriggers {
            on_calls: true,
            ..Default::default()
        }
    }
}

/// Performance monitor hook
struct PerformanceMonitor {
    line_count: Arc<AtomicU64>,
}

impl HookHandler for PerformanceMonitor {
    fn handle_event(&mut self, _lua: &Lua, _ar: &Debug, event: DebugEvent) -> LuaResult<()> {
        if matches!(event, DebugEvent::Line) {
            self.line_count.fetch_add(1, Ordering::Relaxed);
        }
        Ok(())
    }

    fn interested_events(&self) -> HookTriggers {
        HookTriggers {
            every_line: true,
            ..Default::default()
        }
    }
}

/// Test that multiple hooks can coexist through the multiplexer
#[test]
fn test_multiple_hooks_coexist() {
    let lua = Lua::new();
    let multiplexer = Arc::new(HookMultiplexer::new());

    // Set up three different hook systems
    let profiler_samples = Arc::new(AtomicU64::new(0));
    let memory_allocs = Arc::new(AtomicU64::new(0));
    let line_executions = Arc::new(AtomicU64::new(0));

    // Register a profiler (highest priority)
    let profiler = Box::new(ProfilerHook {
        sample_count: profiler_samples.clone(),
    });
    multiplexer
        .register_handler("profiler".to_string(), HookPriority::PROFILER, profiler)
        .unwrap();

    // Register a memory profiler
    let memory_profiler = Box::new(MemoryProfiler {
        allocations: memory_allocs.clone(),
    });
    multiplexer
        .register_handler(
            "memory".to_string(),
            HookPriority(-500), // Between profiler and debugger
            memory_profiler,
        )
        .unwrap();

    // Register a performance monitor (lowest priority)
    let perf_monitor = Box::new(PerformanceMonitor {
        line_count: line_executions.clone(),
    });
    multiplexer
        .register_handler(
            "performance".to_string(),
            HookPriority::MONITOR,
            perf_monitor,
        )
        .unwrap();

    // Install the multiplexer
    multiplexer.install(&lua).unwrap();

    // Run some Lua code with explicit function calls
    lua.load(
        r"
        function test_func(x)
            local y = x * 2
            return y + 1
        end
        
        function helper()
            return 42
        end
        
        for i = 1, 10 do
            test_func(i)
            helper()  -- More calls to trigger call hook
        end
    ",
    )
    .exec()
    .unwrap();

    // All three hook systems should have collected data
    let profiler_count = profiler_samples.load(Ordering::Relaxed);
    let memory_count = memory_allocs.load(Ordering::Relaxed);
    let line_count = line_executions.load(Ordering::Relaxed);

    println!(
        "Debug: profiler_count={profiler_count}, memory_count={memory_count}, line_count={line_count}"
    );

    assert!(profiler_count > 0, "Profiler should have samples");
    assert!(
        memory_count > 0,
        "Memory profiler should have tracked calls (got {memory_count})"
    );
    assert!(
        line_count > 0,
        "Performance monitor should have tracked lines"
    );

    println!("Hook multiplexer results:");
    println!(
        "  Profiler samples: {}",
        profiler_samples.load(Ordering::Relaxed)
    );
    println!(
        "  Memory allocations tracked: {}",
        memory_allocs.load(Ordering::Relaxed)
    );
    println!(
        "  Lines executed: {}",
        line_executions.load(Ordering::Relaxed)
    );
}

/// Test that hooks can be dynamically added and removed
#[test]
fn test_dynamic_hook_management() {
    let lua = Lua::new();
    let multiplexer = Arc::new(HookMultiplexer::new());

    // Start with no hooks
    assert_eq!(multiplexer.handler_count(), 0);

    // Add a profiler
    let profiler_samples = Arc::new(AtomicU64::new(0));
    multiplexer
        .register_handler(
            "profiler".to_string(),
            HookPriority::PROFILER,
            Box::new(ProfilerHook {
                sample_count: profiler_samples.clone(),
            }),
        )
        .unwrap();

    assert_eq!(multiplexer.handler_count(), 1);

    // Install and run
    multiplexer.install(&lua).unwrap();
    lua.load("for i = 1, 100 do local x = i end")
        .exec()
        .unwrap();

    let initial_samples = profiler_samples.load(Ordering::Relaxed);
    assert!(initial_samples > 0);

    // Add another hook while running
    let line_count = Arc::new(AtomicU64::new(0));
    multiplexer
        .register_handler(
            "monitor".to_string(),
            HookPriority::MONITOR,
            Box::new(PerformanceMonitor {
                line_count: line_count.clone(),
            }),
        )
        .unwrap();

    assert_eq!(multiplexer.handler_count(), 2);

    // Re-install to update hooks
    multiplexer.install(&lua).unwrap();

    // Run more code
    lua.load("for i = 1, 100 do local y = i * 2 end")
        .exec()
        .unwrap();

    // Both hooks should have data
    assert!(profiler_samples.load(Ordering::Relaxed) > initial_samples);
    assert!(line_count.load(Ordering::Relaxed) > 0);

    // Remove the profiler
    assert!(multiplexer.unregister_handler("profiler"));
    assert_eq!(multiplexer.handler_count(), 1);

    // Re-install
    multiplexer.install(&lua).unwrap();

    // Run more code
    let line_count_before = line_count.load(Ordering::Relaxed);
    let profiler_before = profiler_samples.load(Ordering::Relaxed);

    lua.load("for i = 1, 100 do local z = i + 1 end")
        .exec()
        .unwrap();

    // Only monitor should have new data
    assert!(line_count.load(Ordering::Relaxed) > line_count_before);
    assert_eq!(profiler_samples.load(Ordering::Relaxed), profiler_before); // No change
}

/// Test priority ordering
#[test]
fn test_hook_priority_ordering() {
    // Create hooks that record their execution order
    struct OrderedHook {
        id: String,
        order: Arc<parking_lot::Mutex<Vec<String>>>,
    }

    impl HookHandler for OrderedHook {
        fn handle_event(&mut self, _lua: &Lua, _ar: &Debug, event: DebugEvent) -> LuaResult<()> {
            if matches!(event, DebugEvent::Line) {
                self.order.lock().push(self.id.clone());
            }
            Ok(())
        }

        fn interested_events(&self) -> HookTriggers {
            HookTriggers {
                every_line: true,
                ..Default::default()
            }
        }
    }

    let lua = Lua::new();
    let multiplexer = Arc::new(HookMultiplexer::new());

    // Shared execution order tracker
    let execution_order = Arc::new(parking_lot::Mutex::new(Vec::new()));

    // Register hooks with different priorities
    for (id, priority) in [("first", -100), ("second", 0), ("third", 100)] {
        multiplexer
            .register_handler(
                id.to_string(),
                HookPriority(priority),
                Box::new(OrderedHook {
                    id: id.to_string(),
                    order: execution_order.clone(),
                }),
            )
            .unwrap();
    }

    // Install and run
    multiplexer.install(&lua).unwrap();
    lua.load("local x = 1").exec().unwrap();

    // Check execution order
    {
        let order = execution_order.lock();
        assert!(!order.is_empty());

        // Should be in priority order for each line
        for i in (0..order.len()).step_by(3) {
            if i + 2 < order.len() {
                assert_eq!(&order[i], "first");
                assert_eq!(&order[i + 1], "second");
                assert_eq!(&order[i + 2], "third");
            }
        }
        drop(order); // Early drop lock to avoid resource contention
    }
}
