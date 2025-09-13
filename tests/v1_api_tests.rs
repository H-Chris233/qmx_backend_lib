// V1 API 测试集合
// 包含所有使用原始 v1 API 的测试

use chrono::{Duration, Utc};
use qmx_backend_lib::cash::{Cash, CashDatabase};
use qmx_backend_lib::student::{Class, Student, StudentDatabase, Subject};
use qmx_backend_lib::*;
use tempfile::TempDir;

// 重新导出所有 v1 API 测试模块
mod student_v1_tests {
    use super::*;

    #[test]
    fn test_student_creation() {
        let mut student = Student::new();
        student
            .set_name("张三".to_string())
            .set_age(18)
            .set_class(Class::TenTry)
            .set_subject(Subject::Shooting);

        assert_eq!(student.name(), "张三");
        assert_eq!(student.age(), 18);
        assert_eq!(student.class(), &Class::TenTry);
        assert_eq!(student.subject(), &Subject::Shooting);
    }

    #[test]
    fn test_student_database_operations() {
        let mut db = StudentDatabase::new();

        let mut student = Student::new();
        student.set_name("李四".to_string()).set_age(20);
        let uid = student.uid();

        db.insert(student);
        assert_eq!(db.len(), 1);

        let retrieved = db.get(&uid).unwrap();
        assert_eq!(retrieved.name(), "李四");
        assert_eq!(retrieved.age(), 20);

        let removed = db.remove(&uid);
        assert!(removed.is_some());
        assert_eq!(db.len(), 0);
    }

    #[test]
    fn test_membership_management() {
        let mut student = Student::new();
        let start = Utc::now();
        let end = start + Duration::days(30);

        student.set_membership_dates(Some(start), Some(end));

        assert!(student.is_membership_active());
        assert_eq!(student.membership_start_date(), Some(start));
        assert_eq!(student.membership_end_date(), Some(end));

        let days_remaining = student.membership_days_remaining();
        assert!(days_remaining.is_some());
        assert!(days_remaining.unwrap() > 0);
    }
}

mod cash_v1_tests {
    use super::*;

    #[test]
    fn test_cash_creation() {
        let mut cash = Cash::new(Some(1));
        cash.set_cash(1000);
        cash.set_note(Some("学费收入".to_string()));

        assert_eq!(cash.student_id, Some(1));
        assert_eq!(cash.cash, 1000);
        assert_eq!(cash.note(), Some("学费收入"));
    }

    #[test]
    fn test_cash_database_operations() {
        let mut db = CashDatabase::new();

        let mut cash = Cash::new(Some(1));
        cash.set_cash(2000);
        let uid = cash.uid;

        db.insert(cash);
        assert_eq!(db.len(), 1);

        let retrieved = db.get(&uid).unwrap();
        assert_eq!(retrieved.cash, 2000);
        assert_eq!(retrieved.student_id, Some(1));

        let removed = db.remove(&uid);
        assert!(removed.is_some());
        assert_eq!(db.len(), 0);
    }

    #[test]
    fn test_installment_management() {
        use qmx_backend_lib::cash::PaymentFrequency;

        let mut db = CashDatabase::new();

        // 创建分期付款
        let installment = Cash::new_installment(
            Some(1),
            6000,
            3,
            PaymentFrequency::Monthly,
            Utc::now() + Duration::days(30),
            1,
            None,
        );

        let plan_id = installment.installment_plan_id().unwrap();
        db.insert(installment);

        // 生成下一期
        let next_due = Utc::now() + Duration::days(60);
        let next_uid = db.generate_next_installment(plan_id, next_due).unwrap();

        assert_eq!(db.len(), 2);

        let next_installment = db.get(&next_uid).unwrap();
        assert_eq!(
            next_installment
                .installment
                .as_ref()
                .unwrap()
                .current_installment,
            2
        );

        // 取消分期计划
        let cancelled = db.cancel_installment_plan(plan_id);
        assert_eq!(cancelled, 2);
    }
}

mod database_v1_tests {
    use super::*;

    #[test]
    fn test_database_initialization() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // 直接创建数据库实例
        let db = database::init().unwrap();
        assert_eq!(db.student.len(), 0);
        assert_eq!(db.cash.len(), 0);

        // 使用自定义路径保存 - 确保目录存在
        let student_path = temp_path.join("student.json");
        let cash_path = temp_path.join("cash.json");

        if let Some(parent) = student_path.parent() {
            std::fs::create_dir_all(parent).unwrap_or(());
        }
        if let Some(parent) = cash_path.parent() {
            std::fs::create_dir_all(parent).unwrap_or(());
        }

        db.student.save_to(student_path.to_str().unwrap()).unwrap();
        db.cash.save_to(cash_path.to_str().unwrap()).unwrap();

        // 验证文件存在
        assert!(temp_path.join("student.json").exists());
        assert!(temp_path.join("cash.json").exists());
    }

    #[test]
    fn test_database_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        let student_path = temp_path.join("student.json");
        let cash_path = temp_path.join("cash.json");

        // 创建并保存数据
        {
            let mut db = database::init().unwrap();

            let mut student = Student::new();
            student.set_name("测试学生".to_string());
            db.student.insert(student);

            let mut cash = Cash::new(None);
            cash.set_cash(500);
            db.cash.insert(cash);

            // 使用自定义路径保存
            db.student.save_to(student_path.to_str().unwrap()).unwrap();
            db.cash.save_to(cash_path.to_str().unwrap()).unwrap();
        }

        // 从自定义路径重新加载并验证
        {
            let student_db = StudentDatabase::read_from(student_path.to_str().unwrap()).unwrap();
            let cash_db = CashDatabase::read_from(cash_path.to_str().unwrap()).unwrap();

            assert_eq!(student_db.len(), 1);
            assert_eq!(cash_db.len(), 1);

            let student = student_db.iter().next().unwrap().1;
            assert_eq!(student.name(), "测试学生");

            let cash = cash_db.iter().next().unwrap().1;
            assert_eq!(cash.cash, 500);
        }
    }
}

mod stats_v1_tests {
    use super::*;

    #[test]
    fn test_dashboard_stats() {
        let mut student_db = StudentDatabase::new();
        let mut cash_db = CashDatabase::new();

        // 添加测试数据
        let mut student = Student::new();
        student
            .set_name("统计测试".to_string())
            .add_ring(85.5)
            .add_ring(92.0);
        let student_id = student.uid();
        student_db.insert(student);

        let mut cash = Cash::new(Some(student_id));
        cash.set_cash(1500);
        cash_db.insert(cash);

        let mut expense = Cash::new(None);
        expense.set_cash(-200);
        cash_db.insert(expense);

        // 生成统计
        let stats = get_dashboard_stats(&student_db, &cash_db).unwrap();

        assert_eq!(stats.total_students, 1);
        assert_eq!(stats.total_revenue, 1500);
        assert_eq!(stats.total_expense, 200);
        assert!((stats.average_score - 88.75).abs() < 0.01);
        assert_eq!(stats.max_score, 92.0);
    }
}

#[test]
fn test_v1_api_integration() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    std::env::set_current_dir(temp_path).unwrap();

    // 完整的 v1 API 工作流程
    let mut db = database::init().unwrap();

    // 1. 创建学生
    let mut student = Student::new();
    student
        .set_name("集成测试学生".to_string())
        .set_age(19)
        .set_class(Class::Month)
        .set_subject(Subject::Archery)
        .add_ring(88.0)
        .add_ring(91.5);

    let student_id = student.uid();
    db.student.insert(student);

    // 2. 记录现金流
    let mut income = Cash::new(Some(student_id));
    income.set_cash(2500);
    income.set_note(Some("月卡费用".to_string()));
    db.cash.insert(income);

    // 3. 生成统计
    let stats = get_dashboard_stats(&db.student, &db.cash).unwrap();
    assert_eq!(stats.total_students, 1);
    assert_eq!(stats.total_revenue, 2500);

    // 4. 保存数据
    db.save().unwrap();

    // 5. 验证持久化
    let reloaded_db = database::init().unwrap();
    assert_eq!(reloaded_db.student.len(), 1);
    assert_eq!(reloaded_db.cash.len(), 1);

    let reloaded_student = reloaded_db.student.get(&student_id).unwrap();
    assert_eq!(reloaded_student.name(), "集成测试学生");
    assert_eq!(reloaded_student.rings().len(), 2);
}
