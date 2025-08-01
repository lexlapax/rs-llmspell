// ABOUTME: Replay scheduler for delayed and recurring hook replay operations
// ABOUTME: Supports cron-like scheduling and one-time delayed execution

use super::{ReplayRequest, ReplayResponse};
use anyhow::Result;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::interval;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Replay schedule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplaySchedule {
    /// Execute once after a delay
    Once { delay: Duration },
    /// Execute at a specific time
    At { time: DateTime<Utc> },
    /// Execute repeatedly at intervals
    Interval {
        initial_delay: Duration,
        interval: Duration,
        max_executions: Option<usize>,
    },
    /// Cron-like schedule (simplified)
    Cron { expression: String },
}

/// Scheduled replay entry
#[derive(Debug, Clone)]
pub struct ScheduledReplay {
    /// Unique ID for this scheduled replay
    pub id: Uuid,
    /// The replay request to execute
    pub request: ReplayRequest,
    /// Schedule configuration
    pub schedule: ReplaySchedule,
    /// When to execute next
    pub next_execution: DateTime<Utc>,
    /// Number of times executed
    pub execution_count: usize,
    /// Whether this schedule is active
    pub active: bool,
    /// Creation time
    pub created_at: DateTime<Utc>,
    /// Last execution time
    pub last_execution: Option<DateTime<Utc>>,
    /// Last execution result
    pub last_result: Option<Result<ReplayResponse, String>>,
}

impl ScheduledReplay {
    /// Create a new scheduled replay
    fn new(request: ReplayRequest, schedule: ReplaySchedule) -> Self {
        let next_execution = Self::calculate_next_execution(&schedule, None);

        Self {
            id: Uuid::new_v4(),
            request,
            schedule,
            next_execution,
            execution_count: 0,
            active: true,
            created_at: Utc::now(),
            last_execution: None,
            last_result: None,
        }
    }

    /// Calculate next execution time
    fn calculate_next_execution(
        schedule: &ReplaySchedule,
        last_execution: Option<DateTime<Utc>>,
    ) -> DateTime<Utc> {
        let now = Utc::now();

        match schedule {
            ReplaySchedule::Once { delay } => {
                if last_execution.is_some() {
                    // Already executed once
                    now + ChronoDuration::days(365 * 100) // Far future
                } else {
                    now + ChronoDuration::from_std(*delay).unwrap_or_default()
                }
            }
            ReplaySchedule::At { time } => {
                if last_execution.is_some() {
                    // Already executed at specified time
                    now + ChronoDuration::days(365 * 100) // Far future
                } else {
                    *time
                }
            }
            ReplaySchedule::Interval {
                initial_delay,
                interval,
                ..
            } => {
                if let Some(last) = last_execution {
                    last + ChronoDuration::from_std(*interval).unwrap_or_default()
                } else {
                    now + ChronoDuration::from_std(*initial_delay).unwrap_or_default()
                }
            }
            ReplaySchedule::Cron { expression } => {
                // Simplified cron parsing - just support basic patterns
                Self::parse_cron_expression(expression, now)
                    .unwrap_or(now + ChronoDuration::hours(1))
            }
        }
    }

    /// Parse a simplified cron expression
    fn parse_cron_expression(expression: &str, after: DateTime<Utc>) -> Option<DateTime<Utc>> {
        // Very basic implementation - just support hourly/daily/weekly
        match expression {
            "hourly" => Some(after + ChronoDuration::hours(1)),
            "daily" => Some(after + ChronoDuration::days(1)),
            "weekly" => Some(after + ChronoDuration::weeks(1)),
            _ => {
                warn!("Unsupported cron expression: {}", expression);
                None
            }
        }
    }

    /// Check if should continue scheduling
    fn should_continue(&self) -> bool {
        if !self.active {
            return false;
        }

        match &self.schedule {
            ReplaySchedule::Once { .. } | ReplaySchedule::At { .. } => self.execution_count == 0,
            ReplaySchedule::Interval { max_executions, .. } => {
                if let Some(max) = max_executions {
                    self.execution_count < *max
                } else {
                    true
                }
            }
            ReplaySchedule::Cron { .. } => true,
        }
    }
}

/// Priority queue ordering for scheduled replays
impl Ord for ScheduledReplay {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse order for min-heap (earliest time first)
        other.next_execution.cmp(&self.next_execution)
    }
}

impl PartialOrd for ScheduledReplay {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ScheduledReplay {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for ScheduledReplay {}

/// Replay scheduler
pub struct ReplayScheduler {
    /// Scheduled replays
    scheduled: Arc<RwLock<HashMap<Uuid, ScheduledReplay>>>,
    /// Priority queue for next executions
    queue: Arc<RwLock<BinaryHeap<ScheduledReplay>>>,
    /// Channel for scheduler commands
    command_tx: mpsc::Sender<SchedulerCommand>,
    /// Running flag
    running: Arc<RwLock<bool>>,
}

/// Commands for the scheduler
enum SchedulerCommand {
    Schedule(ReplayRequest, ReplaySchedule, mpsc::Sender<Result<Uuid>>),
    Cancel(Uuid, mpsc::Sender<Result<()>>),
    Stop,
}

impl ReplayScheduler {
    /// Create a new replay scheduler
    pub fn new() -> Self {
        let (command_tx, command_rx) = mpsc::channel(100);

        let scheduler = Self {
            scheduled: Arc::new(RwLock::new(HashMap::new())),
            queue: Arc::new(RwLock::new(BinaryHeap::new())),
            command_tx,
            running: Arc::new(RwLock::new(false)),
        };

        // Start background task
        let scheduled = scheduler.scheduled.clone();
        let queue = scheduler.queue.clone();
        let running = scheduler.running.clone();

        tokio::spawn(async move {
            Self::run_scheduler(scheduled, queue, running, command_rx).await;
        });

        scheduler
    }

    /// Schedule a replay
    pub async fn schedule(&self, request: ReplayRequest, schedule: ReplaySchedule) -> Result<Uuid> {
        let (response_tx, mut response_rx) = mpsc::channel(1);

        self.command_tx
            .send(SchedulerCommand::Schedule(request, schedule, response_tx))
            .await?;

        response_rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("Scheduler response channel closed"))?
    }

    /// Cancel a scheduled replay
    pub async fn cancel(&self, id: Uuid) -> Result<()> {
        let (response_tx, mut response_rx) = mpsc::channel(1);

        self.command_tx
            .send(SchedulerCommand::Cancel(id, response_tx))
            .await?;

        response_rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("Scheduler response channel closed"))?
    }

    /// Get all scheduled replays
    pub fn get_scheduled(&self) -> Vec<ScheduledReplay> {
        self.scheduled.read().values().cloned().collect()
    }

    /// Get a specific scheduled replay
    pub fn get_scheduled_replay(&self, id: Uuid) -> Option<ScheduledReplay> {
        self.scheduled.read().get(&id).cloned()
    }

    /// Stop the scheduler
    pub async fn stop(&self) -> Result<()> {
        self.command_tx.send(SchedulerCommand::Stop).await?;
        Ok(())
    }

    /// Run the scheduler background task
    async fn run_scheduler(
        scheduled: Arc<RwLock<HashMap<Uuid, ScheduledReplay>>>,
        queue: Arc<RwLock<BinaryHeap<ScheduledReplay>>>,
        running: Arc<RwLock<bool>>,
        mut command_rx: mpsc::Receiver<SchedulerCommand>,
    ) {
        *running.write() = true;
        let mut ticker = interval(Duration::from_secs(1));

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    // Check for replays to execute
                    Self::check_and_execute_replays(&scheduled, &queue).await;
                }

                Some(command) = command_rx.recv() => {
                    match command {
                        SchedulerCommand::Schedule(request, schedule, response_tx) => {
                            let result = Self::handle_schedule(
                                &scheduled,
                                &queue,
                                request,
                                schedule,
                            );
                            let _ = response_tx.send(result).await;
                        }

                        SchedulerCommand::Cancel(id, response_tx) => {
                            let result = Self::handle_cancel(&scheduled, &queue, id);
                            let _ = response_tx.send(result).await;
                        }

                        SchedulerCommand::Stop => {
                            info!("Scheduler stopping");
                            break;
                        }
                    }
                }
            }
        }

        *running.write() = false;
    }

    /// Handle schedule command
    fn handle_schedule(
        scheduled: &Arc<RwLock<HashMap<Uuid, ScheduledReplay>>>,
        queue: &Arc<RwLock<BinaryHeap<ScheduledReplay>>>,
        request: ReplayRequest,
        schedule: ReplaySchedule,
    ) -> Result<Uuid> {
        let replay = ScheduledReplay::new(request, schedule);
        let id = replay.id;

        scheduled.write().insert(id, replay.clone());
        queue.write().push(replay);

        info!("Scheduled replay {}", id);
        Ok(id)
    }

    /// Handle cancel command
    fn handle_cancel(
        scheduled: &Arc<RwLock<HashMap<Uuid, ScheduledReplay>>>,
        _queue: &Arc<RwLock<BinaryHeap<ScheduledReplay>>>,
        id: Uuid,
    ) -> Result<()> {
        if let Some(mut replay) = scheduled.write().remove(&id) {
            replay.active = false;
            info!("Cancelled scheduled replay {}", id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Scheduled replay not found"))
        }
    }

    /// Check and execute due replays
    async fn check_and_execute_replays(
        scheduled: &Arc<RwLock<HashMap<Uuid, ScheduledReplay>>>,
        queue: &Arc<RwLock<BinaryHeap<ScheduledReplay>>>,
    ) {
        let now = Utc::now();
        let mut to_execute = Vec::new();

        // Find replays due for execution
        {
            let mut queue_lock = queue.write();
            while let Some(replay) = queue_lock.peek() {
                if replay.next_execution <= now && replay.active {
                    if let Some(replay) = queue_lock.pop() {
                        to_execute.push(replay);
                    }
                } else {
                    break;
                }
            }
        }

        // Execute replays
        for mut replay in to_execute {
            if !replay.should_continue() {
                scheduled.write().remove(&replay.id);
                continue;
            }

            debug!("Executing scheduled replay {}", replay.id);

            // Note: In a real implementation, we would have access to ReplayManager
            // For now, we just update the scheduled replay state
            replay.execution_count += 1;
            replay.last_execution = Some(now);

            // Calculate next execution
            replay.next_execution =
                ScheduledReplay::calculate_next_execution(&replay.schedule, Some(now));

            // Update and re-queue if needed
            if replay.should_continue() {
                scheduled.write().insert(replay.id, replay.clone());
                queue.write().push(replay);
            } else {
                scheduled.write().remove(&replay.id);
            }
        }
    }
}

impl Default for ReplayScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_schedule_once() {
        let schedule = ReplaySchedule::Once {
            delay: Duration::from_secs(60),
        };

        let request = ReplayRequest {
            execution_id: Uuid::new_v4(),
            config: Default::default(),
            correlation_id: None,
        };

        let replay = ScheduledReplay::new(request, schedule);
        assert!(replay.should_continue());
        assert_eq!(replay.execution_count, 0);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_schedule_interval() {
        let schedule = ReplaySchedule::Interval {
            initial_delay: Duration::from_secs(10),
            interval: Duration::from_secs(60),
            max_executions: Some(5),
        };

        let request = ReplayRequest {
            execution_id: Uuid::new_v4(),
            config: Default::default(),
            correlation_id: None,
        };

        let mut replay = ScheduledReplay::new(request, schedule);

        // Should continue for first 5 executions
        for i in 0..5 {
            assert!(replay.should_continue());
            replay.execution_count = i + 1;
        }

        // Should stop after 5 executions
        assert!(!replay.should_continue());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_cron_expression() {
        let now = Utc::now();

        let hourly = ScheduledReplay::parse_cron_expression("hourly", now);
        assert!(hourly.is_some());
        assert!(hourly.unwrap() > now);

        let daily = ScheduledReplay::parse_cron_expression("daily", now);
        assert!(daily.is_some());
        assert!(daily.unwrap() > now);

        let weekly = ScheduledReplay::parse_cron_expression("weekly", now);
        assert!(weekly.is_some());
        assert!(weekly.unwrap() > now);

        let invalid = ScheduledReplay::parse_cron_expression("invalid", now);
        assert!(invalid.is_none());
    }
}
