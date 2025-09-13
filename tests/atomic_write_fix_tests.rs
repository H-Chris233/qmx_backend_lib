// 测试原子写入问题的修复
use qmx_backend_lib::cash::CashDatabase;
use qmx_backend_lib::common::Database;
use qmx_backend_lib::student::StudentDatabase;
use qmx_backend_lib::*;
use tempfile::TempDir;

#[test]
fn test_improved_atomic_write() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let student_db_path = temp_path.join("student_database.json");
    let cash_db_path = temp_path.join("cash_database.json");

    // 创建空的数据库文件
    let student_db = StudentDatabase::new();
    let cash_db = CashDatabase::new();
    student_db
        .save_to_simple(student_db_path.to_str().unwrap())
        .unwrap();
    cash_db
        .save_to_simple(cash_db_path.to_str().unwrap())
        .unwrap();

    // 使用改进的原子写入
    let manager = QmxManager::from_path(
        student_db_path.to_str().unwrap(),
        cash_db_path.to_str().unwrap(),
        true,
    )
    .unwrap();

    // 创建一些数据
    let student_id = manager
        .create_student(StudentBuilder::new("测试学生", 20))
        .unwrap();

    let _cash_id = manager
        .record_cash(CashBuilder::new(1500).student_id(student_id))
        .unwrap();

    // 手动保存测试
    let save_result = manager.save();
    assert!(save_result.is_ok(), "保存应该成功: {:?}", save_result);

    // 验证文件存在
    assert!(student_db_path.exists());
    assert!(cash_db_path.exists());
}

#[test]
fn test_database_operations_with_improved_save() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let student_db_path = temp_path.join("student_database.json");
    let cash_db_path = temp_path.join("cash_database.json");

    // 第一阶段：创建数据并保存
    {
        // 创建空的数据库文件
        let student_db = StudentDatabase::new();
        let cash_db = CashDatabase::new();
        student_db
            .save_to_simple(student_db_path.to_str().unwrap())
            .unwrap();
        cash_db
            .save_to_simple(cash_db_path.to_str().unwrap())
            .unwrap();

        let manager = QmxManager::from_path(
            student_db_path.to_str().unwrap(),
            cash_db_path.to_str().unwrap(),
            true,
        )
        .unwrap();

        let student_id = manager
            .create_student(StudentBuilder::new("持久化测试", 25))
            .unwrap();

        let _cash_id = manager
            .record_cash(
                CashBuilder::new(2500)
                    .student_id(student_id)
                    .note("测试收入".to_string()),
            )
            .unwrap();

        // 确保保存成功
        let save_result = manager.save();
        assert!(save_result.is_ok());
    }

    // 第二阶段：重新加载数据验证持久化
    {
        let manager = QmxManager::from_path(
            student_db_path.to_str().unwrap(),
            cash_db_path.to_str().unwrap(),
            false,
        )
        .unwrap();

        let students = manager.list_students().unwrap();
        let cash_records = manager.search_cash(CashQuery::new()).unwrap();

        assert_eq!(students.len(), 1);
        assert_eq!(cash_records.len(), 1);
        assert_eq!(students[0].name(), "持久化测试");
        assert_eq!(cash_records[0].cash, 2500);
    }
}
