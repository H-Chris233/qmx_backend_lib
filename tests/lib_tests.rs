use qmx_backend_lib::{cash::*, stats::*, student::*};

#[cfg(test)]
mod tests_student {
    use super::*;

    #[test]
    fn student_new_and_setters() {
        let mut s = Student::new();
        let uid = s.uid();
        assert!(uid >= 1);
        assert_eq!(s.age(), 0);
        assert_eq!(s.name(), "未填写");
        assert_eq!(s.phone(), "未填写");
        assert_eq!(s.lesson_left(), None);
        assert_eq!(s.class(), &Class::Others);
        assert_eq!(s.subject(), &Subject::Others);

        s.set_age(18)
            .set_name("Alice".to_string())
            .set_phone("123".to_string())
            .set_subject(Subject::Shooting);
        assert_eq!(s.age(), 18);
        assert_eq!(s.name(), "Alice");
        assert_eq!(s.phone(), "123");
        assert_eq!(s.subject(), &Subject::Shooting);

        s.set_class(Class::TenTry);
        assert_eq!(s.class(), &Class::TenTry);
        assert_eq!(s.lesson_left(), Some(10));

        s.set_lesson_left(7);
        assert_eq!(s.lesson_left(), Some(7));

        s.add_ring(9.8).add_ring(8.2);
        assert_eq!(s.rings().len(), 2);

        s.set_note("good".to_string());
        assert_eq!(s.note(), "good");
    }

    #[test]
    fn student_db_crud_and_json_roundtrip() {
        let mut db = StudentDatabase::new();
        let mut s1 = Student::new();
        s1.set_name("Bob".to_string());
        let uid1 = s1.uid();
        db.insert(s1.clone());
        assert_eq!(db.len(), 1);
        assert!(db.get(&uid1).is_some());

        let json = db.json();
        let db2 = StudentDatabase::from_json(&json).unwrap();
        assert_eq!(db2.len(), 1);
        assert_eq!(db2.get(&uid1).unwrap().name(), "Bob");

        let removed = db.remove(&uid1);
        assert!(removed.is_some());
        assert!(db.get(&uid1).is_none());
    }
}

#[cfg(test)]
mod tests_cash {
    use super::*;
    use chrono::{Duration, Utc};

    #[test]
    fn cash_basic_and_db_ops() {
        let mut db = CashDatabase::new();
        let mut c = Cash::new(None);
        assert!(c.uid >= 1);
        assert_eq!(c.cash, 0);
        c.add(100);
        assert_eq!(c.cash, 100);
        c.set_cash(-50);
        assert_eq!(c.cash, -50);
        c.set_note(Some("note".to_string()));
        assert_eq!(c.note(), Some("note"));
        db.insert(c.clone());
        assert_eq!(db.len(), 1);
        assert!(db.get(&c.uid).is_some());
        let removed = db.remove(&c.uid);
        assert!(removed.is_some());
        assert!(db.get(&c.uid).is_none());
    }

    #[test]
    fn installment_flow_and_helpers() {
        let plan_total = 1000i64;
        let total_n = 3u32;
        let due1 = Utc::now() - Duration::days(1);
        let c1 = Cash::new_installment(
            Some(1),
            plan_total,
            total_n,
            PaymentFrequency::Monthly,
            due1,
            1,
            None,
        );
        assert!(c1.is_installment());
        let plan_id = c1.installment_plan_id().unwrap();

        let mut db = CashDatabase::new();
        db.insert(c1.clone());

        let due2 = Utc::now() + Duration::days(29);
        let uid2 = db.generate_next_installment(plan_id, due2).unwrap();
        assert!(db.get(&uid2).unwrap().is_installment());

        let due3 = Utc::now() + Duration::days(59);
        let uid3 = db.generate_next_installment(plan_id, due3).unwrap();
        assert!(db.get(&uid3).unwrap().is_installment());

        assert_eq!(db.get_installments_by_plan(plan_id).len(), 3);

        let overdues = db.get_overdue_installments();
        assert!(
            overdues
                .iter()
                .any(|c| c.installment.as_ref().unwrap().current_installment == 1)
        );

        let cancelled = db.cancel_installment_plan(plan_id);
        assert!(cancelled >= 1);
    }
}

#[cfg(test)]
mod tests_stats {
    use super::*;

    #[test]
    fn dashboard_stats_basic() {
        let mut sdb = StudentDatabase::new();
        let mut s1 = Student::new();
        s1.set_class(Class::Month).add_ring(9.0).add_ring(7.0);
        sdb.insert(s1);
        let mut s2 = Student::new();
        s2.set_class(Class::Others);
        sdb.insert(s2);

        let mut cdb = CashDatabase::new();
        let mut c1 = Cash::new(None);
        c1.set_cash(200);
        cdb.insert(c1);
        let mut c2 = Cash::new(None);
        c2.set_cash(-50);
        cdb.insert(c2);

        let stats = get_dashboard_stats(&sdb, &cdb).unwrap();
        assert_eq!(stats.total_students, 2);
        assert_eq!(stats.total_revenue, 200);
        assert_eq!(stats.total_expense, 50);
        assert_eq!(stats.active_courses, 1);
        assert!((stats.average_score - 8.0).abs() < 1e-9);
        assert_eq!(stats.max_score, 9.0);
    }
}
