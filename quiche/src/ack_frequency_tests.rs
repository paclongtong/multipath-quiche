#[cfg(test)]
mod tests {
    use super::*;
    use crate::recovery::congestion::cubic::CubicState;
    use crate::recovery::congestion::bbr::BBRStateMachine;
    use crate::recovery::congestion::bbr2::BBR2StateMachine;
    use std::time::Instant;
    use std::net::SocketAddr;

    #[test]
    fn test_cubic_slow_start_requires_frequent_acks() {
        let mut conn = create_test_connection();
        let now = Instant::now();
        
        // Set CUBIC state to SlowStart
        conn.paths.get_mut(0).unwrap().recovery.congestion.cubic_state.state = CubicState::SlowStart;
        
        // Test the state detection
        let (in_slow_start, in_recovery, algorithm_specific) = 
            conn.analyze_congestion_state(&conn.paths.get(0).unwrap().recovery.congestion, now);
        
        assert!(in_slow_start, "Should detect slow start");
        assert!(algorithm_specific, "CUBIC slow start should need frequent ACKs");
        
        // Test that frequent ACKs are requested
        conn.check_ack_frequency_triggers_on_data_path(0);
        assert_eq!(conn.requested_ack_eliciting_threshold, 1, "Should request frequent ACKs in slow start");
    }

    #[test]
    fn test_cubic_congestion_avoidance_allows_relaxed_acks() {
        let mut conn = create_test_connection();
        let now = Instant::now();
        
        // Set CUBIC state to CongestionAvoidance
        conn.paths.get_mut(0).unwrap().recovery.congestion.cubic_state.state = CubicState::CongestionAvoidance;
        
        // Test the state detection
        let (in_slow_start, in_recovery, algorithm_specific) = 
            conn.analyze_congestion_state(&conn.paths.get(0).unwrap().recovery.congestion, now);
        
        assert!(!in_slow_start, "Should not detect slow start");
        assert!(!in_recovery, "Should not be in recovery");
        assert!(!algorithm_specific, "CUBIC congestion avoidance should allow relaxed ACKs");
        
        // Start with frequent ACKs, then transition to relaxed
        conn.requested_ack_eliciting_threshold = 1;
        conn.check_ack_frequency_triggers_on_data_path(0);
        
        assert_eq!(conn.requested_ack_eliciting_threshold, 10, "Should request relaxed ACKs in congestion avoidance");
        assert!(conn.needs_ack_frequency_update, "Should trigger ACK frequency update");
    }

    #[test]
    fn test_cubic_recovery_requires_frequent_acks() {
        let mut conn = create_test_connection();
        let now = Instant::now();
        
        // Set CUBIC state to Recovery
        conn.paths.get_mut(0).unwrap().recovery.congestion.cubic_state.state = CubicState::Recovery;
        
        // Test the state detection
        let (in_slow_start, in_recovery, algorithm_specific) = 
            conn.analyze_congestion_state(&conn.paths.get(0).unwrap().recovery.congestion, now);
        
        assert!(!in_slow_start, "Should not detect slow start");
        assert!(algorithm_specific, "CUBIC recovery should need frequent ACKs");
    }

    #[test] 
    fn test_bbr_startup_requires_frequent_acks() {
        let mut conn = create_test_connection();
        
        // Set BBR state to Startup
        conn.paths.get_mut(0).unwrap().recovery.congestion.bbr_state.state = BBRStateMachine::Startup;
        
        // Test BBR state detection
        let needs_frequent = conn.check_bbr_states(&conn.paths.get(0).unwrap().recovery.congestion);
        assert!(needs_frequent, "BBR startup should need frequent ACKs");
    }

    #[test]
    fn test_bbr_probe_bw_allows_relaxed_acks() {
        let mut conn = create_test_connection();
        
        // Set BBR state to ProbeBW
        conn.paths.get_mut(0).unwrap().recovery.congestion.bbr_state.state = BBRStateMachine::ProbeBW;
        
        // Test BBR state detection  
        let needs_frequent = conn.check_bbr_states(&conn.paths.get(0).unwrap().recovery.congestion);
        assert!(!needs_frequent, "BBR ProbeBW should allow relaxed ACKs");
    }

    #[test]
    fn test_bbr2_drain_requires_frequent_acks() {
        let mut conn = create_test_connection();
        
        // Set BBR2 state to Drain
        conn.paths.get_mut(0).unwrap().recovery.congestion.bbr2_state.state = BBR2StateMachine::Drain;
        
        // Test BBR2 state detection
        let needs_frequent = conn.check_bbr_states(&conn.paths.get(0).unwrap().recovery.congestion);
        assert!(needs_frequent, "BBR2 drain should need frequent ACKs");
    }

    #[test]
    fn test_state_transition_slow_start_to_congestion_avoidance() {
        let mut conn = create_test_connection();
        
        // Start in slow start
        conn.paths.get_mut(0).unwrap().recovery.congestion.cubic_state.state = CubicState::SlowStart;
        conn.requested_ack_eliciting_threshold = 10; // Start with relaxed to test transition
        
        conn.check_ack_frequency_triggers_on_data_path(0);
        assert_eq!(conn.requested_ack_eliciting_threshold, 1, "Should switch to frequent ACKs in slow start");
        
        // Transition to congestion avoidance
        conn.paths.get_mut(0).unwrap().recovery.congestion.cubic_state.state = CubicState::CongestionAvoidance;
        conn.needs_ack_frequency_update = false; // Reset flag
        
        conn.check_ack_frequency_triggers_on_data_path(0);
        assert_eq!(conn.requested_ack_eliciting_threshold, 10, "Should switch to relaxed ACKs in congestion avoidance");
        assert!(conn.needs_ack_frequency_update, "Should trigger update for transition");
    }

    #[test]
    fn test_no_redundant_ack_frequency_updates() {
        let mut conn = create_test_connection();
        
        // Set to congestion avoidance
        conn.paths.get_mut(0).unwrap().recovery.congestion.cubic_state.state = CubicState::CongestionAvoidance;
        conn.requested_ack_eliciting_threshold = 10; // Already set to relaxed
        conn.needs_ack_frequency_update = false;
        
        conn.check_ack_frequency_triggers_on_data_path(0);
        assert!(!conn.needs_ack_frequency_update, "Should not trigger redundant update");
        
        // Same test for slow start
        conn.paths.get_mut(0).unwrap().recovery.congestion.cubic_state.state = CubicState::SlowStart;
        conn.requested_ack_eliciting_threshold = 1; // Already set to frequent
        conn.needs_ack_frequency_update = false;
        
        conn.check_ack_frequency_triggers_on_data_path(0);
        assert!(!conn.needs_ack_frequency_update, "Should not trigger redundant update");
    }

    // Helper function to create a test connection
    fn create_test_connection() -> Connection {
        let config = Config::new(crate::PROTOCOL_VERSION).unwrap();
        let mut conn = Connection::connect(
            None,
            &crate::ConnectionId::from_ref(b"test"),
            "127.0.0.1:443".parse().unwrap(),
            "127.0.0.1:0".parse().unwrap(),  
            &config,
            None,
            false
        ).unwrap();
        
        // Ensure handshake is completed for ACK frequency logic to activate
        conn.handshake_completed = true;
        
        conn
    }
}