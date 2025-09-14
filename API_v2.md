# QMX Backend Library v2 API 文档

## 概述

QMX Backend Library v2 提供了全新的统一API入口 `QmxManager`，采用现代化的设计模式，包括Builder模式、Updater模式和Query Builder模式，提供更加直观和易用的开发体验。

**版本：** 2.2.0  
**最后更新：** 2025-09-14

## 新特性 (v2.0.0)

- 🚀 **统一API入口QmxManager** - 一个类管理所有功能
- 🏗️ **Builder模式** - 链式构建学生和现金记录
- 🔄 **Updater模式** - 声明式更新数据
- 🔍 **Query Builder** - 表达式查询系统
- 📊 **增强统计分析** - 丰富的数据洞察
- 🔒 **线程安全设计** - Arc<RwLock<T>>架构
- 🔧 **完全向后兼容** - v1 API继续可用

---

## 1. QmxManager 统一入口

### 基本使用

```rust
use qmx_backend_lib::QmxManager;

// 初始化管理器（自动保存模式）
let manager = QmxManager::new(true)?;

// 从指定路径初始化
let manager = QmxManager::from_path(
    "./data/students.json", 
    "./data/cash.json", 
    true
)?;

// 手动保存
manager.save()?;
```

---

## 2. 学生管理

### StudentBuilder - 创建学生

```rust
use qmx_backend_lib::{QmxManager, StudentBuilder, Class, Subject};
use chrono::{Utc, Duration};

let manager = QmxManager::new(true)?;

// 基础创建
let uid = manager.create_student(
    StudentBuilder::new("张三", 18)
        .phone("13800138000")
        .class(Class::TenTry)
        .subject(Subject::Shooting)
        .lesson_left(10)
        .note("新学员")
        .membership(
            Utc::now(),
            Utc::now() + Duration::days(365)
        )
)?;

// 最简创建
let uid = manager.create_student(
    StudentBuilder::new("李四", 16)
)?;
```

#### StudentBuilder API
```rust
impl StudentBuilder {
    pub fn new(name: impl Into<String>, age: u8) -> Self
    pub fn phone(self, phone: impl Into<String>) -> Self
    pub fn class(self, class: Class) -> Self
    pub fn subject(self, subject: Subject) -> Self
    pub fn lesson_left(self, lessons: u32) -> Self
    pub fn note(self, note: impl Into<String>) -> Self
    pub fn membership(self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self
}
```

### StudentUpdater - 更新学生

```rust
use qmx_backend_lib::StudentUpdater;

// 声明式更新
manager.update_student(uid, 
    StudentUpdater::new()
        .name("张三丰")
        .age(19)
        .phone("13900139000")
        .add_ring(9.5)
        .note("更新备注")
)?;

// 批量设置成绩
manager.update_student(uid,
    StudentUpdater::new()
        .set_rings(vec![9.0, 9.5, 8.8, 9.2])
)?;

// 更新会员信息
manager.update_student(uid,
    StudentUpdater::new()
        .membership(
            Some(Utc::now()),
            Some(Utc::now() + Duration::days(180))
        )
)?;

// 更新指定位置的成绩
manager.update_student(uid,
    StudentUpdater::new()
        .update_ring_at(0, 9.8)  // 更新第1个成绩为9.8
)?;

// 删除指定位置的成绩
manager.update_student(uid,
    StudentUpdater::new()
        .remove_ring_at(1)  // 删除第2个成绩
)?;
```

#### StudentUpdater API
```rust
impl StudentUpdater {
    pub fn new() -> Self
    pub fn name(self, name: impl Into<String>) -> Self
    pub fn age(self, age: u8) -> Self
    pub fn phone(self, phone: impl Into<String>) -> Self
    pub fn class(self, class: Class) -> Self
    pub fn subject(self, subject: Subject) -> Self
    pub fn lesson_left(self, lessons: Option<u32>) -> Self
    pub fn note(self, note: impl Into<String>) -> Self
    pub fn add_ring(self, score: f64) -> Self
    pub fn set_rings(self, rings: Vec<f64>) -> Self
    pub fn update_ring_at(self, index: usize, value: f64) -> Self
    pub fn remove_ring_at(self, index: usize) -> Self
    pub fn membership(self, start: Option<DateTime<Utc>>, end: Option<DateTime<Utc>>) -> Self
}
```

### StudentQuery - 查询学生

```rust
use qmx_backend_lib::StudentQuery;

// 复合查询
let students = manager.search_students(
    StudentQuery::new()
        .name_contains("张")
        .age_range(16, 20)
        .class(Class::TenTry)
        .has_membership(true)
)?;

// 查询活跃会员
let active_members = manager.search_students(
    StudentQuery::new()
        .membership_active_at(Utc::now())
)?;

// 获取所有学生
let all_students = manager.list_students()?;

// 获取单个学生
let student = manager.get_student(uid)?;

// 删除学生
let deleted = manager.delete_student(uid)?;
```

#### StudentQuery API
```rust
impl StudentQuery {
    pub fn new() -> Self
    pub fn name_contains(self, name: impl Into<String>) -> Self
    pub fn age_range(self, min: u8, max: u8) -> Self
    pub fn class(self, class: Class) -> Self
    pub fn subject(self, subject: Subject) -> Self
    pub fn has_membership(self, has: bool) -> Self
    pub fn membership_active_at(self, date: DateTime<Utc>) -> Self
}
```

---

## 3. 现金管理

### CashBuilder - 记录现金

```rust
use qmx_backend_lib::{CashBuilder, Installment, PaymentFrequency, InstallmentStatus};

let manager = QmxManager::new(true)?;

// 简单现金记录
let cash_id = manager.record_cash(
    CashBuilder::new(1000)
        .student_id(student_uid)
        .note("学费收入")
)?;

// 支出记录
let expense_id = manager.record_cash(
    CashBuilder::new(-500)
        .note("设备采购")
)?;

// 分期付款记录
let installment = Installment {
    plan_id: 1001,
    total_amount: 3000,
    total_installments: 3,
    current_installment: 1,
    frequency: PaymentFrequency::Monthly,
    due_date: Utc::now() + Duration::days(30),
    status: InstallmentStatus::Pending,
};

let installment_id = manager.record_cash(
    CashBuilder::new(1000)
        .student_id(student_uid)
        .installment(installment)
        .note("分期付款第1期")
)?;
```

#### CashBuilder API
```rust
impl CashBuilder {
    pub fn new(amount: i64) -> Self
    pub fn student_id(self, student_id: u64) -> Self
    pub fn note(self, note: impl Into<String>) -> Self
    pub fn installment(self, installment: Installment) -> Self
}
```

### CashUpdater - 更新现金记录

```rust
use qmx_backend_lib::CashUpdater;

// 更新现金记录
manager.update_cash(cash_id,
    CashUpdater::new()
        .amount(1200)
        .note(Some("更新后的学费".to_string()))
)?;

// 清除学生关联
manager.update_cash(cash_id,
    CashUpdater::new()
        .student_id(None)
)?;
```

#### CashUpdater API
```rust
impl CashUpdater {
    pub fn new() -> Self
    pub fn student_id(self, student_id: Option<u64>) -> Self
    pub fn amount(self, amount: i64) -> Self
    pub fn note(self, note: Option<String>) -> Self
    pub fn installment(self, installment: Option<Installment>) -> Self
}
```

### CashQuery - 查询现金记录

```rust
use qmx_backend_lib::CashQuery;

// 查询学生的所有现金记录
let student_cash = manager.get_student_cash(student_uid)?;

// 金额范围查询
let high_value = manager.search_cash(
    CashQuery::new()
        .amount_range(1000, 5000)
        .has_installment(false)
)?;

// 查询特定学生的分期付款
let installments = manager.search_cash(
    CashQuery::new()
        .student_id(student_uid)
        .has_installment(true)
)?;

// 获取单个现金记录
let cash = manager.get_cash(cash_id)?;

// 删除现金记录
let deleted = manager.delete_cash(cash_id)?;
```

#### CashQuery API
```rust
impl CashQuery {
    pub fn new() -> Self
    pub fn student_id(self, student_id: u64) -> Self
    pub fn amount_range(self, min: i64, max: i64) -> Self
    pub fn has_installment(self, has: bool) -> Self
}
```

---

## 4. 统计分析

### 仪表板统计

```rust
use qmx_backend_lib::stats::DashboardStats;

let dashboard = manager.get_dashboard_stats()?;

println!("学生总数: {}", dashboard.total_students);
println!("总收入: {}", dashboard.total_revenue);
println!("总支出: {}", dashboard.total_expense);
println!("平均成绩: {:.2}", dashboard.average_score);
println!("最高成绩: {:.2}", dashboard.max_score);
println!("活跃课程数: {}", dashboard.active_courses);
```

### 学生统计

```rust
use qmx_backend_lib::{StudentStats, MembershipStatus};

let stats = manager.get_student_stats(student_uid)?;

println!("总付款: {}", stats.total_payments);
println!("付款次数: {}", stats.payment_count);
if let Some(avg) = stats.average_score {
    println!("平均成绩: {:.2}", avg);
}
println!("成绩记录数: {}", stats.score_count);

match stats.membership_status {
    MembershipStatus::None => println!("非会员"),
    MembershipStatus::Active { expires_at } => {
        println!("活跃会员，到期时间: {}", expires_at);
    },
    MembershipStatus::Expired { expired_at } => {
        println!("已过期会员，过期时间: {}", expired_at);
    },
}
```

### 财务统计

```rust
use qmx_backend_lib::{FinancialStats, TimePeriod};

// 本月财务统计
let stats = manager.get_financial_stats(TimePeriod::ThisMonth)?;

println!("总收入: {}", stats.total_income);
println!("总支出: {}", stats.total_expense);
println!("净收入: {}", stats.net_income);
println!("分期付款总额: {}", stats.installment_total);
println!("已收分期款: {}", stats.installment_paid);
println!("待收分期款: {}", stats.installment_pending);

// 其他时间段
let yearly_stats = manager.get_financial_stats(TimePeriod::ThisYear)?;
let weekly_stats = manager.get_financial_stats(TimePeriod::ThisWeek)?;
```

#### TimePeriod 枚举
```rust
pub enum TimePeriod {
    ThisWeek,
    ThisMonth,
    ThisYear,
    AllTime,
}
```

#### 统计结构体定义

```rust
#[derive(Debug, Clone)]
pub struct StudentStats {
    pub total_payments: i64,
    pub payment_count: usize,
    pub average_score: Option<f64>,
    pub score_count: usize,
    pub membership_status: MembershipStatus,
}

#[derive(Debug, Clone)]
pub struct FinancialStats {
    pub total_income: i64,
    pub total_expense: i64,
    pub net_income: i64,
    pub installment_total: i64,
    pub installment_paid: i64,
    pub installment_pending: i64,
}

#[derive(Debug, Clone)]
pub enum MembershipStatus {
    None,
    Active { expires_at: DateTime<Utc> },
    Expired { expired_at: DateTime<Utc> },
}
```

---

## 5. 完整使用示例

### 学生管理流程

```rust
use qmx_backend_lib::*;
use chrono::{Utc, Duration};

fn student_management_example() -> anyhow::Result<()> {
    let manager = QmxManager::new(true)?;
    
    // 1. 创建学生
    let uid = manager.create_student(
        StudentBuilder::new("王小明", 17)
            .phone("13700137000")
            .class(Class::Month)
            .subject(Subject::Archery)
            .membership(
                Utc::now(),
                Utc::now() + Duration::days(30)
            )
    )?;
    
    // 2. 记录学费收入
    let _cash_id = manager.record_cash(
        CashBuilder::new(2000)
            .student_id(uid)
            .note("月卡学费")
    )?;
    
    // 3. 添加成绩
    manager.update_student(uid,
        StudentUpdater::new()
            .add_ring(8.9)
            .add_ring(9.1)
            .add_ring(8.5)
    )?;
    
    // 4. 更新特定位置的成绩
    manager.update_student(uid,
        StudentUpdater::new()
            .update_ring_at(1, 9.3)  // 将第2个成绩从9.1改为9.3
    )?;
    
    // 5. 删除最后一个成绩
    manager.update_student(uid,
        StudentUpdater::new()
            .remove_ring_at(2)  // 删除第3个成绩(8.5)
    )?;
    
    // 6. 查询该学生信息
    if let Some(student) = manager.get_student(uid)? {
        println!("学生: {}, 年龄: {}", student.name(), student.age());
        println!("成绩: {:?}", student.rings());
        
        if student.is_membership_active() {
            println!("会员状态: 有效");
        }
    }
    
    // 7. 获取学生统计
    let stats = manager.get_student_stats(uid)?;
    println!("总付款: {}", stats.total_payments);
    
    Ok(())
}
```

### 财务分析流程

```rust
fn financial_analysis_example() -> anyhow::Result<()> {
    let manager = QmxManager::new(true)?;
    
    // 1. 记录各种收支
    manager.record_cash(CashBuilder::new(5000).note("学费收入"))?;
    manager.record_cash(CashBuilder::new(-800).note("设备维护"))?;
    manager.record_cash(CashBuilder::new(3000).note("新学员注册费"))?;
    
    // 2. 获取仪表板数据
    let dashboard = manager.get_dashboard_stats()?;
    println!("=== 仪表板统计 ===");
    println!("学生总数: {}", dashboard.total_students);
    println!("总收入: {}", dashboard.total_revenue); 
    println!("总支出: {}", dashboard.total_expense);
    println!("净收入: {}", dashboard.total_revenue + dashboard.total_expense);
    
    // 3. 获取本月财务统计
    let monthly = manager.get_financial_stats(TimePeriod::ThisMonth)?;
    println!("\n=== 本月财务 ===");
    println!("收入: {}", monthly.total_income);
    println!("支出: {}", monthly.total_expense);
    println!("净收入: {}", monthly.net_income);
    
    // 4. 查询大额收入
    let high_income = manager.search_cash(
        CashQuery::new().amount_range(2000, i64::MAX)
    )?;
    println!("\n=== 大额收入记录 ===");
    for cash in high_income {
        println!("金额: {}, 备注: {:?}", cash.cash, cash.note);
    }
    
    Ok(())
}
```

### 成绩管理流程

```rust
fn score_management_example() -> anyhow::Result<()> {
    let manager = QmxManager::new(true)?;
    
    // 1. 创建学生
    let uid = manager.create_student(
        StudentBuilder::new("射击学员", 18)
            .class(Class::TenTry)
            .subject(Subject::Shooting)
    )?;
    
    // 2. 添加初始成绩
    manager.update_student(uid,
        StudentUpdater::new()
            .add_ring(8.5)
            .add_ring(9.0)
            .add_ring(8.8)
            .add_ring(9.2)
    )?;
    
    println!("初始成绩: {:?}", manager.get_student(uid)?.unwrap().rings());
    // 输出: [8.5, 9.0, 8.8, 9.2]
    
    // 3. 更新第2个成绩（索引1）
    manager.update_student(uid,
        StudentUpdater::new()
            .update_ring_at(1, 9.5)  // 9.0 -> 9.5
    )?;
    
    println!("更新后成绩: {:?}", manager.get_student(uid)?.unwrap().rings());
    // 输出: [8.5, 9.5, 8.8, 9.2]
    
    // 4. 删除第1个成绩（索引0）
    manager.update_student(uid,
        StudentUpdater::new()
            .remove_ring_at(0)  // 删除8.5
    )?;
    
    println!("删除后成绩: {:?}", manager.get_student(uid)?.unwrap().rings());
    // 输出: [9.5, 8.8, 9.2]
    
    // 5. 批量替换所有成绩
    manager.update_student(uid,
        StudentUpdater::new()
            .set_rings(vec![9.8, 9.6, 9.9, 10.0])
    )?;
    
    println!("批量替换后: {:?}", manager.get_student(uid)?.unwrap().rings());
    // 输出: [9.8, 9.6, 9.9, 10.0]
    
    // 6. 错误处理示例
    match manager.update_student(uid,
        StudentUpdater::new()
            .update_ring_at(10, 9.0)  // 索引越界
    ) {
        Ok(_) => println!("更新成功"),
        Err(e) => println!("更新失败: {}", e),
    }
    
    Ok(())
}
```

### 会员管理流程

```rust
fn membership_management_example() -> anyhow::Result<()> {
    let manager = QmxManager::new(true)?;
    
    // 1. 创建会员学生
    let uid = manager.create_student(
        StudentBuilder::new("VIP学员", 20)
            .class(Class::Year)
            .membership(
                Utc::now(),
                Utc::now() + Duration::days(365)
            )
    )?;
    
    // 2. 查询所有活跃会员
    let active_members = manager.search_students(
        StudentQuery::new()
            .has_membership(true)
            .membership_active_at(Utc::now())
    )?;
    
    println!("活跃会员数量: {}", active_members.len());
    
    // 3. 检查即将到期的会员
    let next_week = Utc::now() + Duration::days(7);
    for student in active_members {
        if let Some(end_date) = student.membership_end_date() {
            if end_date <= next_week {
                println!("会员 {} 即将到期: {}", student.name(), end_date);
            }
        }
    }
    
    // 4. 延长会员期限
    manager.update_student(uid,
        StudentUpdater::new()
            .membership(
                Some(Utc::now()),
                Some(Utc::now() + Duration::days(730)) // 延长2年
            )
    )?;
    
    Ok(())
}
```

---

## 6. 线程安全和性能

### 线程安全设计

QmxManager 使用 `Arc<RwLock<T>>` 设计，支持多线程环境：

```rust
use std::sync::Arc;
use std::thread;

fn concurrent_example() -> anyhow::Result<()> {
    let manager = Arc::new(QmxManager::new(true)?);
    
    let mut handles = vec![];
    
    // 启动多个线程同时操作
    for i in 0..5 {
        let manager_clone = Arc::clone(&manager);
        let handle = thread::spawn(move || {
            let uid = manager_clone.create_student(
                StudentBuilder::new(format!("学生{}", i), 18)
            ).unwrap();
            
            manager_clone.record_cash(
                CashBuilder::new(1000)
                    .student_id(uid)
            ).unwrap();
        });
        handles.push(handle);
    }
    
    // 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("并发操作完成，学生总数: {}", 
             manager.list_students()?.len());
    
    Ok(())
}
```

### 性能优化建议

1. **批量操作**：对于大量数据操作，考虑使用v1 API的批量方法
2. **查询优化**：使用具体的查询条件而不是获取全部数据后过滤
3. **自动保存**：在高频操作场景考虑关闭自动保存，手动控制保存时机

---

## 7. 错误处理

所有v2 API都返回 `anyhow::Result<T>`，提供详细的错误信息：

```rust
use anyhow::Context;

fn error_handling_example() -> anyhow::Result<()> {
    let manager = QmxManager::new(true)
        .context("初始化管理器失败")?;
    
    let uid = manager.create_student(
        StudentBuilder::new("测试学生", 18)
    ).with_context(|| "创建学生失败")?;
    
    manager.update_student(uid, 
        StudentUpdater::new().age(19)
    ).with_context(|| format!("更新学生{}失败", uid))?;
    
    Ok(())
}
```

---

## 8. 迁移指南 (v1 → v2)

### v1 到 v2 对照表

| v1 操作 | v2 等效操作 |
|---------|-------------|
| `Student::new()` | `StudentBuilder::new(name, age)` |
| `student.set_name()` | `StudentUpdater::new().name()` |
| `db.student.insert()` | `manager.create_student()` |
| `db.student.get()` | `manager.get_student()` |
| `Cash::new()` | `CashBuilder::new(amount)` |
| `get_dashboard_stats()` | `manager.get_dashboard_stats()` |

### 渐进式迁移

```rust
// 可以同时使用v1和v2 API
use qmx_backend_lib::*;

fn mixed_usage_example() -> anyhow::Result<()> {
    // v2 API
    let manager = QmxManager::new(true)?;
    let uid = manager.create_student(
        StudentBuilder::new("新学员", 18)
    )?;
    
    // v1 API（仍然可用）
    let mut db = init::init()?;
    let mut student = Student::new();
    student.set_name("传统方式".to_string());
    db.student.insert(student);
    
    Ok(())
}
```

---

## 最佳实践

### 1. Builder模式使用
```rust
// ✅ 推荐：使用链式调用
let uid = manager.create_student(
    StudentBuilder::new("张三", 18)
        .phone("13800138000")
        .class(Class::TenTry)
        .note("新学员")
)?;

// ✅ 推荐：最简创建
let uid = manager.create_student(
    StudentBuilder::new("李四", 16)
)?;
```

### 2. 查询优化
```rust
// ✅ 推荐：使用具体查询条件
let teenagers = manager.search_students(
    StudentQuery::new()
        .age_range(13, 19)
        .has_membership(true)
)?;

// ❌ 避免：获取全部后过滤
let all = manager.list_students()?;
let teenagers: Vec<_> = all.into_iter()
    .filter(|s| s.age() >= 13 && s.age() <= 19)
    .collect();
```

### 3. 错误处理
```rust
// ✅ 推荐：提供上下文信息
manager.update_student(uid, updater)
    .with_context(|| format!("更新学生{}失败", uid))?;

// ✅ 推荐：优雅处理可选结果
match manager.get_student(uid)? {
    Some(student) => println!("找到学生: {}", student.name()),
    None => println!("学生不存在: {}", uid),
}
```

---

*文档版本：2.2.0*  
*最后更新：2025-09-14*