//! Real-world performance test for IO buffering with actual file I/O

use llmspell_core::io::{BufferedStream, IOStream};
use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tempfile::tempdir;

/// File-based IO stream for realistic benchmarking
struct FileStream {
    file: Mutex<File>,
}

impl FileStream {
    fn new(file: File) -> Self {
        Self {
            file: Mutex::new(file),
        }
    }
}

impl IOStream for FileStream {
    fn write(&self, data: &str) -> Result<(), llmspell_core::error::LLMSpellError> {
        let mut file = self.file.lock().unwrap();
        file.write_all(data.as_bytes())
            .map_err(|e| llmspell_core::error::LLMSpellError::Io {
                operation: "write".to_string(),
                source: e,
            })
    }

    fn write_line(&self, line: &str) -> Result<(), llmspell_core::error::LLMSpellError> {
        let mut file = self.file.lock().unwrap();
        writeln!(file, "{}", line)
            .map_err(|e| llmspell_core::error::LLMSpellError::Io {
                operation: "write_line".to_string(),
                source: e,
            })
    }

    fn flush(&self) -> Result<(), llmspell_core::error::LLMSpellError> {
        let mut file = self.file.lock().unwrap();
        file.flush()
            .map_err(|e| llmspell_core::error::LLMSpellError::Io {
                operation: "flush".to_string(),
                source: e,
            })
    }
}

#[test]
#[ignore] // Ignore by default as it creates temp files
fn test_real_io_performance_improvement() {
    let dir = tempdir().unwrap();
    let iterations = 10000;
    
    // Test unbuffered file writes
    let unbuffered_path = dir.path().join("unbuffered.txt");
    let unbuffered_file = File::create(&unbuffered_path).unwrap();
    let unbuffered_stream = Arc::new(FileStream::new(unbuffered_file));
    
    let start = Instant::now();
    for i in 0..iterations {
        unbuffered_stream
            .write_line(&format!("Line {}: This is a test line with some content", i))
            .unwrap();
    }
    unbuffered_stream.flush().unwrap();
    let unbuffered_duration = start.elapsed();
    
    // Test buffered file writes
    let buffered_path = dir.path().join("buffered.txt");
    let buffered_file = File::create(&buffered_path).unwrap();
    let file_stream = Arc::new(FileStream::new(buffered_file));
    let buffered_stream = BufferedStream::with_interval(
        file_stream,
        100, // batch size
        Duration::from_millis(50),
    );
    
    let start = Instant::now();
    for i in 0..iterations {
        buffered_stream
            .write_line(&format!("Line {}: This is a test line with some content", i))
            .unwrap();
    }
    buffered_stream.flush().unwrap();
    let buffered_duration = start.elapsed();
    
    // Calculate improvement
    let improvement = unbuffered_duration.as_secs_f64() / buffered_duration.as_secs_f64();
    
    println!("Real I/O Performance Test Results:");
    println!("Unbuffered: {:?}", unbuffered_duration);
    println!("Buffered: {:?}", buffered_duration);
    println!("Improvement: {:.2}x faster", improvement);
    
    // With real I/O, buffering should provide significant improvement
    assert!(
        improvement >= 2.0,
        "Buffered I/O should be at least 2x faster with real files, got {:.2}x",
        improvement
    );
    
    // Verify both files have the same content
    let unbuffered_content = std::fs::read_to_string(unbuffered_path).unwrap();
    let buffered_content = std::fs::read_to_string(buffered_path).unwrap();
    assert_eq!(
        unbuffered_content.lines().count(),
        buffered_content.lines().count(),
        "Both files should have the same number of lines"
    );
}

#[test]
fn test_buffering_reduces_syscalls() {
    // This test demonstrates that buffering reduces the number of underlying write operations
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    /// Counting stream that tracks number of write operations
    struct CountingStream {
        write_count: Arc<AtomicUsize>,
        write_line_count: Arc<AtomicUsize>,
    }
    
    impl CountingStream {
        fn new() -> Self {
            Self {
                write_count: Arc::new(AtomicUsize::new(0)),
                write_line_count: Arc::new(AtomicUsize::new(0)),
            }
        }
        
        fn total_operations(&self) -> usize {
            self.write_count.load(Ordering::Relaxed) +
            self.write_line_count.load(Ordering::Relaxed)
        }
    }
    
    impl IOStream for CountingStream {
        fn write(&self, _data: &str) -> Result<(), llmspell_core::error::LLMSpellError> {
            self.write_count.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }
        
        fn write_line(&self, _line: &str) -> Result<(), llmspell_core::error::LLMSpellError> {
            self.write_line_count.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }
        
        fn flush(&self) -> Result<(), llmspell_core::error::LLMSpellError> {
            Ok(())
        }
    }
    
    // Test unbuffered - each write_line is a separate operation
    let unbuffered = Arc::new(CountingStream::new());
    for i in 0..100 {
        unbuffered.write_line(&format!("Line {}", i)).unwrap();
    }
    let unbuffered_ops = unbuffered.total_operations();
    
    // Test buffered - operations are batched
    let counting = Arc::new(CountingStream::new());
    let buffered = BufferedStream::with_interval(
        counting.clone(),
        10, // batch size
        Duration::from_secs(1), // long timeout to ensure batching
    );
    
    for i in 0..100 {
        buffered.write_line(&format!("Line {}", i)).unwrap();
    }
    buffered.flush().unwrap();
    
    let buffered_ops = counting.total_operations();
    
    println!("Syscall reduction test:");
    println!("Unbuffered operations: {}", unbuffered_ops);
    println!("Buffered operations: {}", buffered_ops);
    println!("Reduction: {:.2}x", unbuffered_ops as f64 / buffered_ops as f64);
    
    // Buffering should significantly reduce the number of operations
    assert!(
        buffered_ops < unbuffered_ops / 5,
        "Buffering should reduce operations by at least 5x, got {} vs {}",
        buffered_ops,
        unbuffered_ops
    );
}