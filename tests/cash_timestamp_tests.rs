// 测试 Cash 时间戳和 DateRange 功能的单元测试
use chrono::{Duration, Utc};
use qmx_backend_lib::cash::{Cash, CashDatabase};
use std::{thread, time};

#[test]
fn test_cash_has_timestamp() {
    // 测试 Cash::new 是否设置了时间戳
    let before = Utc::now();
    thread::sleep(time::Duration::from_millis(10));

    let cash = Cash::new(Some(123));

    thread::sleep(time::Duration::from_millis(10));
    let after = Utc::now();

    // 验证时间戳在合理范围内
    assert!(cash.created_at >= before);
    assert!(cash.created_at <= after);

    println!("Cash created at: {}", cash.created_at);
}

#[test]
fn test_cash_new_installment_has_timestamp() {
    use qmx_backend_lib::cash::PaymentFrequency;

    let before = Utc::now();
    thread::sleep(time::Duration::from_millis(10));

    let cash = Cash::new_installment(
        Some(456),
        3000,
        3,
        PaymentFrequency::Monthly,
        Utc::now() + Duration::days(30),
        1,
        None,
    );

    thread::sleep(time::Duration::from_millis(10));
    let after = Utc::now();

    // 验证时间戳在合理范围内
    assert!(cash.created_at >= before);
    assert!(cash.created_at <= after);

    // 验证分期付款信息也存在
    assert!(cash.installment.is_some());

    println!("Installment cash created at: {}", cash.created_at);
}

#[test]
fn test_cash_database_operations_with_timestamp() {
    let mut db = CashDatabase::new();

    // 创建几个不同时间的现金记录
    let cash1 = {
        let mut c = Cash::new(Some(1));
        c.set_cash(1000);
        c
    };

    thread::sleep(time::Duration::from_millis(10));

    let cash2 = {
        let mut c = Cash::new(Some(2));
        c.set_cash(2000);
        c
    };

    thread::sleep(time::Duration::from_millis(10));

    let cash3 = {
        let mut c = Cash::new(Some(3));
        c.set_cash(3000);
        c
    };

    // 存储时间戳用于后续验证
    let time1 = cash1.created_at;
    let time2 = cash2.created_at;
    let time3 = cash3.created_at;

    // 插入数据库
    db.insert(cash1);
    db.insert(cash2);
    db.insert(cash3);

    assert_eq!(db.len(), 3);

    // 验证时间戳顺序
    assert!(time1 < time2);
    assert!(time2 < time3);

    // 验证数据库中的记录保持了时间戳
    for (_, cash) in db.iter() {
        assert!(cash.created_at.timestamp() > 0);
        println!("Database cash created at: {}", cash.created_at);
    }
}

#[test]
fn test_cash_serialization_with_timestamp() {
    let cash = {
        let mut c = Cash::new(Some(789));
        c.set_cash(5000);
        c.set_note(Some("测试备注".to_string()));
        c
    };

    let original_time = cash.created_at;

    // 测试序列化
    let json = serde_json::to_string(&cash).unwrap();
    println!("Serialized cash: {}", json);

    // 验证 JSON 中包含时间戳
    assert!(json.contains("created_at"));

    // 测试反序列化
    let deserialized: Cash = serde_json::from_str(&json).unwrap();

    // 验证时间戳保持不变
    assert_eq!(deserialized.created_at, original_time);
    assert_eq!(deserialized.cash, 5000);
    assert_eq!(deserialized.note, Some("测试备注".to_string()));
    assert_eq!(deserialized.student_id, Some(789));
}

#[test]
fn test_cash_database_json_with_timestamp() {
    let mut db = CashDatabase::new();

    let cash1 = {
        let mut c = Cash::new(Some(100));
        c.set_cash(1500);
        c
    };

    let cash2 = {
        let mut c = Cash::new(Some(200));
        c.set_cash(2500);
        c
    };

    db.insert(cash1);
    db.insert(cash2);

    // 序列化整个数据库
    let json = db.json();
    println!("Database JSON: {}", json);

    // 验证包含时间戳
    assert!(json.contains("created_at"));

    // 反序列化
    let restored_db = CashDatabase::from_json(&json).unwrap();

    assert_eq!(restored_db.len(), 2);

    // 验证时间戳被正确恢复
    for (_, cash) in restored_db.iter() {
        assert!(cash.created_at.timestamp() > 0);
        println!("Restored cash created at: {}", cash.created_at);
    }
}
