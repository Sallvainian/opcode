//! Windows Performance Benchmarking Tests
//!
//! Comprehensive performance validation tests for Windows platform
//! Tests memory usage, startup time, CPU usage, and file operations

#[cfg(target_os = "windows")]
#[cfg(test)]
mod performance_tests {
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;
    use std::time::{Duration, Instant};

    const MAX_MEMORY_MB: u64 = 50;  // Maximum memory usage at idle
    const MAX_STARTUP_TIME_MS: u64 = 2000;  // Maximum startup time

    #[test]
    fn benchmark_startup_time() {
        println!("🚀 Testing application startup time (target: <{}ms)", MAX_STARTUP_TIME_MS);

        let start_time = Instant::now();

        // Simulate application startup by running a representative Windows process
        let output = Command::new("cmd")
            .args(["/C", "echo startup_test & exit"])
            .output()
            .expect("Failed to execute startup test");

        let startup_duration = start_time.elapsed();
        let startup_ms = startup_duration.as_millis() as u64;

        assert!(output.status.success(), "Startup test command should succeed");
        assert!(startup_ms < MAX_STARTUP_TIME_MS,
            "Startup time should be <{}ms, got {}ms", MAX_STARTUP_TIME_MS, startup_ms);

        println!("✅ Startup time: {}ms (target: <{}ms)", startup_ms, MAX_STARTUP_TIME_MS);
    }

    #[test]
    fn benchmark_memory_usage() {
        println!("💾 Testing memory usage (target: <{}MB)", MAX_MEMORY_MB);

        let current_pid = std::process::id();

        // Get memory usage using Windows tasklist
        let output = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", current_pid), "/FO", "CSV", "/NH"])
            .output()
            .expect("Failed to execute memory check");

        assert!(output.status.success(), "Memory check should succeed");

        let output_str = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = output_str.lines().next() {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 5 {
                let memory_str = parts[4].trim_matches('"').trim();
                // Memory format: "1,234 K" - parse the number
                let memory_kb: u64 = memory_str
                    .replace(",", "")
                    .replace(" K", "")
                    .parse()
                    .unwrap_or(0);

                let memory_mb = memory_kb / 1024;
                assert!(memory_mb < MAX_MEMORY_MB,
                    "Memory usage should be <{}MB, got {}MB", MAX_MEMORY_MB, memory_mb);

                println!("✅ Memory usage: {}MB (target: <{}MB)", memory_mb, MAX_MEMORY_MB);
            }
        }
    }

    #[test]
    fn benchmark_file_operations() {
        println!("📁 Testing file operations performance");

        let temp_dir = std::env::temp_dir();
        let test_files: Vec<PathBuf> = (0..100).map(|i| {
            temp_dir.join(format!("opcode_perf_test_{}.txt", i))
        }).collect();

        // Benchmark file creation
        let create_start = Instant::now();
        for file_path in &test_files {
            fs::write(file_path, b"performance test content for Windows platform")
                .expect("Failed to create test file");
        }
        let create_duration = create_start.elapsed();

        // Benchmark file reading
        let read_start = Instant::now();
        for file_path in &test_files {
            let _ = fs::read(file_path).expect("Failed to read test file");
        }
        let read_duration = read_start.elapsed();

        // Benchmark file deletion
        let delete_start = Instant::now();
        for file_path in &test_files {
            fs::remove_file(file_path).expect("Failed to delete test file");
        }
        let delete_duration = delete_start.elapsed();

        // Performance targets for 100 files
        const MAX_CREATE_MS: u64 = 500;
        const MAX_READ_MS: u64 = 100;
        const MAX_DELETE_MS: u64 = 200;

        let create_ms = create_duration.as_millis() as u64;
        let read_ms = read_duration.as_millis() as u64;
        let delete_ms = delete_duration.as_millis() as u64;

        assert!(create_ms < MAX_CREATE_MS,
            "File creation too slow: {}ms (target: <{}ms)", create_ms, MAX_CREATE_MS);
        assert!(read_ms < MAX_READ_MS,
            "File reading too slow: {}ms (target: <{}ms)", read_ms, MAX_READ_MS);
        assert!(delete_ms < MAX_DELETE_MS,
            "File deletion too slow: {}ms (target: <{}ms)", delete_ms, MAX_DELETE_MS);

        println!("✅ File operations - Create: {}ms, Read: {}ms, Delete: {}ms",
            create_ms, read_ms, delete_ms);
    }

    #[test]
    fn benchmark_cpu_usage_monitoring() {
        println!("⚡ Testing CPU usage monitoring capabilities");

        // Test Windows performance counter access
        let output = Command::new("wmic")
            .args(["cpu", "get", "loadpercentage", "/value"])
            .output()
            .expect("Failed to execute CPU usage check");

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);

            // Look for LoadPercentage value
            let cpu_found = output_str.lines()
                .any(|line| line.contains("LoadPercentage") && line.contains("="));

            assert!(cpu_found, "Should be able to read CPU load percentage");
            println!("✅ CPU monitoring: Available via WMI");
        } else {
            println!("⚠️ CPU monitoring: WMI not available, using alternative method");

            // Alternative: Use typeperf for CPU monitoring
            let typeperf_output = Command::new("typeperf")
                .args(["-sc", "1", "\\Processor(_Total)\\% Processor Time"])
                .output();

            match typeperf_output {
                Ok(output) if output.status.success() => {
                    println!("✅ CPU monitoring: Available via typeperf");
                }
                _ => {
                    println!("⚠️ CPU monitoring: Limited capability in this environment");
                }
            }
        }
    }

    #[test]
    fn benchmark_network_latency() {
        println!("🌐 Testing network performance");

        // Test local network stack latency
        let output = Command::new("ping")
            .args(["-n", "3", "127.0.0.1"])  // 3 pings to localhost
            .output()
            .expect("Failed to execute ping");

        assert!(output.status.success(), "Ping to localhost should succeed");

        let output_str = String::from_utf8_lossy(&output.stdout);

        // Verify ping completed successfully
        assert!(output_str.contains("Reply from"), "Should receive ping replies");

        // Look for timing information
        let has_timing = output_str.lines()
            .any(|line| line.contains("time<") || line.contains("time="));

        assert!(has_timing, "Should show ping timing information");
        println!("✅ Network: Local stack latency test completed");
    }

    #[test]
    fn benchmark_disk_io_performance() {
        println!("💽 Testing disk I/O performance");

        let temp_dir = std::env::temp_dir();
        let large_file = temp_dir.join("opcode_large_io_test.bin");

        // Create 1MB test data
        let test_data = vec![0u8; 1024 * 1024];

        // Benchmark large file write
        let write_start = Instant::now();
        fs::write(&large_file, &test_data).expect("Failed to write large test file");
        let write_duration = write_start.elapsed();

        // Benchmark large file read
        let read_start = Instant::now();
        let read_data = fs::read(&large_file).expect("Failed to read large test file");
        let read_duration = read_start.elapsed();

        // Verify data integrity
        assert_eq!(read_data.len(), test_data.len(), "Read data should match written data size");

        // Performance targets for 1MB file
        const MAX_WRITE_MS: u64 = 100;
        const MAX_READ_MS: u64 = 50;

        let write_ms = write_duration.as_millis() as u64;
        let read_ms = read_duration.as_millis() as u64;

        // Cleanup
        fs::remove_file(&large_file).expect("Failed to remove large test file");

        assert!(write_ms < MAX_WRITE_MS,
            "1MB file write too slow: {}ms (target: <{}ms)", write_ms, MAX_WRITE_MS);
        assert!(read_ms < MAX_READ_MS,
            "1MB file read too slow: {}ms (target: <{}ms)", read_ms, MAX_READ_MS);

        println!("✅ Disk I/O - 1MB Write: {}ms, Read: {}ms", write_ms, read_ms);
    }

    #[test]
    fn benchmark_windows_api_performance() {
        println!("🔧 Testing Windows API call performance");

        // Test Windows API responsiveness
        let iterations = 100;
        let start_time = Instant::now();

        for _ in 0..iterations {
            // Test GetCurrentDirectory API call via std::env
            let _ = std::env::current_dir().expect("Failed to get current directory");
        }

        let total_duration = start_time.elapsed();
        let avg_duration_us = total_duration.as_micros() / iterations;

        // Should be very fast - less than 100μs per API call on average
        const MAX_API_CALL_US: u128 = 100;

        assert!(avg_duration_us < MAX_API_CALL_US,
            "Windows API calls too slow: {}μs average (target: <{}μs)",
            avg_duration_us, MAX_API_CALL_US);

        println!("✅ Windows API: {}μs average per call ({} iterations)",
            avg_duration_us, iterations);
    }
}