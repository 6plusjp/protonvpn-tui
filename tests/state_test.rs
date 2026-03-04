use protonvpn_tui::state::{ServerFilter, ServerSort, SortDirection};

mod server_filter {
    use super::*;

    #[test]
    fn test_default_filter() {
        let filter = ServerFilter::default();
        assert_eq!(filter, ServerFilter::Id);
    }

    #[test]
    fn test_filter_next_id_to_country() {
        let filter = ServerFilter::Id;
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
        assert_eq!(filter.next(), ServerFilter::Id);
    }

    #[test]
    fn test_filter_label_id() {
        let filter = ServerFilter::Id;
        assert_eq!(filter.label(), "ID");
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
        assert_eq!(filter, ServerFilter::Id);
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
        assert_eq!(sort, ServerSort::Id);
    }

    #[test]
    fn test_sort_next_id_to_country() {
        let sort = ServerSort::Id;
        assert_eq!(sort.next(), ServerSort::Country);
    }

    #[test]
    fn test_sort_next_country_to_id() {
        let sort = ServerSort::Country;
        assert_eq!(sort.next(), ServerSort::Id);
    }

    #[test]
    fn test_label_id() {
        let sort = ServerSort::Id;
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
        assert_eq!(sort, ServerSort::Id);
    }
}
