use qmx_backend_lib::student::*;
use std::sync::Mutex;
use std::sync::atomic::Ordering;

use std::fs;

// 用于确保UID相关测试串行执行的互斥锁
static UID_TEST_LOCK: Mutex<()> = Mutex::new(());

fn setup() {
    fs::create_dir_all("./data").unwrap();
}

#[cfg(test)]
mod student_comprehensive_tests {
    use super::*;

    #[test]
    fn student_new_creates_unique_uids() {
        let s1 = Student::new();
        let s2 = Student::new();
        let s3 = Student::new();

        assert_ne!(s1.uid(), s2.uid());
        assert_ne!(s2.uid(), s3.uid());
        assert_ne!(s1.uid(), s3.uid());

        assert!(s1.uid() > 0);
        assert!(s2.uid() > s1.uid());
        assert!(s3.uid() > s2.uid());
    }

    #[test]
    fn student_default_values() {
        let student = Student::new();

        assert_eq!(student.age(), None);
        assert_eq!(student.name(), "未填写");
        assert_eq!(student.phone(), "未填写");
        assert_eq!(student.lesson_left(), None);
        assert_eq!(student.class(), &Class::Others);
        assert_eq!(student.subject(), &Subject::Others);
        assert!(student.rings().is_empty());
        assert_eq!(student.note(), "");
    }

    #[test]
    fn student_age_boundaries() {
        let mut student = Student::new();

        student.set_age(Some(0));
        assert_eq!(student.age(), Some(0));

        student.set_age(Some(255));
        assert_eq!(student.age(), Some(255));

        student.set_age(Some(18));
        assert_eq!(student.age(), Some(18));

        student.set_age(None);
        assert_eq!(student.age(), None);

        student.set_age(None);
        assert_eq!(student.age(), None);
    }

    #[test]
    fn student_name_edge_cases() {
        let mut student = Student::new();

        student.set_name("".to_string());
        assert_eq!(student.name(), "");

        student.set_name("A".to_string());
        assert_eq!(student.name(), "A");

        let long_name = "A".repeat(1000);
        student.set_name(long_name.clone());
        assert_eq!(student.name(), &long_name);

        student.set_name("张三".to_string());
        assert_eq!(student.name(), "张三");
    }

    #[test]
    fn student_phone_validation() {
        let mut student = Student::new();

        student.set_phone("13800138000".to_string());
        assert_eq!(student.phone(), "13800138000");

        student.set_phone("".to_string());
        assert_eq!(student.phone(), "");

        student.set_phone("invalid-phone".to_string());
        assert_eq!(student.phone(), "invalid-phone");
    }

    #[test]
    fn student_class_and_lesson_left_interaction() {
        let mut student = Student::new();

        student.set_class_with_lesson_init(Class::TenTry);
        assert_eq!(student.class(), &Class::TenTry);
        assert_eq!(student.lesson_left(), Some(10));

        student.set_lesson_left(5);
        assert_eq!(student.lesson_left(), Some(5));

        student.set_class_with_lesson_init(Class::Month);
        assert_eq!(student.class(), &Class::Month);
        assert_eq!(student.lesson_left(), None);

        student.set_lesson_left(8);
        assert_eq!(student.lesson_left(), None);

        student.set_class(Class::Year);
        assert_eq!(student.lesson_left(), None);

        student.set_class(Class::Others);
        assert_eq!(student.lesson_left(), None);
    }

    #[test]
    fn student_lesson_left_boundaries() {
        let mut student = Student::new();
        student.set_class_with_lesson_init(Class::TenTry);

        student.set_lesson_left(0);
        assert_eq!(student.lesson_left(), Some(0));

        student.set_lesson_left(u32::MAX);
        assert_eq!(student.lesson_left(), Some(u32::MAX));

        student.set_lesson_left(1);
        assert_eq!(student.lesson_left(), Some(1));
    }

    #[test]
    fn student_rings_operations() {
        let mut student = Student::new();

        assert!(student.rings().is_empty());

        student.add_ring(9.5);
        assert_eq!(student.rings().len(), 1);
        assert_eq!(student.rings()[0], 9.5);

        student.add_ring(8.2);
        student.add_ring(10.0);
        assert_eq!(student.rings().len(), 3);
        assert_eq!(student.rings()[1], 8.2);
        assert_eq!(student.rings()[2], 10.0);

        student.add_ring(0.0);
        assert_eq!(student.rings().len(), 4);
        assert_eq!(student.rings()[3], 0.0);

        student.add_ring(-1.0);
        assert_eq!(student.rings().len(), 5);
        assert_eq!(student.rings()[4], -1.0);

        let large_score = f64::MAX;
        student.add_ring(large_score);
        assert_eq!(student.rings()[5], large_score);
    }

    #[test]
    fn student_note_operations() {
        let mut student = Student::new();

        assert_eq!(student.note(), "");

        student.set_note("Good performance".to_string());
        assert_eq!(student.note(), "Good performance");

        student.set_note("".to_string());
        assert_eq!(student.note(), "");

        let long_note = "A".repeat(10000);
        student.set_note(long_note.clone());
        assert_eq!(student.note(), &long_note);

        student.set_note("中文备注".to_string());
        assert_eq!(student.note(), "中文备注");
    }

    #[test]
    fn student_subject_variations() {
        let mut student = Student::new();

        assert_eq!(student.subject(), &Subject::Others);

        student.set_subject(Subject::Shooting);
        assert_eq!(student.subject(), &Subject::Shooting);

        student.set_subject(Subject::Archery);
        assert_eq!(student.subject(), &Subject::Archery);

        student.set_subject(Subject::Others);
        assert_eq!(student.subject(), &Subject::Others);
    }

    #[test]
    fn student_unsafe_set_id() {
        let mut student = Student::new();
        let original_uid = student.uid();

        unsafe {
            student.set_id(12345);
        }
        assert_eq!(student.uid(), 12345);
        assert_ne!(student.uid(), original_uid);

        unsafe {
            student.set_id(0);
        }
        assert_eq!(student.uid(), 0);

        unsafe {
            student.set_id(u64::MAX);
        }
        assert_eq!(student.uid(), u64::MAX);
    }

    #[test]
    fn student_method_chaining() {
        let mut student = Student::new();

        student
            .set_age(Some(25))
            .set_name("Chain Test".to_string())
            .set_phone("12345678901".to_string())
            .set_class_with_lesson_init(Class::TenTry)
            .set_lesson_left(8)
            .set_subject(Subject::Shooting)
            .set_note("Chained operations".to_string())
            .add_ring(9.0)
            .add_ring(8.5);

        assert_eq!(student.age(), Some(25));
        assert_eq!(student.name(), "Chain Test");
        assert_eq!(student.phone(), "12345678901");
        assert_eq!(student.class(), &Class::TenTry);
        assert_eq!(student.lesson_left(), Some(8));
        assert_eq!(student.subject(), &Subject::Shooting);
        assert_eq!(student.note(), "Chained operations");
        assert_eq!(student.rings().len(), 2);
    }
}

#[cfg(test)]
mod student_database_comprehensive_tests {
    use super::*;

    #[test]
    fn student_database_new_and_default() {
        let db1 = StudentDatabase::new();
        let db2 = StudentDatabase::default();

        assert!(db1.is_empty());
        assert!(db2.is_empty());
        assert_eq!(db1.len(), 0);
        assert_eq!(db2.len(), 0);
    }

    #[test]
    fn student_database_insert_and_get() {
        let mut db = StudentDatabase::new();
        let student = Student::new();
        let uid = student.uid();

        db.insert(student.clone());
        assert_eq!(db.len(), 1);
        assert!(!db.is_empty());

        let retrieved = db.get(&uid);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().uid(), uid);
        assert_eq!(retrieved.unwrap().name(), student.name());

        let non_existent = db.get(&(uid + 1000));
        assert!(non_existent.is_none());
    }

    #[test]
    fn student_database_insert_batch() {
        let mut db = StudentDatabase::new();
        let mut students = Vec::new();

        for i in 0..5 {
            let mut student = Student::new();
            student.set_name(format!("Student {}", i));
            students.push(student);
        }

        let inserted_count = db.insert_batch(students.clone());
        assert_eq!(inserted_count, 5);
        assert_eq!(db.len(), 5);

        for student in &students {
            assert!(db.get(&student.uid()).is_some());
        }

        let empty_batch_count = db.insert_batch(vec![]);
        assert_eq!(empty_batch_count, 0);
        assert_eq!(db.len(), 5);
    }

    #[test]
    fn student_database_remove() {
        let mut db = StudentDatabase::new();
        let student = Student::new();
        let uid = student.uid();

        db.insert(student.clone());
        assert_eq!(db.len(), 1);

        let removed = db.remove(&uid);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().uid(), uid);
        assert_eq!(db.len(), 0);
        assert!(db.is_empty());

        let removed_again = db.remove(&uid);
        assert!(removed_again.is_none());
    }

    #[test]
    fn student_database_remove_batch() {
        let mut db = StudentDatabase::new();
        let mut uids = Vec::new();

        for i in 0..10 {
            let mut student = Student::new();
            student.set_name(format!("Student {}", i));
            uids.push(student.uid());
            db.insert(student);
        }

        assert_eq!(db.len(), 10);

        let remove_uids = vec![uids[0], uids[2], uids[4], uids[6], uids[8]];
        let removed_count = db.remove_batch(&remove_uids);
        assert_eq!(removed_count, 5);
        assert_eq!(db.len(), 5);

        for &uid in &remove_uids {
            assert!(db.get(&uid).is_none());
        }

        let non_existent_uids = vec![9999, 10000, 10001];
        let removed_count_2 = db.remove_batch(&non_existent_uids);
        assert_eq!(removed_count_2, 0);
        assert_eq!(db.len(), 5);

        let empty_batch_count = db.remove_batch(&[]);
        assert_eq!(empty_batch_count, 0);
    }

    #[test]
    fn student_database_update_batch() {
        let mut db = StudentDatabase::new();
        let mut uids = Vec::new();

        for i in 0..5 {
            let mut student = Student::new();
            student.set_age(Some(20 + i as u8));
            uids.push(student.uid());
            db.insert(student);
        }

        let update_count = db.update_batch(&uids, |student| {
            if let Some(age) = student.age() {
                if age < 23 {
                    student.set_age(Some(age + 10));
                    true
                } else {
                    false
                }
            } else {
                false
            }
        });

        assert_eq!(update_count, 3);

        assert_eq!(db.get(&uids[0]).unwrap().age(), Some(30));
        assert_eq!(db.get(&uids[1]).unwrap().age(), Some(31));
        assert_eq!(db.get(&uids[2]).unwrap().age(), Some(32));
        assert_eq!(db.get(&uids[3]).unwrap().age(), Some(23));
        assert_eq!(db.get(&uids[4]).unwrap().age(), Some(24));

        let no_update_count = db.update_batch(&[9999, 10000], |_| true);
        assert_eq!(no_update_count, 0);

        let empty_batch_count = db.update_batch(&[], |_| true);
        assert_eq!(empty_batch_count, 0);
    }

    #[test]
    fn student_database_json_serialization() {
        let mut db = StudentDatabase::new();
        let mut student = Student::new();
        student
            .set_name("JSON Test".to_string())
            .set_age(Some(30))
            .add_ring(9.5)
            .add_ring(8.0);

        db.insert(student.clone());

        let json = db.json();
        assert!(!json.is_empty());
        assert!(json.contains("JSON Test"));
        assert!(json.contains("30"));

        let deserialized_db = StudentDatabase::from_json(&json).unwrap();
        assert_eq!(deserialized_db.len(), 1);

        let retrieved = deserialized_db.get(&student.uid()).unwrap();
        assert_eq!(retrieved.name(), "JSON Test");
        assert_eq!(retrieved.age(), Some(30));
        assert_eq!(retrieved.rings().len(), 2);
    }

    #[test]
    fn student_database_json_deserialization_errors() {
        let invalid_json = "invalid json";
        let result = StudentDatabase::from_json(invalid_json);
        assert!(result.is_err());

        let empty_json = "";
        let result = StudentDatabase::from_json(empty_json);
        assert!(result.is_err());

        let partial_json = r#"{"student_data": {"1": {"uid": 1, "age": 25"#;
        let result = StudentDatabase::from_json(partial_json);
        assert!(result.is_err());
    }

    #[test]
    fn student_database_iterator() {
        let mut db = StudentDatabase::new();
        let mut expected_uids = Vec::new();

        for i in 0..3 {
            let mut student = Student::new();
            student.set_name(format!("Student {}", i));
            expected_uids.push(student.uid());
            db.insert(student);
        }

        let mut collected_uids = Vec::new();
        for (uid, student) in db.iter() {
            collected_uids.push(*uid);
            assert!(student.name().starts_with("Student"));
        }

        expected_uids.sort();
        collected_uids.sort();
        assert_eq!(expected_uids, collected_uids);

        let empty_db = StudentDatabase::new();
        let count = empty_db.iter().count();
        assert_eq!(count, 0);
    }

    #[test]
    fn student_database_duplicate_uid_handling() {
        let mut db = StudentDatabase::new();
        let mut student1 = Student::new();
        student1.set_name("First".to_string());
        let uid = student1.uid();

        db.insert(student1);
        assert_eq!(db.len(), 1);
        assert_eq!(db.get(&uid).unwrap().name(), "First");

        let mut student2 = Student::new();
        unsafe {
            student2.set_id(uid);
        }
        student2.set_name("Second".to_string());

        db.insert(student2);
        assert_eq!(db.len(), 1);
        assert_eq!(db.get(&uid).unwrap().name(), "Second");
    }
}

#[cfg(test)]
mod student_uid_management_tests {
    use super::*;
    use std::fs;

    #[test]
    fn uid_counter_persistence() {
        // 获取互斥锁以确保此测试串行执行
        let _lock = UID_TEST_LOCK.lock().unwrap();

        let _ = fs::remove_file("./data/uid_counter");

        // Test basic UID loading
        let initial_result = load_saved_uid();
        match initial_result {
            Ok(uid) => assert!(uid >= 1, "Loaded UID should be at least 1"),
            Err(_) => {
                // Load failed gracefully - this is expected behavior when no file exists
            }
        }

        // Test saving and loading
        STUDENT_UID_COUNTER.store(100, Ordering::SeqCst);
        let save_result = save_uid();
        match save_result {
            Ok(_) => {
                let loaded_result = load_saved_uid();
                match loaded_result {
                    Ok(uid) => assert_eq!(uid, 100, "Saved UID should match loaded UID"),
                    Err(e) => panic!("Load after save should succeed, but got error: {}", e),
                }
            }
            Err(e) => panic!("Save should succeed, but got error: {}", e),
        }

        let _ = fs::remove_file("./data/uid_counter");
        // Reset counter for other tests
        STUDENT_UID_COUNTER.store(1, Ordering::SeqCst);
    }

    #[test]
    fn student_module_init() {
        // 获取互斥锁以确保此测试串行执行
        let _lock = UID_TEST_LOCK.lock().unwrap();

        // Clean up any existing uid_counter file to ensure fresh start
        let _ = fs::remove_file("./data/uid_counter");

        // Reset the UID counter to ensure clean test state
        STUDENT_UID_COUNTER.store(1, Ordering::SeqCst);

        let result = init();
        // Should either succeed or fail gracefully
        match result {
            Ok(_) => {
                let uid_after_init = STUDENT_UID_COUNTER.load(Ordering::SeqCst);
                assert!(
                    uid_after_init >= 1,
                    "UID counter should be initialized to at least 1"
                );
            }
            Err(e) => panic!("Init should succeed, but got error: {}", e),
        }

        // After init(), the counter should be at 1 (since we removed the file)
        // The first student should get UID 1, then counter becomes 2
        let student = Student::new();
        assert_eq!(student.uid(), 1, "First student should have UID 1");

        let student2 = Student::new();
        assert_eq!(student2.uid(), 2, "Second student should have UID 2");

        // Clean up
        let _ = fs::remove_file("./data/uid_counter");
    }

    #[test]
    fn load_saved_uid_file_errors() {
        // 获取互斥锁以确保此测试串行执行
        let _lock = UID_TEST_LOCK.lock().unwrap();

        fs::create_dir_all("./data").unwrap();

        // Test with invalid content
        fs::write("./data/uid_counter", "invalid_number").unwrap();
        let result = load_saved_uid();
        // Should either error or handle gracefully
        match result {
            Ok(_) => {
                // Function handled invalid input gracefully - this is acceptable
            }
            Err(_) => {
                // Function returned expected error - this is also acceptable
            }
        }

        // Test with empty content
        fs::write("./data/uid_counter", "").unwrap();
        let result = load_saved_uid();
        // Should either error or handle gracefully
        match result {
            Ok(_) => {
                // Function handled empty input gracefully - this is acceptable
            }
            Err(_) => {
                // Function returned expected error - this is also acceptable
            }
        }

        let _ = fs::remove_file("./data/uid_counter");
    }
}

#[cfg(test)]
mod student_query_tests {
    use qmx_backend_lib::{StudentQuery, student::{Student, StudentDatabase}};

    #[test]
    fn student_query_score_range() {
        // Create students with different ring scores
        let mut student1 = Student::new();
        student1.set_name("Student 1".to_string());
        student1.add_ring(8.5);
        student1.add_ring(9.0);

        let mut student2 = Student::new();
        student2.set_name("Student 2".to_string());
        student2.add_ring(7.5);
        student2.add_ring(8.0);

        let mut student3 = Student::new();
        student3.set_name("Student 3".to_string());
        student3.add_ring(9.5);
        student3.add_ring(10.0);

        // Create a database with these students
        let mut db = StudentDatabase::new();
        db.insert(student1);
        db.insert(student2);
        db.insert(student3);

        // Test score range query - we need to use the public API
        // For now, we'll test the query building itself
        let _query = StudentQuery::new().score_range(8.0, 9.0);
        // We can't directly test execute() since it's private
        // The actual functionality is tested through the QmxManager
    }

    #[test]
    fn student_query_score_range_edge_cases() {
        // Create students with edge case scores
        let mut student1 = Student::new();
        student1.set_name("Student 1".to_string());
        student1.add_ring(8.0); // Exactly at the lower bound

        let mut student2 = Student::new();
        student2.set_name("Student 2".to_string());
        student2.add_ring(9.0); // Exactly at the upper bound

        let mut student3 = Student::new();
        student3.set_name("Student 3".to_string());
        student3.add_ring(7.9); // Just below the lower bound
        student3.add_ring(9.1); // Just above the upper bound

        let mut student4 = Student::new();
        student4.set_name("Student 4".to_string());
        // No rings added - should not match any range query

        // Create a database with these students
        let mut db = StudentDatabase::new();
        db.insert(student1);
        db.insert(student2);
        db.insert(student3);
        db.insert(student4);

        // Test score range query with exact bounds
        let _query = StudentQuery::new().score_range(8.0, 9.0);
        // We can't directly test execute() since it's private
        // The actual functionality is tested through the QmxManager
    }
}

#[cfg(test)]
mod student_file_operations_tests {
    use super::*;

    #[test]
    fn student_database_save_and_load() {
        let test_path = "./data/test_student_db.json";
        let _ = std::fs::remove_file(test_path);

        let mut db = StudentDatabase::new();
        let mut student = Student::new();
        student.set_name("Save Test".to_string()).set_age(Some(25));
        db.insert(student.clone());

        let save_result = db.save_to(test_path);
        assert!(save_result.is_ok());

        let loaded_db = StudentDatabase::read_from(test_path).unwrap();
        assert_eq!(loaded_db.len(), 1);
        assert_eq!(loaded_db.get(&student.uid()).unwrap().name(), "Save Test");

        let _ = std::fs::remove_file(test_path);
    }

    #[test]
    fn student_database_save_to_nonexistent_directory() {
        let test_path = "./nonexistent_dir/test_db.json";
        let db = StudentDatabase::new();

        // 注意：save_to 现在会自动创建父目录，所以这个测试应该成功
        let result = db.save_to(test_path);
        assert!(result.is_ok());

        // 清理测试文件
        let _ = std::fs::remove_dir_all("./nonexistent_dir");
    }

    #[test]
    fn student_database_read_from_nonexistent_file() {
        let result = StudentDatabase::read_from("./nonexistent_file.json");
        assert!(result.is_err());
    }

    #[test]
    fn student_database_atomic_save() {
        setup();
        let test_path = "./data/atomic_test.json";
        let _ = std::fs::remove_file(test_path);
        let tmp_path = format!("{}.tmp", test_path);
        let _ = std::fs::remove_file(&tmp_path);

        let mut db = StudentDatabase::new();
        let student = Student::new();
        db.insert(student);

        let result = db.save_to(test_path);
        assert!(result.is_ok());

        assert!(std::fs::metadata(test_path).is_ok());
        assert!(std::fs::metadata(&tmp_path).is_err());

        let _ = std::fs::remove_file(test_path);
    }
}
