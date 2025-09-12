use chrono::{Duration, Utc};
use qmx_backend_lib::cash::*;
use std::sync::atomic::Ordering;
use std::fs;

fn setup() {
    fs::create_dir_all("./data").unwrap();
}

#[cfg(test)]
mod cash_comprehensive_tests {
    use super::*;

    #[test]
    fn cash_new_creates_unique_uids() {
        let c1 = Cash::new(Some(1));
        let c2 = Cash::new(Some(1));
        let c3 = Cash::new_installment(
            Some(2),
            1200,
            12,
            PaymentFrequency::Monthly,
            Utc::now(),
            1,
            None,
        );

        assert_ne!(c1.uid, c2.uid);
        assert_ne!(c2.uid, c3.uid);
        assert!(c1.uid > 0);
        assert!(c2.uid > c1.uid);
        assert!(c3.uid > c2.uid);
    }

    #[test]
    fn cash_default_values() {
        let cash = Cash::new(Some(10));
        assert_eq!(cash.student_id, Some(10));
        assert_eq!(cash.cash, 0);
        assert_eq!(cash.note, None);
        assert!(cash.installment.is_none());
    }

    #[test]
    fn cash_setters_and_getters() {
        let mut cash = Cash::new(None);

        cash.set_id(100);
        assert_eq!(cash.student_id, Some(100));

        cash.set_id(0);
        assert_eq!(cash.student_id, None);

        cash.set_cash(500);
        assert_eq!(cash.cash, 500);

        cash.add(-200);
        assert_eq!(cash.cash, 300);

        cash.set_note(Some("Test note".to_string()));
        assert_eq!(cash.note(), Some("Test note"));

        cash.set_note(None);
        assert_eq!(cash.note(), None);
    }

    #[test]
    fn cash_installment_creation() {
        let now = Utc::now();
        let cash = Cash::new_installment(
            Some(1),
            1200,
            12,
            PaymentFrequency::Monthly,
            now,
            1,
            Some(101),
        );

        assert_eq!(cash.cash, 100);
        assert!(cash.is_installment());

        let installment = cash.installment.as_ref().unwrap();
        assert_eq!(installment.plan_id, 101);
        assert_eq!(installment.total_amount, 1200);
        assert_eq!(installment.total_installments, 12);
        assert_eq!(installment.current_installment, 1);
        assert_eq!(installment.frequency, PaymentFrequency::Monthly);
        assert_eq!(installment.due_date, now);
        assert_eq!(installment.status, InstallmentStatus::Pending);
    }

    #[test]
    fn cash_installment_amount_calculation_with_remainder() {
        // 1000 / 3 = 333 with remainder 1
        let c1 = Cash::new_installment(
            None,
            1000,
            3,
            PaymentFrequency::Weekly,
            Utc::now(),
            1,
            Some(1),
        );
        let c2 = Cash::new_installment(
            None,
            1000,
            3,
            PaymentFrequency::Weekly,
            Utc::now(),
            2,
            Some(1),
        );
        let c3 = Cash::new_installment(
            None,
            1000,
            3,
            PaymentFrequency::Weekly,
            Utc::now(),
            3,
            Some(1),
        );

        assert_eq!(c1.cash, 333);
        assert_eq!(c2.cash, 333);
        assert_eq!(c3.cash, 334); // Last installment gets the remainder
    }

    #[test]
    fn cash_installment_status_update() {
        let mut cash =
            Cash::new_installment(None, 100, 1, PaymentFrequency::Weekly, Utc::now(), 1, None);

        cash.set_installment_status(InstallmentStatus::Paid);
        assert_eq!(
            cash.installment.as_ref().unwrap().status,
            InstallmentStatus::Paid
        );

        cash.set_installment_status(InstallmentStatus::Overdue);
        assert_eq!(
            cash.installment.as_ref().unwrap().status,
            InstallmentStatus::Overdue
        );

        // no-op on non-installment record
        let mut non_installment = Cash::new(None);
        non_installment.set_installment_status(InstallmentStatus::Paid);
        assert!(!non_installment.is_installment());
    }
}

#[cfg(test)]
mod cash_database_comprehensive_tests {
    use super::*;

    pub(super) fn setup_db_with_installments() -> (CashDatabase, u64) {
        let mut db = CashDatabase::new();
        let plan_id = CASH_UID_COUNTER.fetch_add(1, Ordering::SeqCst);

        let c1 = Cash::new_installment(
            Some(1),
            1000,
            3,
            PaymentFrequency::Monthly,
            Utc::now() - Duration::days(30),
            1,
            Some(plan_id),
        );
        let c2 = Cash::new_installment(
            Some(1),
            1000,
            3,
            PaymentFrequency::Monthly,
            Utc::now() + Duration::days(1),
            2,
            Some(plan_id),
        );

        db.insert(c1);
        db.insert(c2);

        (db, plan_id)
    }

    #[test]
    fn cash_database_crud() {
        let mut db = CashDatabase::new();
        let cash = Cash::new(Some(1));
        let uid = cash.uid;

        db.insert(cash.clone());
        assert_eq!(db.len(), 1);
        assert!(db.get(&uid).is_some());

        let removed = db.remove(&uid);
        assert!(removed.is_some());
        assert_eq!(db.len(), 0);
    }

    #[test]
    fn cash_database_batch_ops() {
        let mut db = CashDatabase::new();
        let records: Vec<Cash> = (0..5).map(|_| Cash::new(Some(1))).collect();
        let uids: Vec<u64> = records.iter().map(|c| c.uid).collect();

        db.insert_batch(records);
        assert_eq!(db.len(), 5);

        let update_count = db.update_batch(&[uids[0], uids[2]], |c| {
            c.set_cash(100);
            true
        });
        assert_eq!(update_count, 2);
        assert_eq!(db.get(&uids[0]).unwrap().cash, 100);

        let remove_count = db.remove_batch(&[uids[1], uids[3]]);
        assert_eq!(remove_count, 2);
        assert_eq!(db.len(), 3);
    }

    #[test]
    fn cash_database_json_roundtrip() {
        let (db, _) = setup_db_with_installments();
        let json = db.json();
        let db_from_json = CashDatabase::from_json(&json).unwrap();
        assert_eq!(db.len(), db_from_json.len());
    }

    #[test]
    fn cash_database_json_errors() {
        assert!(CashDatabase::from_json("invalid").is_err());
        assert!(CashDatabase::from_json("").is_err());
    }

    #[test]
    fn cash_get_installments() {
        let (db, plan_id) = setup_db_with_installments();
        assert_eq!(db.get_installments().len(), 2);
        assert_eq!(db.get_installments_by_plan(plan_id).len(), 2);
        assert_eq!(db.get_installments_by_plan(999).len(), 0);
    }

    #[test]
    fn cash_get_overdue_installments() {
        let (mut db, _) = setup_db_with_installments();
        assert_eq!(db.get_overdue_installments().len(), 1);

        let overdue_uid = db.get_overdue_installments()[0].uid;
        db.update_batch(&[overdue_uid], |c| {
            c.set_installment_status(InstallmentStatus::Paid);
            true
        });
        assert_eq!(db.get_overdue_installments().len(), 0);
    }

    #[test]
    fn cash_get_student_installments() {
        let (db, _) = setup_db_with_installments();
        assert_eq!(db.get_student_installments(1).len(), 2);
        assert_eq!(db.get_student_installments(2).len(), 0);
    }

    #[test]
    fn cash_generate_next_installment() {
        let (mut db, plan_id) = setup_db_with_installments();

        let next_uid = db
            .generate_next_installment(plan_id, Utc::now() + Duration::days(30))
            .unwrap();
        let next_cash = db.get(&next_uid).unwrap();
        let installment = next_cash.installment.as_ref().unwrap();

        assert_eq!(installment.current_installment, 3);
        assert_eq!(db.get_installments_by_plan(plan_id).len(), 3);
    }

    #[test]
    fn cash_generate_next_installment_errors() {
        let (mut db, plan_id) = setup_db_with_installments();

        // Plan not found
        assert!(db.generate_next_installment(999, Utc::now()).is_err());

        // Plan already completed
        let _ = db.generate_next_installment(plan_id, Utc::now() + Duration::days(30));
        assert!(db.generate_next_installment(plan_id, Utc::now()).is_err());
    }

    #[test]
    fn cash_cancel_installment_plan() {
        let (mut db, plan_id) = setup_db_with_installments();

        // One is overdue, one is pending. Both should be cancelled.
        let cancelled_count = db.cancel_installment_plan(plan_id);
        assert_eq!(cancelled_count, 2);

        let installments = db.get_installments_by_plan(plan_id);
        for cash in installments {
            assert_eq!(
                cash.installment.as_ref().unwrap().status,
                InstallmentStatus::Cancelled
            );
        }

        // Cancel again, should be 0.
        assert_eq!(db.cancel_installment_plan(plan_id), 0);
    }
}

#[cfg(test)]
mod cash_file_operations_tests {
    use super::*;

    #[test]
    fn cash_uid_persistence() {
        setup();
        let path = "./data/cash_uid_counter";
        let _ = fs::remove_file(path);

        let initial_uid = load_saved_cash_uid().unwrap();
        assert_eq!(initial_uid, 1);

        CASH_UID_COUNTER.store(200, Ordering::SeqCst);
        save_uid().unwrap();

        let loaded_uid = load_saved_cash_uid().unwrap();
        assert_eq!(loaded_uid, 200);

        let _ = fs::remove_file(path);
        let _ = fs::remove_file("./data/cash_uid_counter");
    }

    #[test]
    fn cash_database_save_and_load() {
        setup();
        let test_path = "./data/test_cash_db.json";
        let _ = fs::remove_file(test_path);

        let (db, _) = cash_database_comprehensive_tests::setup_db_with_installments();
        db.save_to(test_path).unwrap();

        let loaded_db = CashDatabase::read_from(test_path).unwrap();
        assert_eq!(db.len(), loaded_db.len());

        let _ = fs::remove_file(test_path);
    }

    #[test]
    fn cash_file_errors() {
        assert!(CashDatabase::read_from("./nonexistent_cash.json").is_err());
        assert!(
            CashDatabase::new()
                .save_to("./nonexistent_dir/db.json")
                .is_err()
        );
    }
}
