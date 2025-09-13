use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use qmx_backend_lib::{
    QmxManager, StudentBuilder, CashBuilder, StudentUpdater, CashUpdater,
    StudentQuery, CashQuery, TimePeriod
};
use qmx_backend_lib::student::{Class, Subject};

/// 新API使用示例
/// 
/// 展示如何使用重新设计的QmxManager API进行各种操作
fn main() -> Result<()> {
    // ============================================================================
    // 1. 初始化管理器
    // ============================================================================
    
    // 创建自动保存的管理器
    let manager = QmxManager::new(true)?;
    println!("✅ QMX管理器初始化成功");

    // 或者从指定路径加载
    // let manager = QmxManager::from_path(
    //     "./data/students.json",
    //     "./data/cash.json",
    //     false  // 手动保存模式
    // )?;

    // ============================================================================
    // 2. 学生管理 - 使用构建器模式
    // ============================================================================
    
    println!("\n📚 学生管理示例");
    
    // 创建学生 - 链式调用，清晰易读
    let student_id = manager.create_student(
        StudentBuilder::new("张三", 16)
            .phone("13800138000")
            .class(Class::TenTry)
            .subject(Subject::Math)
            .lesson_left(20)
            .note("数学成绩优秀，需要加强英语")
            .membership(
                Utc::now(),
                Utc::now() + Duration::days(365)
            )
    )?;
    println!("✅ 创建学生成功，ID: {}", student_id);

    // 批量创建学生
    let students = vec![
        ("李四", 15, "13900139000", Class::VIP, Subject::English),
        ("王五", 17, "13700137000", Class::TenTry, Subject::Physics),
        ("赵六", 16, "13600136000", Class::VIP, Subject::Math),
    ];

    let mut student_ids = Vec::new();
    for (name, age, phone, class, subject) in students {
        let id = manager.create_student(
            StudentBuilder::new(name, age)
                .phone(phone)
                .class(class)
                .subject(subject)
        )?;
        student_ids.push(id);
        println!("✅ 创建学生 {} 成功，ID: {}", name, id);
    }

    // ============================================================================
    // 3. 学生信息更新 - 使用更新器模式
    // ============================================================================
    
    println!("\n🔄 学生信息更新示例");
    
    // 更新学生信息 - 只更新需要的字段
    manager.update_student(
        student_id,
        StudentUpdater::new()
            .age(17)  // 生日了，年龄+1
            .add_ring(95.5)  // 添加新成绩
            .add_ring(88.0)
            .note("数学成绩持续优秀，英语有所提升")
    )?;
    println!("✅ 更新学生信息成功");

    // 批量添加成绩
    manager.update_student(
        student_ids[0],
        StudentUpdater::new()
            .set_rings(vec![92.0, 88.5, 95.0, 90.5])
    )?;
    println!("✅ 批量更新成绩成功");

    // ============================================================================
    // 4. 现金流管理 - 简化的API
    // ============================================================================
    
    println!("\n💰 现金流管理示例");
    
    // 记录学费收入
    let cash_id = manager.record_cash(
        CashBuilder::new(5000)  // 5000元学费
            .student_id(student_id)
            .note("2025年春季学期学费")
    )?;
    println!("✅ 记录学费收入成功，ID: {}", cash_id);

    // 记录分期付款
    use qmx_backend_lib::cash::{Installment, InstallmentStatus};
    
    let installment_id = manager.record_cash(
        CashBuilder::new(10000)
            .student_id(student_ids[1])
            .note("VIP课程费用 - 分期付款")
            .installment(Installment {
                total_amount: 10000,
                paid_amount: 3000,
                remaining_amount: 7000,
                installment_count: 3,
                status: InstallmentStatus::InProgress,
            })
    )?;
    println!("✅ 记录分期付款成功，ID: {}", installment_id);

    // 记录支出
    let expense_id = manager.record_cash(
        CashBuilder::new(-1200)  // 负数表示支出
            .note("购买教学设备")
    )?;
    println!("✅ 记录支出成功，ID: {}", expense_id);

    // ============================================================================
    // 5. 智能查询 - 使用查询构建器
    // ============================================================================
    
    println!("\n🔍 智能查询示例");
    
    // 查询16-17岁的数学学生
    let math_students = manager.search_students(
        StudentQuery::new()
            .age_range(16, 17)
            .subject(Subject::Math)
    )?;
    println!("✅ 找到 {} 名16-17岁的数学学生", math_students.len());

    // 查询有会员资格的学生
    let members = manager.search_students(
        StudentQuery::new()
            .has_membership(true)
            .membership_active_at(Utc::now())
    )?;
    println!("✅ 找到 {} 名当前会员", members.len());

    // 查询VIP班级学生
    let vip_students = manager.search_students(
        StudentQuery::new()
            .class(Class::VIP)
    )?;
    println!("✅ 找到 {} 名VIP学生", vip_students.len());

    // 查询特定学生的所有现金记录
    let student_cash = manager.get_student_cash(student_id)?;
    println!("✅ 学生 {} 有 {} 条现金记录", student_id, student_cash.len());

    // 查询大额收入记录
    let large_income = manager.search_cash(
        CashQuery::new()
            .amount_range(5000, i64::MAX)
    )?;
    println!("✅ 找到 {} 条大额收入记录", large_income.len());

    // ============================================================================
    // 6. 统计分析 - 丰富的统计信息
    // ============================================================================
    
    println!("\n📊 统计分析示例");
    
    // 获取仪表板统计
    let dashboard = manager.get_dashboard_stats()?;
    println!("✅ 仪表板统计:");
    println!("   - 学生总数: {}", dashboard.total_students);
    println!("   - 现金记录总数: {}", dashboard.total_cash_records);
    println!("   - 总收入: {}", dashboard.total_income);

    // 获取学生详细统计
    let student_stats = manager.get_student_stats(student_id)?;
    println!("✅ 学生 {} 统计:", student_id);
    println!("   - 总付款: {} 元", student_stats.total_payments);
    println!("   - 付款次数: {}", student_stats.payment_count);
    if let Some(avg_score) = student_stats.average_score {
        println!("   - 平均成绩: {:.1}", avg_score);
    }
    println!("   - 会员状态: {:?}", student_stats.membership_status);

    // 获取财务统计
    let financial_stats = manager.get_financial_stats(TimePeriod::ThisMonth)?;
    println!("✅ 本月财务统计:");
    println!("   - 总收入: {} 元", financial_stats.total_income);
    println!("   - 总支出: {} 元", financial_stats.total_expense);
    println!("   - 净收入: {} 元", financial_stats.net_income);
    println!("   - 交易笔数: {}", financial_stats.transaction_count);

    // ============================================================================
    // 7. 数据管理 - 简化的保存操作
    // ============================================================================
    
    println!("\n💾 数据管理示例");
    
    // 手动保存（如果没有启用自动保存）
    manager.save()?;
    println!("✅ 数据保存成功");

    // ============================================================================
    // 8. 错误处理 - 统一的错误类型
    // ============================================================================
    
    println!("\n❌ 错误处理示例");
    
    // 尝试获取不存在的学生
    match manager.get_student(99999) {
        Ok(Some(student)) => println!("找到学生: {}", student.name()),
        Ok(None) => println!("✅ 学生不存在（正常情况）"),
        Err(e) => println!("❌ 查询出错: {}", e),
    }

    // 尝试更新不存在的学生
    if let Err(e) = manager.update_student(
        99999,
        StudentUpdater::new().name("不存在的学生")
    ) {
        println!("✅ 更新失败（预期行为）: {}", e);
    }

    println!("\n🎉 新API使用示例完成！");
    Ok(())
}

/// 高级使用场景示例
fn advanced_usage_examples() -> Result<()> {
    let manager = QmxManager::new(false)?; // 手动保存模式

    // ============================================================================
    // 场景1: 批量导入学生数据
    // ============================================================================
    
    println!("📥 批量导入学生数据");
    
    let student_data = vec![
        ("张小明", 15, "Math", "TenTry"),
        ("李小红", 16, "English", "VIP"),
        ("王小强", 17, "Physics", "TenTry"),
        // ... 更多数据
    ];

    for (name, age, subject_str, class_str) in student_data {
        let subject = match subject_str {
            "Math" => Subject::Math,
            "English" => Subject::English,
            "Physics" => Subject::Physics,
            _ => Subject::Math,
        };
        
        let class = match class_str {
            "VIP" => Class::VIP,
            _ => Class::TenTry,
        };

        manager.create_student(
            StudentBuilder::new(name, age)
                .subject(subject)
                .class(class)
        )?;
    }
    
    // 批量保存
    manager.save()?;
    println!("✅ 批量导入完成");

    // ============================================================================
    // 场景2: 会员到期提醒
    // ============================================================================
    
    println!("⏰ 会员到期提醒");
    
    let now = Utc::now();
    let next_week = now + Duration::days(7);
    
    let all_students = manager.list_students()?;
    let expiring_soon: Vec<_> = all_students
        .iter()
        .filter(|student| {
            if let Some(end_date) = student.membership_end_date() {
                *end_date > now && *end_date <= next_week
            } else {
                false
            }
        })
        .collect();

    println!("✅ 发现 {} 名学生会员即将到期", expiring_soon.len());
    for student in expiring_soon {
        println!("   - {}: 到期日 {:?}", student.name(), student.membership_end_date());
    }

    // ============================================================================
    // 场景3: 财务报表生成
    // ============================================================================
    
    println!("📈 财务报表生成");
    
    let periods = vec![
        TimePeriod::Today,
        TimePeriod::ThisWeek,
        TimePeriod::ThisMonth,
    ];

    for period in periods {
        let stats = manager.get_financial_stats(period.clone())?;
        println!("✅ {:?} 财务统计: 收入{}元, 支出{}元, 净收入{}元", 
                 period, stats.total_income, stats.total_expense, stats.net_income);
    }

    Ok(())
}

/// 性能优化示例
fn performance_examples() -> Result<()> {
    let manager = QmxManager::new(false)?; // 禁用自动保存以提高性能

    // ============================================================================
    // 大批量操作
    // ============================================================================
    
    println!("⚡ 性能优化示例");
    
    // 批量创建（禁用自动保存）
    let start = std::time::Instant::now();
    
    for i in 0..1000 {
        manager.create_student(
            StudentBuilder::new(format!("学生{}", i), 16)
                .class(Class::TenTry)
                .subject(Subject::Math)
        )?;
    }
    
    // 一次性保存
    manager.save()?;
    
    let duration = start.elapsed();
    println!("✅ 创建1000名学生耗时: {:?}", duration);

    // 批量查询
    let start = std::time::Instant::now();
    let math_students = manager.search_students(
        StudentQuery::new().subject(Subject::Math)
    )?;
    let duration = start.elapsed();
    println!("✅ 查询数学学生耗时: {:?}, 找到{}名", duration, math_students.len());

    Ok(())
}