use protonvpn_tui::state::{ConnectionState, Navigatable, ServerFilter, ServerSort, SortDirection};

mod server_filter {
    use super::*;

    #[test]
    fn test_default_filter() {
        let filter = ServerFilter::default();
        assert_eq!(filter, ServerFilter::Code);
    }

    #[test]
    fn test_filter_next_id_to_country() {
        let filter = ServerFilter::Code;
        assert_eq!(filter.next(), ServerFilter::Country);
    }

    #[test]
    fn test_filter_next_country_to_city() {
        let filter = ServerFilter::Country;
        assert_eq!(filter.next(), ServerFilter::City);
    }

    #[test]
    fn test_filter_next_city_to_id() {
        let filter = ServerFilter::City;
        assert_eq!(filter.next(), ServerFilter::Code);
    }

    #[test]
    fn test_filter_label_code() {
        let filter = ServerFilter::Code;
        assert_eq!(filter.label(), "Code");
    }

    #[test]
    fn test_filter_label_country() {
        let filter = ServerFilter::Country;
        assert_eq!(filter.label(), "Country");
    }

    #[test]
    fn test_filter_label_city() {
        let filter = ServerFilter::City;
        assert_eq!(filter.label(), "City");
    }

    #[test]
    fn test_filter_cycle() {
        let mut filter = ServerFilter::default();
        for _ in 0..3 {
            let _next = filter.next();
            filter = filter.next();
        }
        assert_eq!(filter, ServerFilter::Code);
    }
}

mod sort_direction {
    use super::*;

    #[test]
    fn test_default_direction() {
        let dir = SortDirection::default();
        assert_eq!(dir, SortDirection::Asc);
    }

    #[test]
    fn test_toggle_asc_to_desc() {
        let dir = SortDirection::Asc;
        assert_eq!(dir.toggle(), SortDirection::Desc);
    }

    #[test]
    fn test_toggle_desc_to_asc() {
        let dir = SortDirection::Desc;
        assert_eq!(dir.toggle(), SortDirection::Asc);
    }

    #[test]
    fn test_label_asc() {
        let dir = SortDirection::Asc;
        assert_eq!(dir.label(), "↑");
    }

    #[test]
    fn test_label_desc() {
        let dir = SortDirection::Desc;
        assert_eq!(dir.label(), "↓");
    }

    #[test]
    fn test_toggle_twice() {
        let dir = SortDirection::Asc;
        assert_eq!(dir.toggle().toggle(), SortDirection::Asc);
    }
}

mod server_sort {
    use super::*;

    #[test]
    fn test_default_sort() {
        let sort = ServerSort::default();
        assert_eq!(sort, ServerSort::Code);
    }

    #[test]
    fn test_sort_next_id_to_country() {
        let sort = ServerSort::Code;
        assert_eq!(sort.next(), ServerSort::Country);
    }

    #[test]
    fn test_sort_next_country_to_id() {
        let sort = ServerSort::Country;
        assert_eq!(sort.next(), ServerSort::Code);
    }

    #[test]
    fn test_label_id() {
        let sort = ServerSort::Code;
        assert_eq!(sort.label(), "ID");
    }

    #[test]
    fn test_label_country() {
        let sort = ServerSort::Country;
        assert_eq!(sort.label(), "Country");
    }

    #[test]
    fn test_sort_cycle() {
        let mut sort = ServerSort::default();
        for _ in 0..2 {
            sort = sort.next();
        }
        assert_eq!(sort, ServerSort::Code);
    }
}

mod navigatable {
    use super::*;

    mod move_next {
        use super::*;

        #[test]
        fn test_move_next_from_none() {
            let mut selected: Option<usize> = None;
            selected.move_next(10);
            assert_eq!(selected, Some(0));
        }

        #[test]
        fn test_move_next_from_middle() {
            let mut selected = Some(5);
            selected.move_next(10);
            assert_eq!(selected, Some(6));
        }

        #[test]
        fn test_move_next_at_last() {
            let mut selected = Some(9);
            selected.move_next(10);
            assert_eq!(selected, Some(9)); // Stays at max
        }

        #[test]
        fn test_move_next_zero_bounds() {
            let mut selected: Option<usize> = None;
            selected.move_next(0);
            assert_eq!(selected, None); // No movement
        }

        #[test]
        fn test_move_next_wraps_to_last() {
            let mut selected = Some(9);
            selected.move_next(10);
            assert_eq!(selected, Some(9));
        }
    }

    mod move_prev {
        use super::*;

        #[test]
        fn test_move_prev_from_none() {
            let mut selected: Option<usize> = None;
            selected.move_prev(10);
            assert_eq!(selected, Some(0));
        }

        #[test]
        fn test_move_prev_from_middle() {
            let mut selected = Some(5);
            selected.move_prev(10);
            assert_eq!(selected, Some(4));
        }

        #[test]
        fn test_move_prev_at_first() {
            let mut selected = Some(0);
            selected.move_prev(10);
            assert_eq!(selected, Some(0)); // Stays at first
        }

        #[test]
        fn test_move_prev_zero_bounds() {
            let mut selected: Option<usize> = None;
            selected.move_prev(0);
            assert_eq!(selected, None);
        }
    }

    mod move_first {
        use super::*;

        #[test]
        fn test_move_first() {
            let mut selected = Some(5);
            selected.move_first(10);
            assert_eq!(selected, Some(0));
        }

        #[test]
        fn test_move_first_zero_bounds() {
            let mut selected = Some(5);
            selected.move_first(0);
            assert_eq!(selected, Some(5)); // No change
        }
    }

    mod move_last {
        use super::*;

        #[test]
        fn test_move_last() {
            let mut selected = Some(5);
            selected.move_last(10);
            assert_eq!(selected, Some(9));
        }

        #[test]
        fn test_move_last_zero_bounds() {
            let mut selected = Some(5);
            selected.move_last(0);
            assert_eq!(selected, Some(5)); // No change
        }
    }

    mod move_page_down {
        use super::*;

        #[test]
        fn test_move_page_down_from_middle() {
            let mut selected = Some(5);
            selected.move_page_down(100);
            assert_eq!(selected, Some(15));
        }

        #[test]
        fn test_move_page_down_near_end() {
            let mut selected = Some(95);
            selected.move_page_down(100);
            assert_eq!(selected, Some(99)); // Clamps to max
        }

        #[test]
        fn test_move_page_down_zero_bounds() {
            let mut selected = Some(5);
            selected.move_page_down(0);
            assert_eq!(selected, Some(5)); // No change
        }

        #[test]
        fn test_move_page_down_from_none() {
            let mut selected: Option<usize> = None;
            selected.move_page_down(100);
            assert_eq!(selected, Some(0));
        }
    }

    mod move_page_up {
        use super::*;

        #[test]
        fn test_move_page_up_from_middle() {
            let mut selected = Some(50);
            selected.move_page_up(100);
            assert_eq!(selected, Some(40));
        }

        #[test]
        fn test_move_page_up_near_start() {
            let mut selected = Some(5);
            selected.move_page_up(100);
            assert_eq!(selected, Some(0)); // Clamps to 0
        }

        #[test]
        fn test_move_page_up_zero_bounds() {
            let mut selected = Some(5);
            selected.move_page_up(0);
            assert_eq!(selected, Some(5)); // No change
        }

        #[test]
        fn test_move_page_up_from_none() {
            let mut selected: Option<usize> = None;
            selected.move_page_up(100);
            assert_eq!(selected, Some(0));
        }
    }
}

mod connection_state {
    use super::*;

    #[test]
    fn test_default_disconnected() {
        let state = ConnectionState::default();
        assert_eq!(state, ConnectionState::Disconnected);
    }

    #[test]
    fn test_is_connected_true() {
        let state = ConnectionState::Connected {
            server: "JP".to_string(),
            ip: "1.2.3.4".to_string(),
            city: None,
            country: None,
            via: None,
        };
        assert!(state.is_connected());
    }

    #[test]
    fn test_is_connected_false_disconnected() {
        let state = ConnectionState::Disconnected;
        assert!(!state.is_connected());
    }

    #[test]
    fn test_is_connected_false_connecting() {
        let state = ConnectionState::Connecting;
        assert!(!state.is_connected());
    }

    #[test]
    fn test_is_connecting_true() {
        let state = ConnectionState::Connecting;
        assert!(state.is_connecting());
    }

    #[test]
    fn test_is_connecting_false() {
        let state = ConnectionState::Disconnected;
        assert!(!state.is_connecting());
    }

    #[test]
    fn test_can_connect_when_disconnected() {
        let state = ConnectionState::Disconnected;
        assert!(state.can_connect());
    }

    #[test]
    fn test_can_connect_when_error() {
        let state = ConnectionState::Error("timeout".to_string());
        assert!(state.can_connect());
    }

    #[test]
    fn test_can_connect_false_when_connected() {
        let state = ConnectionState::Connected {
            server: "JP".to_string(),
            ip: "1.2.3.4".to_string(),
            city: None,
            country: None,
            via: None,
        };
        assert!(!state.can_connect());
    }

    #[test]
    fn test_can_connect_false_when_connecting() {
        let state = ConnectionState::Connecting;
        assert!(!state.can_connect());
    }

    #[test]
    fn test_can_connect_false_when_disconnecting() {
        let state = ConnectionState::Disconnecting;
        assert!(!state.can_connect());
    }

    #[test]
    fn test_is_disconnecting_true() {
        let state = ConnectionState::Disconnecting;
        assert!(state.is_disconnecting());
    }

    #[test]
    fn test_is_disconnected_true() {
        let state = ConnectionState::Disconnected;
        assert!(state.is_disconnected());
    }

    #[test]
    fn test_is_disconnected_false_when_connected() {
        let state = ConnectionState::Connected {
            server: "JP".to_string(),
            ip: "1.2.3.4".to_string(),
            city: None,
            country: None,
            via: None,
        };
        assert!(!state.is_disconnected());
    }

    #[test]
    fn test_connected_state_equality() {
        let state1 = ConnectionState::Connected {
            server: "JP".to_string(),
            ip: "1.2.3.4".to_string(),
            city: None,
            country: None,
            via: None,
        };
        let state2 = ConnectionState::Connected {
            server: "JP".to_string(),
            ip: "1.2.3.4".to_string(),
            city: None,
            country: None,
            via: None,
        };
        assert_eq!(state1, state2);
    }

    #[test]
    fn test_error_state_equality() {
        let state1 = ConnectionState::Error("timeout".to_string());
        let state2 = ConnectionState::Error("timeout".to_string());
        assert_eq!(state1, state2);
    }

    #[test]
    fn test_error_state_inequality() {
        let state1 = ConnectionState::Error("timeout".to_string());
        let state2 = ConnectionState::Error("auth failed".to_string());
        assert_ne!(state1, state2);
    }
}
