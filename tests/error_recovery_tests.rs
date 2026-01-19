//! Integration tests for the error recovery system.
//!
//! This module tests the complete error recovery system end-to-end,
//! including checkpoint save/load, error classification, retry backoff,
//! pause controller state transitions, and timeout wrapper behavior.

use std::time::Duration;

use ralphmacchio::checkpoint::{Checkpoint, CheckpointManager, PauseReason, StoryCheckpoint};
use ralphmacchio::error::{
    ClassifiedError, ErrorCategory, ErrorDetector, FatalReason, RecoveryHint, TimeoutReason,
    TransientReason, UsageLimitReason,
};
use ralphmacchio::pause::{PauseController, PauseState, RetryStrategy};
use ralphmacchio::timeout::{HeartbeatEvent, HeartbeatMonitor, TimeoutConfig};

// ============================================================================
// Checkpoint Save and Load Roundtrip Tests
// ============================================================================

#[test]
fn test_checkpoint_save_load_roundtrip_with_story() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let manager = CheckpointManager::new(temp_dir.path()).expect("Failed to create manager");

    let original = Checkpoint::new(
        Some(StoryCheckpoint::new("US-001", 3, 10)),
        PauseReason::RateLimited,
        vec!["src/lib.rs".to_string(), "src/main.rs".to_string()],
    );

    manager.save(&original).expect("Failed to save checkpoint");
    let loaded = manager
        .load()
        .expect("Failed to load checkpoint")
        .expect("Checkpoint should exist");

    assert_eq!(loaded.version, original.version);
    assert_eq!(loaded.pause_reason, original.pause_reason);
    assert_eq!(loaded.current_story, original.current_story);
    assert_eq!(loaded.uncommitted_files, original.uncommitted_files);

    let story = loaded.current_story.as_ref().unwrap();
    assert_eq!(story.story_id, "US-001");
    assert_eq!(story.iteration, 3);
    assert_eq!(story.max_iterations, 10);
}

#[test]
fn test_checkpoint_save_load_roundtrip_without_story() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let manager = CheckpointManager::new(temp_dir.path()).expect("Failed to create manager");

    let original = Checkpoint::new(None, PauseReason::UserRequested, vec![]);

    manager.save(&original).expect("Failed to save checkpoint");
    let loaded = manager
        .load()
        .expect("Failed to load checkpoint")
        .expect("Checkpoint should exist");

    assert!(loaded.current_story.is_none());
    assert_eq!(loaded.pause_reason, PauseReason::UserRequested);
    assert!(loaded.uncommitted_files.is_empty());
}

#[test]
fn test_checkpoint_roundtrip_all_pause_reasons() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let manager = CheckpointManager::new(temp_dir.path()).expect("Failed to create manager");

    let pause_reasons = vec![
        PauseReason::UsageLimitExceeded,
        PauseReason::RateLimited,
        PauseReason::UserRequested,
        PauseReason::Timeout,
        PauseReason::Error("Connection failed".to_string()),
    ];

    for reason in pause_reasons {
        let checkpoint = Checkpoint::new(None, reason.clone(), vec![]);
        manager.save(&checkpoint).expect("Failed to save");
        let loaded = manager.load().expect("Failed to load").unwrap();
        assert_eq!(loaded.pause_reason, reason);
        manager.clear().expect("Failed to clear");
    }
}

#[test]
fn test_checkpoint_verify_validates_story() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let manager = CheckpointManager::new(temp_dir.path()).expect("Failed to create manager");

    // Valid checkpoint
    let valid = Checkpoint::new(
        Some(StoryCheckpoint::new("US-001", 5, 10)),
        PauseReason::RateLimited,
        vec![],
    );
    assert!(manager.verify(&valid).is_ok());

    // Invalid: empty story_id
    let invalid_empty_id = Checkpoint::new(
        Some(StoryCheckpoint::new("", 1, 5)),
        PauseReason::RateLimited,
        vec![],
    );
    assert!(manager.verify(&invalid_empty_id).is_err());

    // Invalid: iteration exceeds max
    let invalid_iteration = Checkpoint::new(
        Some(StoryCheckpoint::new("US-001", 15, 10)),
        PauseReason::RateLimited,
        vec![],
    );
    assert!(manager.verify(&invalid_iteration).is_err());
}

#[test]
fn test_checkpoint_overwrite_and_clear() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let manager = CheckpointManager::new(temp_dir.path()).expect("Failed to create manager");

    // Save first checkpoint
    let first = Checkpoint::new(
        Some(StoryCheckpoint::new("US-001", 1, 5)),
        PauseReason::RateLimited,
        vec!["file1.rs".to_string()],
    );
    manager.save(&first).expect("Failed to save first");
    assert!(manager.exists());

    // Overwrite with second checkpoint
    let second = Checkpoint::new(
        Some(StoryCheckpoint::new("US-002", 3, 8)),
        PauseReason::Timeout,
        vec!["file2.rs".to_string()],
    );
    manager.save(&second).expect("Failed to save second");

    let loaded = manager.load().expect("Failed to load").unwrap();
    assert_eq!(loaded.current_story.as_ref().unwrap().story_id, "US-002");
    assert_eq!(loaded.pause_reason, PauseReason::Timeout);

    // Clear checkpoint
    manager.clear().expect("Failed to clear");
    assert!(!manager.exists());
    assert!(manager.load().expect("Failed to load").is_none());
}

// ============================================================================
// Error Classification Tests for Known Patterns
// ============================================================================

#[test]
fn test_error_classification_rate_limit_patterns() {
    let detector = ErrorDetector::new();

    let rate_limit_texts = vec![
        "Error 429: Too Many Requests",
        "rate limit exceeded",
        "Rate-Limit Error",
        "Too many requests, please try again later",
    ];

    for text in rate_limit_texts {
        let result = detector.classify_error(text);
        assert!(result.is_some(), "Should detect rate limit in: {}", text);
        let error = result.unwrap();
        assert!(
            matches!(
                error.category,
                ErrorCategory::UsageLimit(UsageLimitReason::RateLimited)
            ),
            "Wrong category for: {}",
            text
        );
    }
}

#[test]
fn test_error_classification_authentication_patterns() {
    let detector = ErrorDetector::new();

    let auth_texts = vec![
        "401 Unauthorized",
        "Authentication failed",
        "Invalid API key",
        "Invalid token provided",
    ];

    for text in auth_texts {
        let result = detector.classify_error(text);
        assert!(result.is_some(), "Should detect auth error in: {}", text);
        let error = result.unwrap();
        assert!(
            matches!(
                error.category,
                ErrorCategory::Fatal(FatalReason::AuthenticationFailed)
            ),
            "Wrong category for: {}",
            text
        );
    }
}

#[test]
fn test_error_classification_timeout_patterns() {
    let detector = ErrorDetector::new();

    let timeout_texts = vec![
        "Request timeout",
        "request timed out",
        "Operation timeout occurred",
        "Deadline exceeded",
    ];

    for text in timeout_texts {
        let result = detector.classify_error(text);
        assert!(result.is_some(), "Should detect timeout in: {}", text);
        let error = result.unwrap();
        assert!(
            matches!(error.category, ErrorCategory::Timeout(_)),
            "Wrong category for: {}",
            text
        );
    }
}

#[test]
fn test_error_classification_transient_patterns() {
    let detector = ErrorDetector::new();

    let transient_texts = vec![
        "Connection refused",
        "Network error occurred",
        "503 Service Unavailable",
        "500 Internal Server Error",
    ];

    for text in transient_texts {
        let result = detector.classify_error(text);
        assert!(
            result.is_some(),
            "Should detect transient error in: {}",
            text
        );
        let error = result.unwrap();
        assert!(
            matches!(error.category, ErrorCategory::Transient(_)),
            "Wrong category for: {}",
            text
        );
    }
}

#[test]
fn test_error_classification_exit_codes() {
    let detector = ErrorDetector::new();

    // Exit code 124 = timeout
    let result = detector.classify_exit_code(124);
    assert!(result.is_some());
    assert!(matches!(
        result.unwrap().category,
        ErrorCategory::Timeout(TimeoutReason::ProcessTimeout)
    ));

    // Exit code 137 = SIGKILL
    let result = detector.classify_exit_code(137);
    assert!(result.is_some());
    let error = result.unwrap();
    assert!(matches!(
        error.category,
        ErrorCategory::Timeout(TimeoutReason::ProcessTimeout)
    ));
    assert_eq!(error.context.get("signal"), Some(&"SIGKILL".to_string()));

    // Exit code 143 = SIGTERM
    let result = detector.classify_exit_code(143);
    assert!(result.is_some());
    let error = result.unwrap();
    assert_eq!(error.context.get("signal"), Some(&"SIGTERM".to_string()));

    // Unknown exit codes return None
    assert!(detector.classify_exit_code(0).is_none());
    assert!(detector.classify_exit_code(1).is_none());
}

#[test]
fn test_error_classification_combined() {
    let detector = ErrorDetector::new();

    // Text classification takes priority over exit code
    let result = detector.classify("rate limit exceeded", Some(124));
    assert!(result.is_some());
    assert!(matches!(
        result.unwrap().category,
        ErrorCategory::UsageLimit(UsageLimitReason::RateLimited)
    ));

    // Falls back to exit code when text doesn't match
    let result = detector.classify("Everything is fine", Some(124));
    assert!(result.is_some());
    assert!(matches!(
        result.unwrap().category,
        ErrorCategory::Timeout(TimeoutReason::ProcessTimeout)
    ));

    // No match when both don't match
    let result = detector.classify("Everything is fine", Some(0));
    assert!(result.is_none());
}

#[test]
fn test_classified_error_utilities() {
    let transient = ClassifiedError::new(
        ErrorCategory::Transient(TransientReason::NetworkError),
        "Network error",
        RecoveryHint::RetryNow,
    );
    assert!(transient.is_transient());
    assert!(transient.should_retry());
    assert!(!transient.is_fatal());

    let fatal = ClassifiedError::new(
        ErrorCategory::Fatal(FatalReason::AuthenticationFailed),
        "Auth failed",
        RecoveryHint::StopExecution,
    );
    assert!(fatal.is_fatal());
    assert!(!fatal.should_retry());

    let usage_limit = ClassifiedError::new(
        ErrorCategory::UsageLimit(UsageLimitReason::RateLimited),
        "Rate limited",
        RecoveryHint::RetryAfter(Duration::from_secs(60)),
    );
    assert!(usage_limit.is_usage_limit());
    assert!(usage_limit.should_retry());

    let timeout = ClassifiedError::new(
        ErrorCategory::Timeout(TimeoutReason::RequestTimeout),
        "Timeout",
        RecoveryHint::RetryNow,
    );
    assert!(timeout.is_timeout());
}

// ============================================================================
// Retry Backoff Calculation Tests
// ============================================================================

#[test]
fn test_retry_backoff_exponential_growth() {
    let strategy = RetryStrategy::new(
        Duration::from_secs(1),
        Duration::from_secs(60),
        5,
        0, // No jitter for predictable testing
    );

    // Verify exponential growth: base_delay * 2^(attempt-1)
    assert_eq!(strategy.calculate_delay(1), Duration::from_secs(1));
    assert_eq!(strategy.calculate_delay(2), Duration::from_secs(2));
    assert_eq!(strategy.calculate_delay(3), Duration::from_secs(4));
    assert_eq!(strategy.calculate_delay(4), Duration::from_secs(8));
    assert_eq!(strategy.calculate_delay(5), Duration::from_secs(16));
}

#[test]
fn test_retry_backoff_respects_max_delay() {
    let strategy = RetryStrategy::new(
        Duration::from_secs(10),
        Duration::from_secs(30),
        10,
        0, // No jitter
    );

    // Attempt 3: 10 * 2^2 = 40, but capped to 30
    assert_eq!(strategy.calculate_delay(3), Duration::from_secs(30));
    // Higher attempts should still be capped
    assert_eq!(strategy.calculate_delay(5), Duration::from_secs(30));
    assert_eq!(strategy.calculate_delay(10), Duration::from_secs(30));
}

#[test]
fn test_retry_backoff_with_jitter() {
    let strategy = RetryStrategy::new(
        Duration::from_secs(10),
        Duration::from_secs(60),
        5,
        10, // 10% jitter
    );

    // With 10% jitter on 10 seconds = ±1 second range
    let delay = strategy.calculate_delay(1);
    assert!(delay >= Duration::from_millis(9000));
    assert!(delay <= Duration::from_millis(11000));
}

#[test]
fn test_retry_should_retry_transient_only() {
    let strategy = RetryStrategy::default();

    // Transient errors should retry
    let transient = ErrorCategory::Transient(TransientReason::NetworkError);
    assert!(strategy.should_retry(1, &transient));
    assert!(strategy.should_retry(4, &transient));

    // Non-transient errors should not retry
    let usage_limit = ErrorCategory::UsageLimit(UsageLimitReason::RateLimited);
    assert!(!strategy.should_retry(1, &usage_limit));

    let fatal = ErrorCategory::Fatal(FatalReason::AuthenticationFailed);
    assert!(!strategy.should_retry(1, &fatal));

    let timeout = ErrorCategory::Timeout(TimeoutReason::RequestTimeout);
    assert!(!strategy.should_retry(1, &timeout));
}

#[test]
fn test_retry_respects_max_attempts() {
    let strategy = RetryStrategy::new(Duration::from_secs(1), Duration::from_secs(60), 3, 0);
    let transient = ErrorCategory::Transient(TransientReason::NetworkError);

    // Attempts 1, 2 should retry (less than max_attempts of 3)
    assert!(strategy.should_retry(1, &transient));
    assert!(strategy.should_retry(2, &transient));

    // Attempt 3 and beyond should not retry
    assert!(!strategy.should_retry(3, &transient));
    assert!(!strategy.should_retry(4, &transient));
}

#[test]
fn test_retry_strategy_defaults() {
    let strategy = RetryStrategy::default();

    assert_eq!(strategy.base_delay, Duration::from_secs(1));
    assert_eq!(strategy.max_delay, Duration::from_secs(60));
    assert_eq!(strategy.max_attempts, 5);
    assert_eq!(strategy.jitter_percent, 10);
}

// ============================================================================
// Pause Controller State Transition Tests
// ============================================================================

#[test]
fn test_pause_controller_initial_state() {
    let controller = PauseController::new();

    assert_eq!(controller.state(), PauseState::Running);
    assert!(controller.is_running());
    assert!(!controller.is_paused());
    assert!(!controller.is_pause_requested());
}

#[test]
fn test_pause_controller_full_cycle() {
    let controller = PauseController::new();

    // Running -> PauseRequested
    assert!(controller.request_pause());
    assert_eq!(controller.state(), PauseState::PauseRequested);
    assert!(controller.is_pause_requested());

    // PauseRequested -> Paused
    assert!(controller.execute_pause());
    assert_eq!(controller.state(), PauseState::Paused);
    assert!(controller.is_paused());

    // Paused -> Running
    assert!(controller.resume());
    assert_eq!(controller.state(), PauseState::Running);
    assert!(controller.is_running());
}

#[test]
fn test_pause_controller_invalid_transitions() {
    let controller = PauseController::new();

    // Can't execute_pause from Running
    assert!(!controller.execute_pause());
    assert_eq!(controller.state(), PauseState::Running);

    // Can't resume from Running
    assert!(!controller.resume());
    assert_eq!(controller.state(), PauseState::Running);

    // Request pause
    controller.request_pause();

    // Can't request_pause from PauseRequested
    assert!(!controller.request_pause());

    // Can't resume from PauseRequested
    assert!(!controller.resume());

    // Execute pause
    controller.execute_pause();

    // Can't request_pause from Paused
    assert!(!controller.request_pause());

    // Can't execute_pause from Paused
    assert!(!controller.execute_pause());
}

#[test]
fn test_pause_controller_multiple_cycles() {
    let controller = PauseController::new();

    for _ in 0..3 {
        assert!(controller.request_pause());
        assert!(controller.execute_pause());
        assert!(controller.resume());
        assert_eq!(controller.state(), PauseState::Running);
    }
}

#[test]
fn test_pause_controller_clone_shares_state() {
    let controller1 = PauseController::new();
    let controller2 = controller1.clone();

    // Changes through one clone affect the other
    controller1.request_pause();
    assert!(controller2.is_pause_requested());

    controller2.execute_pause();
    assert!(controller1.is_paused());

    controller1.resume();
    assert!(controller2.is_running());
}

#[test]
fn test_pause_controller_thread_safety() {
    use std::thread;

    let controller = PauseController::new();
    let controller_clone = controller.clone();

    let handle = thread::spawn(move || {
        controller_clone.request_pause();
        controller_clone.is_pause_requested()
    });

    let result = handle.join().unwrap();
    assert!(result);
    assert!(controller.is_pause_requested());
}

// ============================================================================
// Timeout Wrapper Behavior Tests
// ============================================================================

#[test]
fn test_timeout_config_defaults() {
    let config = TimeoutConfig::default();

    assert_eq!(config.agent_timeout, Duration::from_secs(600));
    assert_eq!(config.iteration_timeout, Duration::from_secs(900));
    assert_eq!(config.heartbeat_interval, Duration::from_secs(30));
    assert_eq!(config.missed_heartbeats_threshold, 3);
}

#[test]
fn test_timeout_config_builder() {
    let config = TimeoutConfig::new()
        .with_agent_timeout(Duration::from_secs(300))
        .with_iteration_timeout(Duration::from_secs(600))
        .with_heartbeat_interval(Duration::from_secs(15))
        .with_missed_heartbeats_threshold(5);

    assert_eq!(config.agent_timeout, Duration::from_secs(300));
    assert_eq!(config.iteration_timeout, Duration::from_secs(600));
    assert_eq!(config.heartbeat_interval, Duration::from_secs(15));
    assert_eq!(config.missed_heartbeats_threshold, 5);
}

#[tokio::test]
async fn test_heartbeat_monitor_no_events_with_pulses() {
    let config = TimeoutConfig::new()
        .with_heartbeat_interval(Duration::from_millis(50))
        .with_missed_heartbeats_threshold(3);

    let (monitor, mut receiver) = HeartbeatMonitor::new(config);
    monitor.start_monitoring().await;

    // Send regular heartbeats
    for _ in 0..5 {
        tokio::time::sleep(Duration::from_millis(30)).await;
        monitor.pulse().await;
    }

    // Should have no events
    let event = receiver.try_recv();
    assert!(
        event.is_err(),
        "Should not have any events with regular pulses"
    );

    monitor.stop().await;
}

#[tokio::test]
async fn test_heartbeat_monitor_warning_on_missed_beats() {
    let config = TimeoutConfig::new()
        .with_heartbeat_interval(Duration::from_millis(50))
        .with_missed_heartbeats_threshold(3);

    let (monitor, mut receiver) = HeartbeatMonitor::new(config);
    monitor.start_monitoring().await;

    // Wait for threshold - 1 intervals to trigger warning
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Should receive a warning
    let event = tokio::time::timeout(Duration::from_millis(100), receiver.recv()).await;
    monitor.stop().await;

    assert!(event.is_ok(), "Should receive an event");
    let event = event.unwrap();
    assert!(event.is_some(), "Event should be present");
    assert!(
        matches!(event.unwrap(), HeartbeatEvent::Warning(_)),
        "Should be a warning event"
    );
}

#[tokio::test]
async fn test_heartbeat_monitor_stall_detection() {
    let config = TimeoutConfig::new()
        .with_heartbeat_interval(Duration::from_millis(50))
        .with_missed_heartbeats_threshold(3);

    let (monitor, mut receiver) = HeartbeatMonitor::new(config);
    monitor.start_monitoring().await;

    // Wait for threshold intervals to trigger stall detection
    tokio::time::sleep(Duration::from_millis(200)).await;
    monitor.stop().await;

    // Collect events
    let mut events = Vec::new();
    while let Ok(event) = tokio::time::timeout(Duration::from_millis(50), receiver.recv()).await {
        if let Some(e) = event {
            events.push(e);
        } else {
            break;
        }
    }

    // Should have stall detection
    assert!(
        events
            .iter()
            .any(|e| matches!(e, HeartbeatEvent::StallDetected(_))),
        "Should have stall detection event"
    );
}

#[tokio::test]
async fn test_heartbeat_monitor_start_stop() {
    let config = TimeoutConfig::new()
        .with_heartbeat_interval(Duration::from_millis(50))
        .with_missed_heartbeats_threshold(3);

    let (monitor, _receiver) = HeartbeatMonitor::new(config);

    assert!(!monitor.is_running().await);

    monitor.start_monitoring().await;
    assert!(monitor.is_running().await);

    monitor.stop().await;
    tokio::time::sleep(Duration::from_millis(10)).await;
    assert!(!monitor.is_running().await);
}

#[tokio::test]
async fn test_heartbeat_monitor_restart() {
    let config = TimeoutConfig::new()
        .with_heartbeat_interval(Duration::from_millis(50))
        .with_missed_heartbeats_threshold(3);

    let (monitor, _receiver) = HeartbeatMonitor::new(config);

    // First cycle
    monitor.start_monitoring().await;
    assert!(monitor.is_running().await);
    monitor.stop().await;
    tokio::time::sleep(Duration::from_millis(10)).await;
    assert!(!monitor.is_running().await);

    // Second cycle
    monitor.start_monitoring().await;
    assert!(monitor.is_running().await);
    monitor.stop().await;
}

#[tokio::test]
async fn test_heartbeat_pulse_resets_timer() {
    let config = TimeoutConfig::new()
        .with_heartbeat_interval(Duration::from_millis(50))
        .with_missed_heartbeats_threshold(3);

    let (monitor, mut receiver) = HeartbeatMonitor::new(config);
    monitor.start_monitoring().await;

    // Wait almost to warning threshold
    tokio::time::sleep(Duration::from_millis(80)).await;
    monitor.pulse().await;

    // Wait a bit more (should not reach threshold from reset point)
    tokio::time::sleep(Duration::from_millis(80)).await;
    monitor.pulse().await;

    // No events should have been sent
    let event = receiver.try_recv();
    assert!(
        event.is_err(),
        "No events should be sent when pulses reset timer"
    );

    monitor.stop().await;
}

// ============================================================================
// Integration: End-to-End Error Recovery Flow Tests
// ============================================================================

#[test]
fn test_error_recovery_flow_checkpoint_on_error() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let manager = CheckpointManager::new(temp_dir.path()).expect("Failed to create manager");
    let detector = ErrorDetector::new();

    // Simulate detecting a rate limit error
    let error_output = "Error 429: Rate limit exceeded";
    let classified = detector
        .classify_error(error_output)
        .expect("Should classify");

    // Create checkpoint based on error
    let pause_reason = match &classified.category {
        ErrorCategory::UsageLimit(UsageLimitReason::RateLimited) => PauseReason::RateLimited,
        ErrorCategory::Timeout(_) => PauseReason::Timeout,
        _ => PauseReason::Error(classified.message.clone()),
    };

    let checkpoint = Checkpoint::new(
        Some(StoryCheckpoint::new("US-001", 3, 10)),
        pause_reason,
        vec!["src/modified.rs".to_string()],
    );

    manager.save(&checkpoint).expect("Failed to save");
    assert!(manager.exists());

    // Load and verify
    let loaded = manager.load().expect("Failed to load").unwrap();
    assert_eq!(loaded.pause_reason, PauseReason::RateLimited);
}

#[test]
fn test_error_recovery_flow_retry_decision() {
    let detector = ErrorDetector::new();
    let strategy = RetryStrategy::default();

    // Transient error should allow retry
    let transient_error = detector
        .classify_error("Connection refused")
        .expect("Should classify");
    assert!(strategy.should_retry(1, &transient_error.category));

    // Calculate backoff delay
    let delay = strategy.calculate_delay(1);
    assert!(delay >= Duration::from_millis(900)); // ~1s with jitter

    // Fatal error should not retry
    let fatal_error = detector
        .classify_error("401 Unauthorized")
        .expect("Should classify");
    assert!(!strategy.should_retry(1, &fatal_error.category));
}

#[tokio::test]
async fn test_error_recovery_flow_pause_on_stall() {
    let config = TimeoutConfig::new()
        .with_heartbeat_interval(Duration::from_millis(30))
        .with_missed_heartbeats_threshold(2);

    let (monitor, mut receiver) = HeartbeatMonitor::new(config);
    let pause_controller = PauseController::new();

    monitor.start_monitoring().await;

    // Simulate stall (no pulses)
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Check for stall event
    let mut stall_detected = false;
    while let Ok(event) = tokio::time::timeout(Duration::from_millis(50), receiver.recv()).await {
        if let Some(HeartbeatEvent::StallDetected(_)) = event {
            stall_detected = true;
            break;
        }
    }

    if stall_detected {
        // Request pause in response to stall
        pause_controller.request_pause();
        assert!(pause_controller.is_pause_requested());
    }

    monitor.stop().await;
    assert!(stall_detected, "Should detect stall");
}

#[test]
fn test_error_recovery_flow_resume_from_checkpoint() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let manager = CheckpointManager::new(temp_dir.path()).expect("Failed to create manager");

    // Save checkpoint before interruption
    let checkpoint = Checkpoint::new(
        Some(StoryCheckpoint::new("US-005", 7, 15)),
        PauseReason::Timeout,
        vec!["src/feature.rs".to_string()],
    );
    manager.save(&checkpoint).expect("Failed to save");

    // Simulate restart and load checkpoint
    let loaded = manager.load().expect("Failed to load").unwrap();
    manager.verify(&loaded).expect("Checkpoint should be valid");

    // Verify we can resume from the right point
    let story = loaded.current_story.as_ref().unwrap();
    assert_eq!(story.story_id, "US-005");
    assert_eq!(story.iteration, 7);
    assert_eq!(story.max_iterations, 15);

    // Clear checkpoint after successful resume
    manager.clear().expect("Failed to clear");
    assert!(!manager.exists());
}
