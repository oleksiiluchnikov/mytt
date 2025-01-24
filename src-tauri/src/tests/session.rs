
// tests/session.rs
#[cfg(test)]
mod session_tests {
    use crate::Session;
    use crate::SessionType;

    #[test]
    fn test_start_session() {
        let mut session = Session::new();
        assert_eq!(session.current_session, SessionType::Work);
        session.start_session();
        assert_eq!(session.current_session, SessionType::ShortBreak);
        assert_eq!(session.work_sessions, 1);
        session.start_session();
        assert_eq!(session.current_session, SessionType::Work);
        assert_eq!(session.work_sessions, 1);
    }

    #[test]
    fn test_long_break_session() {
        let mut session = Session::new();
        for _ in 0..3 {
            session.start_session();
            session.start_session();
        }
        assert_eq!(session.current_session, SessionType::LongBreak);
        assert_eq!(session.work_sessions, 4);
        assert_eq!(session.long_break_sessions, 0);
        session.start_session();
        assert_eq!(session.current_session, SessionType::Work);
        assert_eq!(session.work_sessions, 4);
        assert_eq!(session.long_break_sessions, 1);
    }

    #[test]
    fn test_get_session_duration() {
        let session = Session::new();
        assert_eq!(session.get_session_duration(), 25);
        let mut session = Session {
            current_session: SessionType::ShortBreak,
            ..Default::default()
        };
        assert_eq!(session.get_session_duration(), 5);
        let mut session = Session {
            current_session: SessionType::LongBreak,
            ..Default::default()
        };
        assert_eq!(session.get_session_duration(), 15);
    }

    #[test]
    fn test_skip_break() {
        let mut session = Session::new();
        session.start_session();
        assert_eq!(session.current_session, SessionType::ShortBreak);
        session.skip_break();
        assert_eq!(session.current_session, SessionType::Work);
        let mut session = Session {
            current_session: SessionType::LongBreak,
            ..Default::default()
        };
        session.skip_break();
        assert_eq!(session.current_session, SessionType::Work);
    }
}

