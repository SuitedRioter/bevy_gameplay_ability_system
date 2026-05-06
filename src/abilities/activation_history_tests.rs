#[cfg(test)]
mod activation_history_tests {
    use crate::abilities::components::{AbilityActivationHistory, ActivationResult};

    #[test]
    fn test_activation_history_creation() {
        let history = AbilityActivationHistory::new();
        assert_eq!(history.activation_count, 0);
        assert_eq!(history.successful_activation_count, 0);
        assert_eq!(history.failed_activation_count, 0);
        assert_eq!(history.last_activation_time, 0.0);
        assert!(history.last_successful_activation_time.is_none());
    }

    #[test]
    fn test_record_successful_activation() {
        let mut history = AbilityActivationHistory::new();
        history.record_activation(1.0, ActivationResult::Success);

        assert_eq!(history.activation_count, 1);
        assert_eq!(history.successful_activation_count, 1);
        assert_eq!(history.failed_activation_count, 0);
        assert_eq!(history.last_activation_time, 1.0);
        assert_eq!(history.last_successful_activation_time, Some(1.0));
    }

    #[test]
    fn test_record_failed_activation() {
        let mut history = AbilityActivationHistory::new();
        history.record_activation(1.0, ActivationResult::FailedCooldown);

        assert_eq!(history.activation_count, 1);
        assert_eq!(history.successful_activation_count, 0);
        assert_eq!(history.failed_activation_count, 1);
        assert_eq!(history.last_activation_time, 1.0);
        assert!(history.last_successful_activation_time.is_none());
    }

    #[test]
    fn test_multiple_activations() {
        let mut history = AbilityActivationHistory::new();
        history.record_activation(1.0, ActivationResult::Success);
        history.record_activation(2.0, ActivationResult::FailedCost);
        history.record_activation(3.0, ActivationResult::Success);

        assert_eq!(history.activation_count, 3);
        assert_eq!(history.successful_activation_count, 2);
        assert_eq!(history.failed_activation_count, 1);
        assert_eq!(history.last_activation_time, 3.0);
        assert_eq!(history.last_successful_activation_time, Some(3.0));
    }

    #[test]
    fn test_time_since_last_activation() {
        let mut history = AbilityActivationHistory::new();
        history.record_activation(1.0, ActivationResult::Success);

        assert_eq!(history.time_since_last_activation(5.0), 4.0);
    }

    #[test]
    fn test_time_since_last_successful_activation() {
        let mut history = AbilityActivationHistory::new();
        history.record_activation(1.0, ActivationResult::Success);
        history.record_activation(2.0, ActivationResult::FailedCost);

        assert_eq!(
            history.time_since_last_successful_activation(5.0),
            Some(4.0)
        );
    }

    #[test]
    fn test_success_rate() {
        let mut history = AbilityActivationHistory::new();
        assert_eq!(history.success_rate(), 0.0);

        history.record_activation(1.0, ActivationResult::Success);
        assert_eq!(history.success_rate(), 1.0);

        history.record_activation(2.0, ActivationResult::FailedCost);
        assert_eq!(history.success_rate(), 0.5);

        history.record_activation(3.0, ActivationResult::Success);
        assert!((history.success_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_activation_result_variants() {
        let mut history = AbilityActivationHistory::new();

        // 测试所有失败类型
        history.record_activation(1.0, ActivationResult::FailedRequirements);
        history.record_activation(2.0, ActivationResult::FailedCost);
        history.record_activation(3.0, ActivationResult::FailedCooldown);
        history.record_activation(4.0, ActivationResult::FailedBlocked);
        history.record_activation(5.0, ActivationResult::Failed);

        assert_eq!(history.activation_count, 5);
        assert_eq!(history.successful_activation_count, 0);
        assert_eq!(history.failed_activation_count, 5);
    }
}
