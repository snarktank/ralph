//! Heartbeat monitor for stall detection.
//!
//! This module provides a heartbeat monitoring system that detects stalled
//! agents by tracking time between heartbeat pulses. When pulses stop arriving,
//! warnings and stall detection events are sent through a channel.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use super::TimeoutConfig;

/// Events emitted by the heartbeat monitor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeartbeatEvent {
    /// Warning: heartbeats are being missed but threshold not yet reached.
    /// Contains the number of missed heartbeats.
    Warning(u32),
    /// Stall detected: missed heartbeats threshold has been reached.
    /// Contains the number of missed heartbeats.
    StallDetected(u32),
}

/// Heartbeat monitor for detecting stalled agent execution.
///
/// The monitor tracks the time since the last heartbeat pulse and sends
/// events when heartbeats are missed. It runs a background task that
/// periodically checks elapsed time since the last heartbeat.
///
/// # Example
///
/// ```ignore
/// use std::time::Duration;
/// use ralphmacchio::timeout::{HeartbeatMonitor, TimeoutConfig};
///
/// let config = TimeoutConfig::new()
///     .with_heartbeat_interval(Duration::from_secs(1))
///     .with_missed_heartbeats_threshold(3);
///
/// let (monitor, mut receiver) = HeartbeatMonitor::new(config);
/// monitor.start_monitoring();
///
/// // Send heartbeats to indicate progress
/// monitor.pulse();
///
/// // Check for events
/// if let Some(event) = receiver.try_recv().ok() {
///     match event {
///         HeartbeatEvent::Warning(missed) => println!("Warning: {} missed", missed),
///         HeartbeatEvent::StallDetected(missed) => println!("Stall: {} missed", missed),
///     }
/// }
///
/// // Stop monitoring when done
/// monitor.stop();
/// ```
pub struct HeartbeatMonitor {
    /// Configuration for timeout behavior.
    config: TimeoutConfig,
    /// Timestamp of the last heartbeat pulse.
    last_heartbeat: Arc<Mutex<Instant>>,
    /// Channel sender for heartbeat events.
    sender: mpsc::Sender<HeartbeatEvent>,
    /// Flag to signal the background task to stop.
    stop_flag: Arc<AtomicBool>,
    /// Handle to the background monitoring task.
    task_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl HeartbeatMonitor {
    /// Creates a new heartbeat monitor with the given configuration.
    ///
    /// Returns a tuple of the monitor and a receiver for heartbeat events.
    /// The receiver will receive `HeartbeatEvent::Warning` when heartbeats
    /// start being missed, and `HeartbeatEvent::StallDetected` when the
    /// threshold is reached.
    ///
    /// # Arguments
    ///
    /// * `config` - Timeout configuration including heartbeat interval and threshold
    ///
    /// # Returns
    ///
    /// A tuple of (`HeartbeatMonitor`, `mpsc::Receiver<HeartbeatEvent>`)
    pub fn new(config: TimeoutConfig) -> (Self, mpsc::Receiver<HeartbeatEvent>) {
        let (sender, receiver) = mpsc::channel(16);

        let monitor = Self {
            config,
            last_heartbeat: Arc::new(Mutex::new(Instant::now())),
            sender,
            stop_flag: Arc::new(AtomicBool::new(false)),
            task_handle: Arc::new(Mutex::new(None)),
        };

        (monitor, receiver)
    }

    /// Records a heartbeat pulse, updating the last heartbeat timestamp.
    ///
    /// Call this method periodically to indicate that the agent is still
    /// making progress. If pulses stop arriving, the monitor will detect
    /// the stall and send appropriate events.
    pub async fn pulse(&self) {
        let mut last = self.last_heartbeat.lock().await;
        *last = Instant::now();
    }

    /// Starts the background monitoring task.
    ///
    /// The task periodically checks the elapsed time since the last heartbeat
    /// and sends events when heartbeats are missed:
    ///
    /// - `HeartbeatEvent::Warning` is sent after `missed_heartbeats_threshold - 1`
    ///   consecutive missed heartbeats.
    /// - `HeartbeatEvent::StallDetected` is sent after `missed_heartbeats_threshold`
    ///   consecutive missed heartbeats.
    ///
    /// The task continues running until `stop()` is called.
    pub async fn start_monitoring(&self) {
        // Reset state
        self.stop_flag.store(false, Ordering::SeqCst);
        {
            let mut last = self.last_heartbeat.lock().await;
            *last = Instant::now();
        }

        let config = self.config.clone();
        let last_heartbeat = Arc::clone(&self.last_heartbeat);
        let sender = self.sender.clone();
        let stop_flag = Arc::clone(&self.stop_flag);

        let handle = tokio::spawn(async move {
            let interval = config.heartbeat_interval;
            let threshold = config.missed_heartbeats_threshold;
            let mut last_warning_sent: Option<u32> = None;

            loop {
                if stop_flag.load(Ordering::SeqCst) {
                    break;
                }

                tokio::time::sleep(interval).await;

                if stop_flag.load(Ordering::SeqCst) {
                    break;
                }

                let elapsed = {
                    let last = last_heartbeat.lock().await;
                    last.elapsed()
                };

                // Calculate number of missed heartbeats
                let missed = (elapsed.as_secs_f64() / interval.as_secs_f64()).floor() as u32;

                if missed >= threshold {
                    // Stall detected
                    let _ = sender.send(HeartbeatEvent::StallDetected(missed)).await;
                    // Reset warning tracking after stall
                    last_warning_sent = None;
                } else if missed >= threshold.saturating_sub(1) && missed > 0 {
                    // Warning threshold reached (threshold - 1 missed beats)
                    // Only send warning if we haven't sent one for this level
                    if last_warning_sent != Some(missed) {
                        let _ = sender.send(HeartbeatEvent::Warning(missed)).await;
                        last_warning_sent = Some(missed);
                    }
                } else if missed == 0 {
                    // Reset warning tracking when heartbeats resume
                    last_warning_sent = None;
                }
            }
        });

        let mut task = self.task_handle.lock().await;
        *task = Some(handle);
    }

    /// Stops the background monitoring task.
    ///
    /// This method signals the background task to stop and waits for it
    /// to complete. After calling this method, `start_monitoring()` can
    /// be called again to restart monitoring.
    pub async fn stop(&self) {
        self.stop_flag.store(true, Ordering::SeqCst);

        let handle = {
            let mut task = self.task_handle.lock().await;
            task.take()
        };

        if let Some(handle) = handle {
            let _ = handle.await;
        }
    }

    /// Returns a reference to the timeout configuration.
    pub fn config(&self) -> &TimeoutConfig {
        &self.config
    }

    /// Returns true if the monitoring task is currently running.
    pub async fn is_running(&self) -> bool {
        let task = self.task_handle.lock().await;
        if let Some(handle) = task.as_ref() {
            !handle.is_finished()
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn test_config() -> TimeoutConfig {
        TimeoutConfig::new()
            .with_heartbeat_interval(Duration::from_millis(50))
            .with_missed_heartbeats_threshold(3)
    }

    #[tokio::test]
    async fn test_monitor_creation() {
        let config = test_config();
        let (monitor, _receiver) = HeartbeatMonitor::new(config.clone());

        assert_eq!(
            monitor.config().heartbeat_interval,
            config.heartbeat_interval
        );
        assert_eq!(
            monitor.config().missed_heartbeats_threshold,
            config.missed_heartbeats_threshold
        );
    }

    #[tokio::test]
    async fn test_pulse_updates_timestamp() {
        let config = test_config();
        let (monitor, _receiver) = HeartbeatMonitor::new(config);

        // Record initial timestamp
        let initial = {
            let last = monitor.last_heartbeat.lock().await;
            *last
        };

        // Wait a bit and pulse
        tokio::time::sleep(Duration::from_millis(10)).await;
        monitor.pulse().await;

        // Verify timestamp was updated
        let updated = {
            let last = monitor.last_heartbeat.lock().await;
            *last
        };

        assert!(updated > initial);
    }

    #[tokio::test]
    async fn test_no_events_with_regular_heartbeats() {
        let config = TimeoutConfig::new()
            .with_heartbeat_interval(Duration::from_millis(50))
            .with_missed_heartbeats_threshold(3);

        let (monitor, mut receiver) = HeartbeatMonitor::new(config);
        monitor.start_monitoring().await;

        // Send heartbeats regularly
        for _ in 0..5 {
            tokio::time::sleep(Duration::from_millis(30)).await;
            monitor.pulse().await;
        }

        // Check no events were sent
        let event = receiver.try_recv();
        assert!(event.is_err());

        monitor.stop().await;
    }

    #[tokio::test]
    async fn test_warning_after_threshold_minus_one_missed() {
        let config = TimeoutConfig::new()
            .with_heartbeat_interval(Duration::from_millis(50))
            .with_missed_heartbeats_threshold(3);

        let (monitor, mut receiver) = HeartbeatMonitor::new(config);
        monitor.start_monitoring().await;

        // Wait for threshold - 1 intervals (2 missed)
        // Need to wait for the monitor to check, which happens after one interval
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should have received a warning
        let event = tokio::time::timeout(Duration::from_millis(100), receiver.recv()).await;

        monitor.stop().await;

        assert!(event.is_ok());
        let event = event.unwrap();
        assert!(event.is_some());
        assert!(matches!(event.unwrap(), HeartbeatEvent::Warning(_)));
    }

    #[tokio::test]
    async fn test_stall_detected_after_threshold_missed() {
        let config = TimeoutConfig::new()
            .with_heartbeat_interval(Duration::from_millis(50))
            .with_missed_heartbeats_threshold(3);

        let (monitor, mut receiver) = HeartbeatMonitor::new(config);
        monitor.start_monitoring().await;

        // Wait for threshold intervals (3 missed)
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Stop monitoring to stop event generation
        monitor.stop().await;

        // Collect events that were sent
        let mut events = Vec::new();
        while let Ok(event) = tokio::time::timeout(Duration::from_millis(50), receiver.recv()).await
        {
            if let Some(e) = event {
                events.push(e);
            } else {
                break;
            }
        }

        // Should have received a stall detection
        assert!(events
            .iter()
            .any(|e| matches!(e, HeartbeatEvent::StallDetected(_))));
    }

    #[tokio::test]
    async fn test_stop_terminates_task() {
        let config = test_config();
        let (monitor, _receiver) = HeartbeatMonitor::new(config);

        monitor.start_monitoring().await;
        assert!(monitor.is_running().await);

        monitor.stop().await;

        // Give it a moment to fully stop
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert!(!monitor.is_running().await);
    }

    #[tokio::test]
    async fn test_restart_monitoring() {
        let config = test_config();
        let (monitor, _receiver) = HeartbeatMonitor::new(config);

        // Start and stop
        monitor.start_monitoring().await;
        assert!(monitor.is_running().await);
        monitor.stop().await;

        // Give it a moment to fully stop
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert!(!monitor.is_running().await);

        // Start again
        monitor.start_monitoring().await;
        assert!(monitor.is_running().await);
        monitor.stop().await;
    }

    #[tokio::test]
    async fn test_heartbeat_event_equality() {
        let warning1 = HeartbeatEvent::Warning(2);
        let warning2 = HeartbeatEvent::Warning(2);
        let warning3 = HeartbeatEvent::Warning(3);
        let stall = HeartbeatEvent::StallDetected(3);

        assert_eq!(warning1, warning2);
        assert_ne!(warning1, warning3);
        assert_ne!(warning1, stall);
    }

    #[tokio::test]
    async fn test_heartbeat_event_debug() {
        let warning = HeartbeatEvent::Warning(2);
        let stall = HeartbeatEvent::StallDetected(3);

        let warning_debug = format!("{:?}", warning);
        let stall_debug = format!("{:?}", stall);

        assert!(warning_debug.contains("Warning"));
        assert!(warning_debug.contains("2"));
        assert!(stall_debug.contains("StallDetected"));
        assert!(stall_debug.contains("3"));
    }

    #[tokio::test]
    async fn test_heartbeat_event_clone() {
        let warning = HeartbeatEvent::Warning(2);
        let cloned = warning.clone();
        assert_eq!(warning, cloned);
    }

    #[tokio::test]
    async fn test_config_accessor() {
        let config = TimeoutConfig::new()
            .with_heartbeat_interval(Duration::from_secs(5))
            .with_missed_heartbeats_threshold(10);

        let (monitor, _receiver) = HeartbeatMonitor::new(config);

        assert_eq!(monitor.config().heartbeat_interval, Duration::from_secs(5));
        assert_eq!(monitor.config().missed_heartbeats_threshold, 10);
    }

    #[tokio::test]
    async fn test_pulse_resets_missed_count() {
        let config = TimeoutConfig::new()
            .with_heartbeat_interval(Duration::from_millis(50))
            .with_missed_heartbeats_threshold(3);

        let (monitor, mut receiver) = HeartbeatMonitor::new(config);
        monitor.start_monitoring().await;

        // Wait almost to warning threshold
        tokio::time::sleep(Duration::from_millis(80)).await;

        // Pulse to reset
        monitor.pulse().await;

        // Wait a bit more (should not reach threshold from reset point)
        tokio::time::sleep(Duration::from_millis(80)).await;

        // Pulse again
        monitor.pulse().await;

        // No events should have been sent
        let event = receiver.try_recv();
        assert!(event.is_err());

        monitor.stop().await;
    }

    #[tokio::test]
    async fn test_warning_before_stall() {
        let config = TimeoutConfig::new()
            .with_heartbeat_interval(Duration::from_millis(30))
            .with_missed_heartbeats_threshold(3);

        let (monitor, mut receiver) = HeartbeatMonitor::new(config);
        monitor.start_monitoring().await;

        // Wait long enough for both warning and stall
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Stop monitoring to stop event generation
        monitor.stop().await;

        // Collect events that were sent
        let mut events = Vec::new();
        while let Ok(event) = tokio::time::timeout(Duration::from_millis(50), receiver.recv()).await
        {
            if let Some(e) = event {
                events.push(e);
            } else {
                break;
            }
        }

        // Should have warning(s) and stall detection
        let has_warning = events
            .iter()
            .any(|e| matches!(e, HeartbeatEvent::Warning(_)));
        let has_stall = events
            .iter()
            .any(|e| matches!(e, HeartbeatEvent::StallDetected(_)));

        assert!(has_warning, "Expected warning event");
        assert!(has_stall, "Expected stall detection event");
    }

    #[tokio::test]
    async fn test_is_running_before_start() {
        let config = test_config();
        let (monitor, _receiver) = HeartbeatMonitor::new(config);

        assert!(!monitor.is_running().await);
    }
}
