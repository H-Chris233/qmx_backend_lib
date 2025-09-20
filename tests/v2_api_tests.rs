// V2 API 测试集合
// 包含所有使用新 QmxManager API 的测试

use chrono::{Duration, Utc};
use qmx_backend_lib::student::{Class, Subject};
use qmx_backend_lib::{
    CashBuilder, CashQuery, CashUpdater, MembershipStatus, QmxManager, StudentBuilder,
    StudentQuery, StudentUpdater, TimePeriod,
};
use tempfile::TempDir;

mod qmx_manager_tests {
    use super::*;

    #[test]
    fn test_manager_initialization() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        std::env::set_current_dir(temp_path).unwrap();

        // 确保data目录存在
        let _ = std::fs::create_dir_all("data");

        let manager = QmxManager::new(true).unwrap();

        // 测试基本功能
        let students = manager.list_students().unwrap();
        assert_eq!(students.len(), 0);

        // 测试手动保存
        manager.save().unwrap();
    }

    #[test]
    fn test_manager_from_path() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        std::env::set_current_dir(temp_path).unwrap();

        // 确保data目录存在
        let _ = std::fs::create_dir_all("data");

        // 先创建一些数据文件
        let initial_manager = QmxManager::new(true).unwrap();
        let _student_id = initial_manager
            .create_student(StudentBuilder::new("初始学生").age(18).class(Class::TenTry))
            .unwrap();

        // 从路径加载
        let manager = QmxManager::from_path(
            "./data/student_database.json",
            "./data/cash_database.json",
            false,
        )
        .unwrap();

        let students = manager.list_students().unwrap();
        assert_eq!(students.len(), 1);
        assert_eq!(students[0].name(), "初始学生");
    }
}

mod student_builder_tests {
    use super::*;

    #[test]
    fn test_student_builder_basic() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        std::env::set_current_dir(temp_path).unwrap();

        // 确保data目录存在
        let _ = std::fs::create_dir_all("data");

        let manager = QmxManager::new(true).unwrap();

        let student_id = manager
            .create_student(
                StudentBuilder::new("张三").age(16)
                    .phone("13800138000")
                    .class(Class::TenTry)
                    .subject(Subject::Shooting)
                    .note("优秀学生"),
            )
            .unwrap();

        let student = manager.get_student(student_id).unwrap().unwrap();
        assert_eq!(student.name(), "张三");
        assert_eq!(student.age(), Some(16));
        assert_eq!(student.phone(), "13800138000");
        assert_eq!(student.class(), &Class::TenTry);
        assert_eq!(student.subject(), &Subject::Shooting);
        assert_eq!(student.note(), "优秀学生");
    }

    #[test]
    fn test_student_builder_with_membership() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        std::env::set_current_dir(temp_path).unwrap();

        // 确保data目录存在
        let _ = std::fs::create_dir_all("data");

        let manager = QmxManager::new(true).unwrap();

        let start = Utc::now();
        let end = start + Duration::days(365);

        let student_id = manager
            .create_student(StudentBuilder::new("会员学生").age(20).membership(start, end))
            .unwrap();

        let student = manager.get_student(student_id).unwrap().unwrap();
        assert_eq!(student.membership_start_date(), Some(start));
        assert_eq!(student.membership_end_date(), Some(end));
        assert!(student.is_membership_active());
    }

    #[test]
    fn test_student_builder_minimal() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        std::env::set_current_dir(temp_path).unwrap();

        // 确保data目录存在
        let _ = std::fs::create_dir_all("data");

        let manager = QmxManager::new(true).unwrap();

        let student_id = manager
            .create_student(StudentBuilder::new("最小学生").age(15))
            .unwrap();

        let student = manager.get_student(student_id).unwrap().unwrap();
        assert_eq!(student.name(), "最小学生");
        assert_eq!(student.age(), Some(15));
        assert_eq!(student.phone(), "未填写");
        assert_eq!(student.class(), &Class::Others);
        assert_eq!(student.subject(), &Subject::Others);
    }
}

mod student_updater_tests {
    use super::*;

    #[test]
    fn test_student_updater_basic() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        std::env::set_current_dir(temp_path).unwrap();

        // 确保data目录存在
        let _ = std::fs::create_dir_all("data");

        let manager = QmxManager::new(true).unwrap();

        let student_id = manager
            .create_student(StudentBuilder::new("更新测试").age(16))
            .unwrap();

        manager
            .update_student(
                student_id,
                StudentUpdater::new()
                                        .age(Some(17))
                    .phone("新电话")
                    .class(Class::Month)
                    .note("更新后的备注"),
            )
            .unwrap();

        let student = manager.get_student(student_id).unwrap().unwrap();
        assert_eq!(student.age(), Some(17));
        assert_eq!(student.phone(), "新电话");
        assert_eq!(student.class(), &Class::Month);
        assert_eq!(student.note(), "更新后的备注");
    }

    #[test]
    fn test_student_updater_rings() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        std::env::set_current_dir(temp_path).unwrap();

        // 确保data目录存在
        let _ = std::fs::create_dir_all("data");

        let manager = QmxManager::new(true).unwrap();

        let student_id = manager
            .create_student(StudentBuilder::new("成绩测试").age(18))
            .unwrap();

        // 添加单个成绩
        manager
            .update_student(student_id, StudentUpdater::new().add_ring(85.5))
            .unwrap();

        // 替换所有成绩
        manager
            .update_student(
                student_id,
                StudentUpdater::new().set_rings(vec![90.0, 88.5, 92.0]),
            )
            .unwrap();

        let student = manager.get_student(student_id).unwrap().unwrap();
        assert_eq!(student.rings().len(), 3);
        assert_eq!(student.rings(), &[90.0, 88.5, 92.0]);

        manager
            .update_student(student_id, StudentUpdater::new().update_ring_at(1, 91.0))
            .unwrap();
        let student = manager.get_student(student_id).unwrap().unwrap();
        assert_eq!(student.rings(), &[90.0, 91.0, 92.0]);

        manager
            .update_student(student_id, StudentUpdater::new().remove_ring_at(0))
            .unwrap();
        let student = manager.get_student(student_id).unwrap().unwrap();
        assert_eq!(student.rings(), &[91.0, 92.0]);
    }

    #[test]
    fn test_student_updater_membership() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        std::env::set_current_dir(temp_path).unwrap();

        // 确保data目录存在
        let _ = std::fs::create_dir_all("data");

        let manager = QmxManager::new(true).unwrap();

        let student_id = manager
            .create_student(StudentBuilder::new("会员更新").age(19))
            .unwrap();

        let start = Utc::now();
        let end = start + Duration::days(30);

        manager
            .update_student(
                student_id,
                StudentUpdater::new().membership(Some(start), Some(end)),
            )
            .unwrap();

        let student = manager.get_student(student_id).unwrap().unwrap();
        assert_eq!(student.membership_start_date(), Some(start));
        assert_eq!(student.membership_end_date(), Some(end));
    }
}

mod cash_builder_tests {
    use super::*;

    #[test]
    fn test_cash_builder_basic() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        std::env::set_current_dir(temp_path).unwrap();

        // 确保data目录存在
        let _ = std::fs::create_dir_all("data");

        let manager = QmxManager::new(true).unwrap();

        let student_id = manager
            .create_student(StudentBuilder::new("现金测试").age(18))
            .unwrap();

        let cash_id = manager
            .record_cash(
                CashBuilder::new(1500)
                    .student_id(student_id)
                    .note("学费收入"),
            )
            .unwrap();

        let cash = manager.get_cash(cash_id).unwrap().unwrap();
        assert_eq!(cash.cash, 1500);
        assert_eq!(cash.student_id, Some(student_id));
        assert_eq!(cash.note(), Some("学费收入"));
    }

    #[test]
    fn test_cash_builder_expense() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        std::env::set_current_dir(temp_path).unwrap();

        // 确保data目录存在
        let _ = std::fs::create_dir_all("data");

        let manager = QmxManager::new(true).unwrap();

        let cash_id = manager
            .record_cash(CashBuilder::new(-200).note("设备采购"))
            .unwrap();

        let cash = manager.get_cash(cash_id).unwrap().unwrap();
        assert_eq!(cash.cash, -200);
        assert_eq!(cash.student_id, None);
        assert_eq!(cash.note(), Some("设备采购"));
    }
}

mod student_query_tests {
    use super::*;

    #[test]
    fn test_student_query_age_range() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        std::env::set_current_dir(temp_path).unwrap();

        // 确保data目录存在
        let _ = std::fs::create_dir_all("data");

        let manager = QmxManager::new(true).unwrap();

        // 创建不同年龄的学生
        let _id1 = manager
            .create_student(StudentBuilder::new("学生1").age(15))
            .unwrap();
        let _id2 = manager
            .create_student(StudentBuilder::new("学生2").age(18))
            .unwrap();
        let _id3 = manager
            .create_student(StudentBuilder::new("学生3").age(22))
            .unwrap();
        let _id4 = manager
            .create_student(StudentBuilder::new("学生4")) // 年龄为空的学生
            .unwrap();

        // 查询年龄在16-20之间的学生
        let students = manager
            .search_students(StudentQuery::new().age_range(16, 20))
            .unwrap();
        assert_eq!(students.len(), 1);
        assert_eq!(students[0].name(), "学生2");
        assert_eq!(students[0].age(), Some(18));

        // 查询年龄在15-22之间的学生（应该包含3个有年龄的学生）
        let students = manager
            .search_students(StudentQuery::new().age_range(15, 22))
            .unwrap();
        assert_eq!(students.len(), 3); // 不包括年龄为空的学生
    }

    #[test]
    fn test_student_query_class_and_subject() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        std::env::set_current_dir(temp_path).unwrap();

        // 确保data目录存在
        let _ = std::fs::create_dir_all("data");

        let manager = QmxManager::new(true).unwrap();

        manager
            .create_student(
                StudentBuilder::new("TenTry射击").age(16)
                    .class(Class::TenTry)
                    .subject(Subject::Shooting),
            )
            .unwrap();

        manager
            .create_student(
                StudentBuilder::new("Month射箭").age(17)
                    .class(Class::Month)
                    .subject(Subject::Archery),
            )
            .unwrap();

        manager
            .create_student(
                StudentBuilder::new("TenTry射箭").age(18)
                    .class(Class::TenTry)
                    .subject(Subject::Archery),
            )
            .unwrap();

        let tentry_students = manager
            .search_students(StudentQuery::new().class(Class::TenTry))
            .unwrap();
        assert_eq!(tentry_students.len(), 2);

        let archery_students = manager
            .search_students(StudentQuery::new().subject(Subject::Archery))
            .unwrap();
        assert_eq!(archery_students.len(), 2);

        let tentry_archery = manager
            .search_students(
                StudentQuery::new()
                    .class(Class::TenTry)
                    .subject(Subject::Archery),
            )
            .unwrap();
        assert_eq!(tentry_archery.len(), 1);
        assert_eq!(tentry_archery[0].name(), "TenTry射箭");
    }

    #[test]
    fn test_student_query_membership() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        std::env::set_current_dir(temp_path).unwrap();

        // 确保data目录存在
        let _ = std::fs::create_dir_all("data");

        let manager = QmxManager::new(true).unwrap();

        let start = Utc::now();
        let end = start + Duration::days(30);

        // 有会员的学生
        manager
            .create_student(StudentBuilder::new("会员学生").age(18).membership(start, end))
            .unwrap();

        // 无会员的学生
        manager
            .create_student(StudentBuilder::new("普通学生").age(19))
            .unwrap();

        let members = manager
            .search_students(StudentQuery::new().has_membership(true))
            .unwrap();
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].name(), "会员学生");

        let non_members = manager
            .search_students(StudentQuery::new().has_membership(false))
            .unwrap();
        assert_eq!(non_members.len(), 1);
        assert_eq!(non_members[0].name(), "普通学生");

        let active_members = manager
            .search_students(StudentQuery::new().membership_active_at(Utc::now()))
            .unwrap();
        assert_eq!(active_members.len(), 1);
        assert_eq!(active_members[0].name(), "会员学生");
    }
}

mod cash_query_tests {
    use super::*;

    #[test]
    fn test_cash_query_student_id() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        std::env::set_current_dir(temp_path).unwrap();

        // 确保data目录存在
        let _ = std::fs::create_dir_all("data");

        let manager = QmxManager::new(true).unwrap();

        let student1_id = manager
            .create_student(StudentBuilder::new("学生1").age(18))
            .unwrap();
        let student2_id = manager
            .create_student(StudentBuilder::new("学生2").age(19))
            .unwrap();

        manager
            .record_cash(CashBuilder::new(1000).student_id(student1_id))
            .unwrap();
        manager
            .record_cash(CashBuilder::new(1500).student_id(student1_id))
            .unwrap();
        manager
            .record_cash(CashBuilder::new(2000).student_id(student2_id))
            .unwrap();

        let student1_cash = manager
            .search_cash(CashQuery::new().student_id(student1_id))
            .unwrap();
        assert_eq!(student1_cash.len(), 2);

        let student2_cash = manager
            .search_cash(CashQuery::new().student_id(student2_id))
            .unwrap();
        assert_eq!(student2_cash.len(), 1);
        assert_eq!(student2_cash[0].cash, 2000);
    }

    #[test]
    fn test_cash_query_amount_range() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        std::env::set_current_dir(temp_path).unwrap();

        // 确保data目录存在
        let _ = std::fs::create_dir_all("data");

        let manager = QmxManager::new(true).unwrap();

        manager.record_cash(CashBuilder::new(500)).unwrap();
        manager.record_cash(CashBuilder::new(1500)).unwrap();
        manager.record_cash(CashBuilder::new(2500)).unwrap();
        manager.record_cash(CashBuilder::new(-200)).unwrap();

        let medium_amounts = manager
            .search_cash(CashQuery::new().amount_range(1000, 2000))
            .unwrap();
        assert_eq!(medium_amounts.len(), 1);
        assert_eq!(medium_amounts[0].cash, 1500);

        let positive_amounts = manager
            .search_cash(CashQuery::new().amount_range(0, i64::MAX))
            .unwrap();
        assert_eq!(positive_amounts.len(), 3);
    }
}

mod statistics_tests {
    use super::*;

    #[test]
    fn test_dashboard_stats_v2() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        std::env::set_current_dir(temp_path).unwrap();

        // 确保data目录存在
        let _ = std::fs::create_dir_all("data");

        let manager = QmxManager::new(true).unwrap();

        // 创建学生和现金记录
        let student_id = manager
            .create_student(StudentBuilder::new("统计学生").age(18))
            .unwrap();

        manager
            .update_student(
                student_id,
                StudentUpdater::new()
                    .add_ring(85.0)
                    .add_ring(90.0)
                    .add_ring(88.0),
            )
            .unwrap();

        manager
            .record_cash(CashBuilder::new(2000).student_id(student_id))
            .unwrap();
        manager.record_cash(CashBuilder::new(-300)).unwrap();

        let stats = manager.get_dashboard_stats().unwrap();
        assert_eq!(stats.total_students, 1);
        assert_eq!(stats.total_revenue, 2000);
        assert_eq!(stats.total_expense, 300);
        assert!((stats.average_score - 87.67).abs() < 0.1);
        assert_eq!(stats.max_score, 90.0);
    }

    #[test]
    fn test_student_stats() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        std::env::set_current_dir(temp_path).unwrap();

        // 确保data目录存在
        let _ = std::fs::create_dir_all("data");

        let manager = QmxManager::new(true).unwrap();

        let start = Utc::now();
        let end = start + Duration::days(30);

        let student_id = manager
            .create_student(StudentBuilder::new("详细统计").age(20).membership(start, end))
            .unwrap();

        manager
            .update_student(
                student_id,
                StudentUpdater::new().add_ring(92.0).add_ring(88.5),
            )
            .unwrap();

        manager
            .record_cash(CashBuilder::new(1500).student_id(student_id))
            .unwrap();
        manager
            .record_cash(CashBuilder::new(800).student_id(student_id))
            .unwrap();

        let stats = manager.get_student_stats(student_id).unwrap();
        assert_eq!(stats.total_payments, 2300);
        assert_eq!(stats.payment_count, 2);
        assert_eq!(stats.score_count, 2);
        assert!((stats.average_score.unwrap() - 90.25).abs() < 0.01);

        match stats.membership_status {
            MembershipStatus::Active { expires_at } => {
                assert_eq!(expires_at, end);
            }
            _ => panic!("Expected active membership"),
        }
    }

    #[test]
    fn test_financial_stats() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        std::env::set_current_dir(temp_path).unwrap();

        // 确保data目录存在
        let _ = std::fs::create_dir_all("data");

        let manager = QmxManager::new(true).unwrap();

        manager.record_cash(CashBuilder::new(2000)).unwrap();
        manager.record_cash(CashBuilder::new(1500)).unwrap();
        manager.record_cash(CashBuilder::new(-300)).unwrap();
        manager.record_cash(CashBuilder::new(-150)).unwrap();

        let stats = manager.get_financial_stats(TimePeriod::ThisMonth).unwrap();
        assert_eq!(stats.total_income, 3500);
        assert_eq!(stats.total_expense, 450);
        assert_eq!(stats.net_income, 3050);
        assert_eq!(stats.transaction_count, 4);
        assert_eq!(stats.installment_count, 0);
    }
}

mod crud_operations_tests {
    use super::*;

    #[test]
    fn test_student_crud_operations() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        std::env::set_current_dir(temp_path).unwrap();

        // 确保data目录存在
        let _ = std::fs::create_dir_all("data");

        let manager = QmxManager::new(true).unwrap();

        // Create
        let student_id = manager
            .create_student(StudentBuilder::new("CRUD测试").age(18).class(Class::TenTry))
            .unwrap();

        // Read
        let student = manager.get_student(student_id).unwrap().unwrap();
        assert_eq!(student.name(), "CRUD测试");

        // Update
        manager
            .update_student(
                student_id,
                StudentUpdater::new().name("更新后的名字").age(Some(19)),
            )
            .unwrap();

        let updated_student = manager.get_student(student_id).unwrap().unwrap();
        assert_eq!(updated_student.name(), "更新后的名字");
        assert_eq!(updated_student.age(), Some(19));

        // Delete
        let deleted = manager.delete_student(student_id).unwrap();
        assert!(deleted);

        let not_found = manager.get_student(student_id).unwrap();
        assert!(not_found.is_none());

        // Delete non-existent
        let not_deleted = manager.delete_student(student_id).unwrap();
        assert!(!not_deleted);
    }

    #[test]
    fn test_cash_crud_operations() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        std::env::set_current_dir(temp_path).unwrap();

        // 确保data目录存在
        let _ = std::fs::create_dir_all("data");

        let manager = QmxManager::new(true).unwrap();

        // Create
        let cash_id = manager
            .record_cash(CashBuilder::new(1000).note("CRUD测试"))
            .unwrap();

        // Read
        let cash = manager.get_cash(cash_id).unwrap().unwrap();
        assert_eq!(cash.cash, 1000);
        assert_eq!(cash.note(), Some("CRUD测试"));

        // Update
        manager
            .update_cash(
                cash_id,
                CashUpdater::new()
                    .amount(1500)
                    .note(Some("更新后的备注".to_string())),
            )
            .unwrap();

        let updated_cash = manager.get_cash(cash_id).unwrap().unwrap();
        assert_eq!(updated_cash.cash, 1500);
        assert_eq!(updated_cash.note(), Some("更新后的备注"));

        // Delete
        let deleted = manager.delete_cash(cash_id).unwrap();
        assert!(deleted);

        let not_found = manager.get_cash(cash_id).unwrap();
        assert!(not_found.is_none());
    }
}

#[test]
fn test_v2_api_integration() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    std::env::set_current_dir(temp_path).unwrap();

    // 完整的 v2 API 工作流程
    let manager = QmxManager::new(true).unwrap();

    // 1. 创建学生（使用构建器）
    let student_id = manager
        .create_student(
            StudentBuilder::new("集成测试学生").age(19)
                .phone("13800138000")
                .class(Class::Month)
                .subject(Subject::Archery)
                .note("v2 API集成测试")
                .membership(Utc::now(), Utc::now() + Duration::days(30)),
        )
        .unwrap();

    // 2. 更新学生信息（使用更新器）
    manager
        .update_student(
            student_id,
            StudentUpdater::new()
                .add_ring(88.0)
                .add_ring(91.5)
                .add_ring(89.0),
        )
        .unwrap();

    // 3. 记录现金流（使用构建器）
    let _cash_id = manager
        .record_cash(
            CashBuilder::new(2500)
                .student_id(student_id)
                .note("月卡费用"),
        )
        .unwrap();

    // 4. 查询操作（使用查询构建器）
    let month_students = manager
        .search_students(StudentQuery::new().class(Class::Month).has_membership(true))
        .unwrap();
    assert_eq!(month_students.len(), 1);
    assert_eq!(month_students[0].name(), "集成测试学生");

    let student_cash = manager
        .search_cash(CashQuery::new().student_id(student_id))
        .unwrap();
    assert_eq!(student_cash.len(), 1);
    assert_eq!(student_cash[0].cash, 2500);

    // 5. 统计分析
    let dashboard_stats = manager.get_dashboard_stats().unwrap();
    assert_eq!(dashboard_stats.total_students, 1);
    assert_eq!(dashboard_stats.total_revenue, 2500);

    let student_stats = manager.get_student_stats(student_id).unwrap();
    assert_eq!(student_stats.total_payments, 2500);
    assert_eq!(student_stats.score_count, 3);
    assert!((student_stats.average_score.unwrap() - 89.5).abs() < 0.1);

    // 6. 验证数据持久化（自动保存已启用）
    let new_manager = QmxManager::new(false).unwrap();
    let reloaded_students = new_manager.list_students().unwrap();
    assert_eq!(reloaded_students.len(), 1);
    assert_eq!(reloaded_students[0].name(), "集成测试学生");
}
