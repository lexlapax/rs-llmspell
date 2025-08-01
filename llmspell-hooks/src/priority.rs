// ABOUTME: Priority ordering logic for hook execution with comparison utilities
// ABOUTME: Provides priority-based sorting and filtering for the hook registry

use crate::types::Priority;
use std::cmp::Ordering;

/// Priority comparator for hook ordering
#[derive(Debug, Clone)]
pub struct PriorityComparator;

impl PriorityComparator {
    /// Compare two priorities for execution order
    /// Lower priority values execute first (HIGHEST = i32::MIN executes before HIGH = -100)
    pub fn compare(a: &Priority, b: &Priority) -> Ordering {
        a.0.cmp(&b.0)
    }

    /// Sort a slice of items by priority
    pub fn sort_by_priority<T, F>(items: &mut [T], priority_fn: F)
    where
        F: Fn(&T) -> Priority,
    {
        items.sort_by(|a, b| Self::compare(&priority_fn(a), &priority_fn(b)));
    }

    /// Check if a priority is within a range
    pub fn is_in_range(priority: &Priority, min: &Priority, max: &Priority) -> bool {
        priority.0 >= min.0 && priority.0 <= max.0
    }

    /// Get priority bucket for grouping
    pub fn get_bucket(priority: &Priority) -> PriorityBucket {
        match priority.0 {
            i if i <= Priority::HIGHEST.0 + 50 => PriorityBucket::Critical,
            i if i <= Priority::HIGH.0 + 50 => PriorityBucket::High,
            i if i <= Priority::NORMAL.0 + 50 => PriorityBucket::Normal,
            i if i <= Priority::LOW.0 + 50 => PriorityBucket::Low,
            _ => PriorityBucket::Lowest,
        }
    }
}

/// Priority buckets for grouping hooks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PriorityBucket {
    Critical,
    High,
    Normal,
    Low,
    Lowest,
}

impl PriorityBucket {
    /// Get the display name for the bucket
    pub fn name(&self) -> &'static str {
        match self {
            PriorityBucket::Critical => "Critical",
            PriorityBucket::High => "High",
            PriorityBucket::Normal => "Normal",
            PriorityBucket::Low => "Low",
            PriorityBucket::Lowest => "Lowest",
        }
    }

    /// Get the priority range for this bucket
    pub fn range(&self) -> (Priority, Priority) {
        match self {
            PriorityBucket::Critical => (Priority::HIGHEST, Priority(Priority::HIGHEST.0 + 50)),
            PriorityBucket::High => (
                Priority(Priority::HIGHEST.0 + 51),
                Priority(Priority::HIGH.0 + 50),
            ),
            PriorityBucket::Normal => (
                Priority(Priority::HIGH.0 + 51),
                Priority(Priority::NORMAL.0 + 50),
            ),
            PriorityBucket::Low => (
                Priority(Priority::NORMAL.0 + 51),
                Priority(Priority::LOW.0 + 50),
            ),
            PriorityBucket::Lowest => (Priority(Priority::LOW.0 + 51), Priority::LOWEST),
        }
    }
}

/// Builder for custom priority configurations
#[derive(Debug, Clone)]
pub struct PriorityBuilder {
    base: i32,
}

impl PriorityBuilder {
    /// Create a new priority builder starting from a base value
    pub fn new(base: i32) -> Self {
        Self { base }
    }

    /// Create a priority relative to the base
    pub fn offset(self, offset: i32) -> Priority {
        Priority(self.base.saturating_add(offset))
    }

    /// Create a priority before the base (higher priority)
    pub fn before(self, distance: u32) -> Priority {
        Priority(self.base.saturating_sub(distance as i32))
    }

    /// Create a priority after the base (lower priority)
    pub fn after(self, distance: u32) -> Priority {
        Priority(self.base.saturating_add(distance as i32))
    }
}

/// Extension trait for Priority
impl Priority {
    /// Create a builder starting from this priority
    pub fn builder(self) -> PriorityBuilder {
        PriorityBuilder::new(self.0)
    }

    /// Check if this priority is higher than another (executes first)
    pub fn is_higher_than(&self, other: &Priority) -> bool {
        self.0 < other.0
    }

    /// Check if this priority is lower than another (executes later)
    pub fn is_lower_than(&self, other: &Priority) -> bool {
        self.0 > other.0
    }

    /// Get the distance between two priorities
    pub fn distance_from(&self, other: &Priority) -> u32 {
        self.0.abs_diff(other.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_priority_comparison() {
        assert!(Priority::HIGHEST.is_higher_than(&Priority::HIGH));
        assert!(Priority::HIGH.is_higher_than(&Priority::NORMAL));
        assert!(Priority::NORMAL.is_higher_than(&Priority::LOW));
        assert!(Priority::LOW.is_higher_than(&Priority::LOWEST));

        assert!(Priority::LOWEST.is_lower_than(&Priority::LOW));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_priority_sorting() {
        #[derive(Debug)]
        struct Item {
            name: String,
            priority: Priority,
        }

        let mut items = vec![
            Item {
                name: "low".to_string(),
                priority: Priority::LOW,
            },
            Item {
                name: "highest".to_string(),
                priority: Priority::HIGHEST,
            },
            Item {
                name: "normal".to_string(),
                priority: Priority::NORMAL,
            },
            Item {
                name: "high".to_string(),
                priority: Priority::HIGH,
            },
        ];

        PriorityComparator::sort_by_priority(&mut items, |item| item.priority);

        assert_eq!(items[0].name, "highest");
        assert_eq!(items[1].name, "high");
        assert_eq!(items[2].name, "normal");
        assert_eq!(items[3].name, "low");
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_priority_buckets() {
        assert_eq!(
            PriorityComparator::get_bucket(&Priority::HIGHEST),
            PriorityBucket::Critical
        );
        assert_eq!(
            PriorityComparator::get_bucket(&Priority::HIGH),
            PriorityBucket::High
        );
        assert_eq!(
            PriorityComparator::get_bucket(&Priority::NORMAL),
            PriorityBucket::Normal
        );
        assert_eq!(
            PriorityComparator::get_bucket(&Priority::LOW),
            PriorityBucket::Low
        );
        assert_eq!(
            PriorityComparator::get_bucket(&Priority::LOWEST),
            PriorityBucket::Lowest
        );
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_priority_builder() {
        let base = Priority::NORMAL;

        let before = base.builder().before(10);
        assert_eq!(before.0, Priority::NORMAL.0 - 10);

        let after = base.builder().after(10);
        assert_eq!(after.0, Priority::NORMAL.0 + 10);

        let offset = base.builder().offset(-5);
        assert_eq!(offset.0, Priority::NORMAL.0 - 5);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_priority_distance() {
        let p1 = Priority(0);
        let p2 = Priority(100);

        assert_eq!(p1.distance_from(&p2), 100);
        assert_eq!(p2.distance_from(&p1), 100);

        let p3 = Priority(-50);
        assert_eq!(p1.distance_from(&p3), 50);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_bucket_ranges() {
        let (min, max) = PriorityBucket::Normal.range();
        assert!(PriorityComparator::is_in_range(
            &Priority::NORMAL,
            &min,
            &max
        ));

        let (min, max) = PriorityBucket::High.range();
        assert!(PriorityComparator::is_in_range(&Priority::HIGH, &min, &max));
    }
}
