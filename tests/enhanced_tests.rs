// 增强测试集合 - 专注于边界条件、错误处理和性能测试

use chrono::{Duration, Utc};
use qmx_backend_lib::cash::*;
use qmx_backend_lib::student::*;
use qmx_backend_lib::*;
use std::fs;
use tempfile::TempDir;

fn setup() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    std::env::set_current_dir(temp_dir.path()).expect("Failed to change directory");
    let _ = std::fs::create_dir_all("data");
    temp_dir
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_database_with_corrupted_files() {
        let _temp_dir = setup();

        // 创建损坏的JSON文件
        fs::write("./data/student_database.json", "invalid json content").unwrap();
        fs::write("./data/cash_database.json", "{incomplete json").unwrap();

        // 数据库应该能够处理损坏的文件并重新初始化
        let result = database::init();
        // 如果损坏的文件导致初始化失败，这是预期的行为
        // 我们测试的是系统能够优雅地处理这种情况
        if result.is_err() {
            // 删除损坏的文件，重新初始化
            let _ = fs::remove_file("./data/student_database.json");
            let _ = fs::remove_file("./data/cash_database.json");
            let retry_result = database::init();
            assert!(
                retry_result.is_ok(),
                "Database should initialize after removing corrupted files"
            );
        } else {
            let db = result.unwrap();
            assert_eq!(db.student.len(), 0);
            assert_eq!(db.cash.len(), 0);
        }
    }

    #[test]
    fn test_student_extreme_values() {
        let _temp_dir = setup();
        let mut student = Student::new();

        // 测试极端年龄值
        student.set_age(255); // u8::MAX
        assert_eq!(student.age(), 255);

        // 测试极端lesson_left值 - 需要先设置为TenTry班级才能设置课时
        assert_eq!(student.lesson_left(), None);
        student.set_class(Class::TenTry); // 这会设置lesson_left为Some(10)
        assert_eq!(student.lesson_left(), Some(10));
        student.set_lesson_left(u32::MAX);
        assert_eq!(student.lesson_left(), Some(u32::MAX));

        // 测试非常长的字符串
        let very_long_name = "a".repeat(1000);
        student.set_name(very_long_name.clone());
        assert_eq!(student.name(), very_long_name);
    }

    #[test]
    fn test_cash_extreme_amounts() {
        let _temp_dir = setup();

        // 测试极端金额
        let mut cash = Cash::new(Some(1));
        cash.set_cash(i64::MAX);
        assert_eq!(cash.cash, i64::MAX);

        cash.set_cash(i64::MIN);
        assert_eq!(cash.cash, i64::MIN);
    }

    #[test]
    fn test_uid_counter_edge_cases() {
        let _temp_dir = setup();

        // 创建包含非数字内容的UID计数器文件
        fs::write("./data/uid_counter", "not_a_number").unwrap();

        // 应该重置为默认值
        let student1 = Student::new();
        let student2 = Student::new();

        assert!(student1.uid() > 0);
        assert!(student2.uid() > student1.uid());
    }

    #[test]
    fn test_manager_invalid_paths() {
        let _temp_dir = setup();

        // 测试无效路径
        let result = QmxManager::from_path("/nonexistent/path", "/another/nonexistent", true);
        assert!(result.is_err());
    }

    #[test]
    fn test_serialization_with_special_characters() {
        let _temp_dir = setup();

        // 创建包含特殊字符的学生
        let mut student = Student::new();
        student.set_name("测试\n\t\"'\\学生".to_string());
        student.set_note("包含特殊字符的备注: {}[]()!@#$%^&*".to_string());
        student.set_phone("+86-138-0013-8000".to_string());

        let mut db = StudentDatabase::new();
        db.insert(student);

        // 测试序列化和反序列化
        let json_str = db.json();
        assert!(json_str.contains("测试"));
        assert!(json_str.contains("特殊字符"));

        // 测试从JSON恢复
        let new_db = StudentDatabase::from_json(&json_str).unwrap();
        assert_eq!(new_db.len(), 1);

        let restored_student = new_db.iter().next().unwrap().1;
        assert_eq!(restored_student.name(), "测试\n\t\"'\\学生");
        assert_eq!(
            restored_student.note(),
            "包含特殊字符的备注: {}[]()!@#$%^&*"
        );
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn test_large_student_creation() {
        let _temp_dir = setup();
        let manager = QmxManager::new(false).unwrap(); // 关闭自动保存提高性能

        let start_time = std::time::Instant::now();
        let mut student_uids = Vec::new();

        // 创建1000个学生
        for i in 0..1000 {
            let uid = manager
                .create_student(
                    StudentBuilder::new(
                        format!("Performance_Student_{}", i),
                        ((i % 80) + 10) as u8,
                    )
                    .lesson_left((i % 50) + 1),
                )
                .unwrap();
            student_uids.push(uid);
        }

        let creation_duration = start_time.elapsed();
        println!("创建1000个学生耗时: {:?}", creation_duration);

        // 性能基准：应该在合理时间内完成
        assert!(
            creation_duration.as_secs() < 5,
            "创建1000个学生应该在5秒内完成"
        );

        // 验证数据完整性
        let students = manager.list_students().unwrap();
        assert_eq!(students.len(), 1000);
    }

    #[test]
    fn test_batch_operations_performance() {
        let _temp_dir = setup();
        let mut db = StudentDatabase::new();

        // 创建大批量学生数据
        let students: Vec<Student> = (0..2000)
            .map(|i| {
                let mut student = Student::new();
                student.set_name(format!("Batch_Student_{}", i));
                student.set_age(((i % 100) + 10) as u8);
                student.set_lesson_left((i % 30) + 1);

                // 添加一些分数
                for j in 0..5 {
                    student.add_ring(((i + j) % 10) as f64 + 1.0);
                }

                student
            })
            .collect();

        // 测试批量插入性能
        let batch_start = std::time::Instant::now();
        db.insert_batch(students);
        let batch_duration = batch_start.elapsed();

        println!("批量插入2000个学生耗时: {:?}", batch_duration);
        assert_eq!(db.len(), 2000);

        // 测试查询性能
        let query_start = std::time::Instant::now();
        let filtered_students: Vec<_> = db
            .iter()
            .filter(|(_, s)| s.age() > 50 && s.lesson_left().unwrap_or(0) > 15)
            .collect();
        let query_duration = query_start.elapsed();

        println!(
            "复杂查询耗时: {:?}, 找到{}个学生",
            query_duration,
            filtered_students.len()
        );

        // 性能基准检查
        assert!(batch_duration.as_millis() < 2000, "批量插入应该在2秒内完成");
        assert!(query_duration.as_millis() < 100, "查询应该在100毫秒内完成");
    }

    #[test]
    fn test_serialization_performance() {
        let _temp_dir = setup();
        let mut db = StudentDatabase::new();

        // 创建大量复杂数据
        for i in 0..1000 {
            let mut student = Student::new();
            student.set_name(format!(
                "Serialization_Test_Student_{}_with_long_name_containing_unicode_测试",
                i
            ));
            student.set_age((i % 100) as u8);
            student.set_phone(format!(
                "+86-{:04}-{:04}-{:04}",
                i % 1000,
                (i * 2) % 10000,
                (i * 3) % 10000
            ));
            student.set_lesson_left((i % 100) + 1);
            student.set_note(format!(
                "详细备注信息包含特殊字符 !@#$%^&*()_+ 和中文 测试备注 {}",
                i
            ));

            // 添加大量分数
            for j in 0..10 {
                student.add_ring(((i + j) as f64 % 100.0) / 10.0);
            }

            // 设置会员信息
            student.set_membership_start_date(Utc::now());
            student.set_membership_end_date(Utc::now() + Duration::days(365));

            db.insert(student);
        }

        println!("准备序列化{}个复杂学生记录", db.len());

        // 测试序列化性能
        let serialize_start = std::time::Instant::now();
        let json_data = db.json();
        let serialize_duration = serialize_start.elapsed();

        let json_size = json_data.len();
        println!("序列化耗时: {:?}", serialize_duration);
        println!(
            "JSON大小: {} bytes ({:.2} MB)",
            json_size,
            json_size as f64 / 1024.0 / 1024.0
        );

        // 测试反序列化性能
        let deserialize_start = std::time::Instant::now();
        let new_db = StudentDatabase::from_json(&json_data).unwrap();
        let deserialize_duration = deserialize_start.elapsed();

        println!("反序列化耗时: {:?}", deserialize_duration);
        assert_eq!(new_db.len(), db.len());

        // 验证数据完整性
        for (uid, original_student) in db.iter() {
            let loaded_student = new_db.get(uid).unwrap();
            assert_eq!(original_student.name(), loaded_student.name());
            assert_eq!(original_student.age(), loaded_student.age());
            assert_eq!(original_student.rings().len(), loaded_student.rings().len());
        }

        // 性能基准
        assert!(serialize_duration.as_secs() < 3, "序列化应该在3秒内完成");
        assert!(
            deserialize_duration.as_secs() < 3,
            "反序列化应该在3秒内完成"
        );
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_complete_workflow() {
        let _temp_dir = setup();
        let manager = QmxManager::new(true).unwrap();

        // 1. 创建学生
        let student_uid = manager
            .create_student(
                StudentBuilder::new("张三", 25)
                    .phone("138-0013-8000")
                    .class(Class::Month)
                    .subject(Subject::Archery)
                    .lesson_left(20)
                    .note("优秀学员"),
            )
            .unwrap();

        // 2. 验证学生创建
        let stats = manager.get_dashboard_stats().unwrap();
        assert_eq!(stats.total_students, 1);

        // 3. 更新学生信息 - 先改为TenTry class才能设置lesson_left
        let update_result = manager.update_student(
            student_uid,
            StudentUpdater::new()
                .age(26)
                .class(Class::TenTry) // 设置为TenTry才能有lesson_left
                .add_ring(8.5)
                .add_ring(9.0)
                .add_ring(7.5),
        );
        assert!(update_result.is_ok());

        // 再次更新设置lesson_left
        let lesson_update =
            manager.update_student(student_uid, StudentUpdater::new().lesson_left(Some(18)));
        assert!(lesson_update.is_ok());

        // 4. 添加现金收入记录
        let income_uid = manager
            .record_cash(
                CashBuilder::new(1200)
                    .student_id(student_uid)
                    .note("月费缴费"),
            )
            .unwrap();

        // 5. 验证现金记录
        let cash_record = manager.get_cash(income_uid).unwrap().unwrap();
        assert_eq!(cash_record.cash, 1200);
        assert_eq!(cash_record.student_id, Some(student_uid));

        // 6. 获取综合统计
        let dashboard_stats = manager.get_dashboard_stats().unwrap();
        assert_eq!(dashboard_stats.total_students, 1);
        assert!(dashboard_stats.total_revenue > 0);
        assert!(dashboard_stats.average_score > 0.0);

        // 7. 验证学生更新
        let updated_student = manager.get_student(student_uid).unwrap().unwrap();
        assert_eq!(updated_student.age(), 26);
        assert_eq!(updated_student.class(), &Class::TenTry);
        assert_eq!(updated_student.lesson_left(), Some(18));
        assert_eq!(updated_student.rings().len(), 3);
    }

    #[test]
    fn test_multi_student_scenario() {
        let _temp_dir = setup();
        let manager = QmxManager::new(true).unwrap();

        // 创建多个学生
        let mut student_uids = Vec::new();

        let student_configs = vec![
            ("李四", 20, Class::TenTry, Subject::Shooting, 10),
            ("王五", 35, Class::Month, Subject::Archery, 25),
            ("赵六", 28, Class::Year, Subject::Shooting, 50),
            ("钱七", 42, Class::Others, Subject::Others, 5),
        ];

        for (name, age, class, subject, lessons) in student_configs {
            let uid = manager
                .create_student(
                    StudentBuilder::new(name, age)
                        .class(class)
                        .subject(subject)
                        .lesson_left(lessons),
                )
                .unwrap();
            student_uids.push(uid);
        }

        // 为每个学生添加成绩和财务记录
        for (i, &uid) in student_uids.iter().enumerate() {
            // 添加成绩
            let scores = match i {
                0 => vec![6.0, 7.0, 8.0],
                1 => vec![8.5, 9.0, 9.5],
                2 => vec![7.5, 8.0, 8.5, 9.0],
                3 => vec![5.0, 6.0],
                _ => vec![],
            };

            let mut updater = StudentUpdater::new();
            for score in scores {
                updater = updater.add_ring(score);
            }
            manager.update_student(uid, updater).unwrap();

            // 添加财务记录
            let amount = ((i + 1) * 500) as i64;
            manager
                .record_cash(
                    CashBuilder::new(amount)
                        .student_id(uid)
                        .note(format!("学员{}缴费", i + 1)),
                )
                .unwrap();
        }

        // 验证统计数据
        let stats = manager.get_dashboard_stats().unwrap();
        assert_eq!(stats.total_students, 4);
        assert!(stats.total_revenue > 0);
        assert!(stats.average_score > 0.0);

        // 测试查询功能
        let all_students = manager.list_students().unwrap();
        assert_eq!(all_students.len(), 4);

        // 验证每个学生的数据
        for &uid in &student_uids {
            let student = manager.get_student(uid).unwrap().unwrap();
            assert!(!student.name().is_empty());
            assert!(student.age() > 0);
        }
    }

    #[test]
    fn test_concurrent_operations() {
        let _temp_dir = setup();
        use std::sync::Arc;
        use std::thread;

        let manager = Arc::new(QmxManager::new(true).unwrap());
        let mut handles = Vec::new();

        // 并发创建学生
        for i in 0..10 {
            let manager_clone = Arc::clone(&manager);
            let handle = thread::spawn(move || {
                let result = manager_clone.create_student(StudentBuilder::new(
                    format!("并发学生_{}", i),
                    20 + (i % 50) as u8,
                ));
                result.unwrap()
            });
            handles.push(handle);
        }

        // 等待所有线程完成
        let mut student_uids = Vec::new();
        for handle in handles {
            let uid = handle.join().unwrap();
            student_uids.push(uid);
        }

        // 验证所有学生都被创建
        assert_eq!(student_uids.len(), 10);
        let stats = manager.get_dashboard_stats().unwrap();
        assert_eq!(stats.total_students, 10);

        // 并发更新学生 - 先设置class为TenTry
        let mut update_handles = Vec::new();
        for (i, &uid) in student_uids.iter().enumerate() {
            let manager_clone = Arc::clone(&manager);
            let handle = thread::spawn(move || {
                // 先设置为TenTry class
                manager_clone
                    .update_student(
                        uid,
                        StudentUpdater::new()
                            .class(Class::TenTry)
                            .add_ring((i as f64) * 0.5 + 7.0),
                    )
                    .unwrap();

                // 然后设置lesson_left
                manager_clone
                    .update_student(uid, StudentUpdater::new().lesson_left(Some(30 + i as u32)))
                    .unwrap()
            });
            update_handles.push(handle);
        }

        // 等待所有更新完成
        for handle in update_handles {
            handle.join().unwrap();
        }

        // 验证更新结果
        for (i, &uid) in student_uids.iter().enumerate() {
            let student = manager.get_student(uid).unwrap().unwrap();
            assert_eq!(student.lesson_left(), Some(30 + i as u32));
            assert!(!student.rings().is_empty());
        }

        // 验证最终状态
        let final_stats = manager.get_dashboard_stats().unwrap();
        assert_eq!(final_stats.total_students, 10);
        assert!(final_stats.average_score > 0.0);
    }
}

#[cfg(test)]
mod stress_tests {
    use super::*;

    #[test]
    fn test_memory_usage_stability() {
        let _temp_dir = setup();
        let mut db = CashDatabase::new();

        // 反复插入和删除数据，检查内存是否稳定
        for cycle in 0..5 {
            let mut cash_records = Vec::new();

            // 插入500条记录
            for i in 0..500 {
                let mut cash = Cash::new_installment(
                    Some((cycle * 500 + i) as u64),
                    (i * 100) as i64,
                    12,
                    PaymentFrequency::Monthly,
                    Utc::now(),
                    cycle + 1,
                    None,
                );
                cash.set_note(Some(format!("Cycle {} Record {}", cycle, i)));
                cash_records.push(cash.uid);
                db.insert(cash);
            }

            assert_eq!(db.len(), 500);

            // 删除所有记录
            for uid in cash_records {
                db.remove(&uid);
            }

            assert_eq!(db.len(), 0);
        }

        // 最终数据库应该是空的
        assert!(db.is_empty());
    }

    #[test]
    fn test_large_dataset_queries() {
        let _temp_dir = setup();
        let manager = QmxManager::new(false).unwrap();

        // 创建大量测试数据
        let mut student_uids = Vec::new();

        for i in 0..500 {
            let uid = manager
                .create_student(
                    StudentBuilder::new(format!("Query_Test_Student_{}", i), ((i % 80) + 10) as u8)
                        .class(match i % 4 {
                            0 => Class::TenTry,
                            1 => Class::Month,
                            2 => Class::Year,
                            _ => Class::Others,
                        })
                        .subject(match i % 3 {
                            0 => Subject::Shooting,
                            1 => Subject::Archery,
                            _ => Subject::Others,
                        })
                        .lesson_left((i % 50) + 1),
                )
                .unwrap();
            student_uids.push(uid);

            // 为每个学生添加分数
            let score_count = (i % 10) + 1;
            let mut updater = StudentUpdater::new();
            for j in 0..score_count {
                let score = ((i + j) % 100) as f64 / 10.0;
                updater = updater.add_ring(score);
            }
            manager.update_student(uid, updater).unwrap();

            // 添加现金记录
            let amount = ((i % 10) + 1) * 100;
            manager
                .record_cash(
                    CashBuilder::new(amount as i64)
                        .student_id(uid)
                        .note(format!("收入记录 {}", i)),
                )
                .unwrap();
        }

        // 测试统计计算性能
        let stats_start = std::time::Instant::now();
        let dashboard_stats = manager.get_dashboard_stats().unwrap();
        let stats_duration = stats_start.elapsed();

        println!("计算500学生统计耗时: {:?}", stats_duration);
        println!(
            "统计结果: {} 学生, 总收入 {}, 平均分 {:.2}",
            dashboard_stats.total_students,
            dashboard_stats.total_revenue,
            dashboard_stats.average_score
        );

        // 验证统计正确性
        assert_eq!(dashboard_stats.total_students, 500);
        assert!(dashboard_stats.total_revenue > 0);
        assert!(dashboard_stats.average_score >= 0.0);

        // 性能基准
        assert!(stats_duration.as_millis() < 1000, "统计计算应该在1秒内完成");
    }
}
