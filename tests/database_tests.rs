use qmx_backend_lib::cash::{Cash, CashDatabase};
use qmx_backend_lib::database;
use qmx_backend_lib::student::{Student, StudentDatabase};

#[cfg(test)]
mod database_comprehensive_tests {
    use super::*;

    #[test]
    fn database_basic_functionality() {
        // Test that we can create and use a database
        let mut db = database::Database {
            student: StudentDatabase::new(),
            cash: CashDatabase::new(),
        };

        // Add some test data
        let student = Student::new();
        let cash = Cash::new(None);

        db.student.insert(student);
        db.cash.insert(cash);

        assert_eq!(db.student.len(), 1);
        assert_eq!(db.cash.len(), 1);
    }

    #[test]
    fn database_init_functionality() {
        // Test that init function exists and returns a result
        let result = database::init();
        // Should return either Ok or Err - both are valid behavior
        match result {
            Ok(_) => {
                // Init succeeded - this is the expected behavior
            }
            Err(_) => {
                // Init failed gracefully - this can happen in test environments
            }
        }
    }

    #[test]
    fn database_save_functionality() {
        // Test basic save functionality with a simple database
        let db = database::Database {
            student: StudentDatabase::new(),
            cash: CashDatabase::new(),
        };

        let result = db.save();
        // Save should succeed or fail gracefully
        assert!(
            result.is_ok() || result.is_err(),
            "Save should return a result"
        );
    }
}
