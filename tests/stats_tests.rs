use qmx_backend_lib::cash::{Cash, CashDatabase};
use qmx_backend_lib::stats::*;
use qmx_backend_lib::student::{Class, Student, StudentDatabase};

#[cfg(test)]
mod stats_comprehensive_tests {
    use super::*;

    #[test]
    fn stats_empty_databases() {
        let student_db = StudentDatabase::new();
        let cash_db = CashDatabase::new();
        let stats = get_dashboard_stats(&student_db, &cash_db).unwrap();

        assert_eq!(stats.total_students, 0);
        assert_eq!(stats.total_revenue, 0);
        assert_eq!(stats.total_expense, 0);
        assert_eq!(stats.average_score, 0.0);
        assert_eq!(stats.max_score, 0.0);
        assert_eq!(stats.active_courses, 0);
    }

    #[test]
    fn stats_with_students_no_cash() {
        let mut student_db = StudentDatabase::new();
        let mut s1 = Student::new();
        s1.set_class(Class::Month).add_ring(8.0);
        student_db.insert(s1);

        let cash_db = CashDatabase::new();
        let stats = get_dashboard_stats(&student_db, &cash_db).unwrap();

        assert_eq!(stats.total_students, 1);
        assert_eq!(stats.total_revenue, 0);
        assert_eq!(stats.total_expense, 0);
        assert_eq!(stats.average_score, 8.0);
        assert_eq!(stats.max_score, 8.0);
        assert_eq!(stats.active_courses, 1);
    }

    #[test]
    fn stats_with_cash_no_students() {
        let student_db = StudentDatabase::new();
        let mut cash_db = CashDatabase::new();
        let mut c1 = Cash::new(None);
        c1.set_cash(100);
        cash_db.insert(c1);

        let stats = get_dashboard_stats(&student_db, &cash_db).unwrap();

        assert_eq!(stats.total_students, 0);
        assert_eq!(stats.total_revenue, 100);
        assert_eq!(stats.total_expense, 0);
        assert_eq!(stats.average_score, 0.0);
        assert_eq!(stats.max_score, 0.0);
        assert_eq!(stats.active_courses, 0);
    }

    #[test]
    fn stats_full_scenario() {
        let mut student_db = StudentDatabase::new();
        let mut s1 = Student::new();
        s1.set_class(Class::Year).add_ring(10.0).add_ring(9.0);
        student_db.insert(s1);
        let mut s2 = Student::new();
        s2.set_class(Class::TenTry).add_ring(7.5);
        student_db.insert(s2);
        let mut s3 = Student::new();
        s3.set_class(Class::Others);
        student_db.insert(s3);

        let mut cash_db = CashDatabase::new();
        let mut c1 = Cash::new(Some(1));
        c1.set_cash(5000);
        cash_db.insert(c1);
        let mut c2 = Cash::new(Some(2));
        c2.set_cash(300);
        cash_db.insert(c2);
        let mut c3 = Cash::new(None);
        c3.set_cash(-150);
        cash_db.insert(c3);
        let mut c4 = Cash::new(None);
        c4.set_cash(0);
        cash_db.insert(c4);

        let stats = get_dashboard_stats(&student_db, &cash_db).unwrap();

        assert_eq!(stats.total_students, 3);
        assert_eq!(stats.total_revenue, 5300);
        assert_eq!(stats.total_expense, 150);
        assert!((stats.average_score - (10.0 + 9.0 + 7.5) / 3.0).abs() < 1e-9);
        assert_eq!(stats.max_score, 10.0);
        assert_eq!(stats.active_courses, 2);
    }

    #[test]
    fn stats_students_with_no_scores() {
        let mut student_db = StudentDatabase::new();
        student_db.insert(Student::new());

        let cash_db = CashDatabase::new();
        let stats = get_dashboard_stats(&student_db, &cash_db).unwrap();

        assert_eq!(stats.total_students, 1);
        assert_eq!(stats.average_score, 0.0);
        assert_eq!(stats.max_score, 0.0);
    }

    #[test]
    fn stats_all_classes_are_others() {
        let mut student_db = StudentDatabase::new();
        let mut s1 = Student::new();
        s1.set_class(Class::Others);
        student_db.insert(s1);

        let cash_db = CashDatabase::new();
        let stats = get_dashboard_stats(&student_db, &cash_db).unwrap();

        assert_eq!(stats.active_courses, 0);
    }

    #[test]
    fn stats_revenue_and_expense_logic() {
        let student_db = StudentDatabase::new();
        let mut cash_db = CashDatabase::new();

        let mut c1 = Cash::new(None);
        c1.set_cash(100); // revenue
        let mut c2 = Cash::new(None);
        c2.set_cash(-50); // expense
        let mut c3 = Cash::new(None);
        c3.set_cash(0); // neutral
        let mut c4 = Cash::new(None);
        c4.set_cash(200); // revenue

        cash_db.insert(c1);
        cash_db.insert(c2);
        cash_db.insert(c3);
        cash_db.insert(c4);

        let stats = get_dashboard_stats(&student_db, &cash_db).unwrap();

        assert_eq!(stats.total_revenue, 300);
        assert_eq!(stats.total_expense, 50);
    }
}
