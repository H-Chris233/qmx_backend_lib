# QMX Backend Library API 完整文档

## 概述

QMX Backend Library 是一个用 Rust 编写的学生管理和财务管理库，提供完整的学生信息管理、现金流记录、分期付款处理和统计分析功能。

**版本：** 2.0.0  
**最后更新：** 2025-01-09

## 新特性 (v1.2.0)

- 🔧 **通用Database<T> trait** - 统一数据库操作接口，消除重复代码
- 🎯 **统一错误处理** - 全面使用 `anyhow::Result<T>`
- 🛠️ **优化API设计** - 更符合Rust最佳实践
- 👥 **会员管理系统** - 完整的会员期限管理功能
- 🔒 **向后兼容性** - 现有代码无需修改
- 🚀 **新增统一入口 QmxManager** - 更易用的新公共API（详见第8章）

## 模块结构

- **student.rs** - 学生信息和会员管理
- **cash.rs** - 现金流和分期付款管理  
- **database.rs** - 运行时数据库容器
- **stats.rs** - 统计分析
- **init.rs** - 系统初始化
- **save.rs** - 数据持久化
- **common.rs** - 通用数据库trait和接口 ✨新增✨

---

## 1. 学生管理模块 (student.rs)

### Student 结构体

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Student {
    uid: u64,                                    // 唯一标识符（自动生成）
    age: u8,                                     // 年龄（0-255）
    name: String,                                // 姓名
    phone: String,                               // 电话号码
    lesson_left: Option<u32>,                    // 剩余课时（仅TenTry班级有效）
    class: Class,                                // 班级类型枚举
    subject: Subject,                            // 科目类型枚举
    rings: Vec<f64>,                            // 成绩记录列表
    note: String,                               // 备注文本
    membership_start_date: Option<DateTime<Utc>>, // 会员开始日期 ✨新增✨
    membership_end_date: Option<DateTime<Utc>>,   // 会员结束日期 ✨新增✨
}
```

#### 枚举定义
```rust
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Class {
    TenTry,  // 十次体验课（自动设置lesson_left=10）
    Month,   // 月卡
    Year,    // 年卡  
    Others   // 其他
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Subject {
    Shooting,  // 射击
    Archery,   // 射箭
    Others     // 其他
}
```

### Student API 方法

#### 构造方法
```rust
pub fn new() -> Self
```
- 自动生成递增UID
- 默认值：`age=0`, `name/phone="未填写"`, `class/subject=Others`

#### 基本信息设置（链式调用）
```rust
pub fn set_age(&mut self, age: u8) -> &mut Self
pub fn set_name(&mut self, name: String) -> &mut Self
pub fn set_phone(&mut self, phone: String) -> &mut Self
pub fn set_class(&mut self, class: Class) -> &mut Self
pub fn set_lesson_left(&mut self, lesson: u32) -> &mut Self
pub fn set_subject(&mut self, subject: Subject) -> &mut Self
pub fn set_note(&mut self, note: String) -> &mut Self
pub fn add_ring(&mut self, ring: f64) -> &mut Self
pub unsafe fn set_id(&mut self, id: u64) -> &mut Self  // 强制覆盖UID
```

#### 会员管理方法 ✨新增✨
```rust
pub fn set_membership_dates(&mut self, start_date: Option<DateTime<Utc>>, end_date: Option<DateTime<Utc>>) -> &mut Self
pub fn set_membership_start_date(&mut self, start_date: DateTime<Utc>) -> &mut Self
pub fn set_membership_end_date(&mut self, end_date: DateTime<Utc>) -> &mut Self
pub fn clear_membership(&mut self) -> &mut Self
pub fn is_membership_active(&self) -> bool                    // 检查会员是否有效
pub fn membership_days_remaining(&self) -> Option<i64>        // 获取剩余天数
```

#### 字段获取方法
```rust
pub fn uid(&self) -> u64
pub fn age(&self) -> u8
pub fn name(&self) -> &str
pub fn phone(&self) -> &str
pub fn lesson_left(&self) -> Option<u32>
pub fn class(&self) -> &Class
pub fn subject(&self) -> &Subject 
pub fn rings(&self) -> &[f64]                                // ✨改进✨ 返回切片而非Vec引用
pub fn note(&self) -> &str
pub fn membership_start_date(&self) -> Option<DateTime<Utc>>  // ✨新增✨
pub fn membership_end_date(&self) -> Option<DateTime<Utc>>    // ✨新增✨
```

### StudentDatabase 结构体

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StudentDatabase {
    pub student_data: BTreeMap<u64, Student>,
}
```

#### StudentDatabase API 方法

**基础操作** (实现了 `Database<Student>` trait ✨新增✨)
```rust
// 构造和序列化
pub fn new() -> Self
pub fn from_json(json: &str) -> Result<Self>
pub fn json(&self) -> String

// 持久化
pub fn save(&self) -> Result<()>                        // ✨改进✨ 现在返回 anyhow::Result
pub fn save_to(&self, path: &str) -> Result<()>
pub fn read_from(path: &str) -> Result<Self>

// CRUD操作
pub fn insert(&mut self, student: Student)
pub fn insert_batch(&mut self, students: Vec<Student>) -> usize
pub fn update_batch<F>(&mut self, uids: &[u64], update_fn: F) -> usize
pub fn get(&self, uid: &u64) -> Option<&Student>
pub fn remove(&mut self, uid: &u64) -> Option<Student>
pub fn remove_batch(&mut self, uids: &[u64]) -> usize

// 查询和迭代
pub fn iter(&self) -> impl Iterator<Item = (&u64, &Student)>
pub fn len(&self) -> usize
pub fn is_empty(&self) -> bool
```

### 模块级函数
```rust
pub fn load_saved_uid() -> Result<u64>
pub fn save_uid() -> Result<()>
pub fn init() -> Result<()>
```

### 全局常量
```rust
pub static STUDENT_UID_COUNTER: AtomicU64
```

---

## 2. 现金流管理模块 (cash.rs)

### Cash 结构体

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Cash {
    pub uid: u64,                           // 现金记录唯一ID
    pub student_id: Option<u64>,            // 关联学生ID（可选）
    pub cash: i64,                          // 现金金额（正数收入，负数支出）
    pub note: Option<String>,               // 备注信息
    pub installment: Option<Installment>,   // 分期付款信息（可选）
}
```

### Installment 结构体

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Installment {
    pub plan_id: u64,                    // 分期计划ID
    pub total_amount: i64,               // 总金额
    pub total_installments: u32,         // 总期数
    pub current_installment: u32,        // 当前期数
    pub frequency: PaymentFrequency,     // 付款频率
    pub due_date: DateTime<Utc>,         // 到期日期
    pub status: InstallmentStatus,       // 付款状态
}
```

#### 相关枚举
```rust
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum PaymentFrequency {
    Weekly,           // 每周
    Monthly,          // 每月
    Quarterly,        // 每季度
    Custom(u32),      // 自定义天数
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum InstallmentStatus {
    Pending,    // 待付款
    Paid,       // 已付款
    Overdue,    // 逾期
    Cancelled,  // 已取消
}
```

### Cash API 方法

#### 构造方法
```rust
pub fn new(student_id: Option<u64>) -> Self
pub fn new_installment(
    student_id: Option<u64>,
    total_amount: i64,
    total_installments: u32,
    frequency: PaymentFrequency,
    due_date: DateTime<Utc>,
    current_installment: u32,
    plan_id: Option<u64>
) -> Self
```

#### 基础操作方法
```rust
pub fn add(&mut self, num: i64)
pub fn set_cash(&mut self, num: i64)
pub fn set_id(&mut self, id: u64)
pub fn set_note(&mut self, note: Option<String>)
pub fn note(&self) -> Option<&str>
pub fn is_installment(&self) -> bool
pub fn installment_plan_id(&self) -> Option<u64>
pub fn set_installment_status(&mut self, status: InstallmentStatus)
```

### CashDatabase 结构体

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CashDatabase {
    pub cash_data: BTreeMap<u64, Cash>,
}
```

#### CashDatabase API 方法

**基础操作** (实现了 `Database<Cash>` trait ✨新增✨)
```rust
// 构造和序列化
pub fn new() -> Self
pub fn from_json(json_str: &str) -> Result<Self>
pub fn json(&self) -> String

// 持久化
pub fn save(&self) -> Result<()>                        // ✨改进✨ 现在返回 anyhow::Result
pub fn save_to(&self, path: &str) -> Result<()>
pub fn read_from(path: &str) -> Result<Self>

// CRUD操作
pub fn insert(&mut self, cash: Cash)
pub fn insert_batch(&mut self, cash_records: Vec<Cash>) -> usize
pub fn update_batch<F>(&mut self, uids: &[u64], update_fn: F) -> usize
pub fn get(&self, index: &u64) -> Option<&Cash>
pub fn remove(&mut self, uid: &u64) -> Option<Cash>
pub fn remove_batch(&mut self, uids: &[u64]) -> usize

// 查询和迭代
pub fn iter(&self) -> impl Iterator<Item = (&u64, &Cash)>
pub fn len(&self) -> usize
pub fn is_empty(&self) -> bool
```

#### 分期付款专用方法
```rust
pub fn get_installments(&self) -> Vec<&Cash>
pub fn get_installments_by_plan(&self, plan_id: u64) -> Vec<&Cash>
pub fn get_overdue_installments(&self) -> Vec<&Cash>
pub fn get_student_installments(&self, student_id: u64) -> Vec<&Cash>
pub fn generate_next_installment(&mut self, plan_id: u64, due_date: DateTime<Utc>) -> Result<u64>
pub fn cancel_installment_plan(&mut self, plan_id: u64) -> usize
```

### 模块级函数
```rust
pub fn load_saved_cash_uid() -> Result<u64>
pub fn save_uid() -> Result<()>
pub fn init() -> Result<()>
```

### 全局常量
```rust
pub static CASH_UID_COUNTER: AtomicU64
```

---

## 3. 通用数据库接口 (common.rs) ✨新增✨

### Database<T> Trait

为所有数据库提供统一的操作接口：

```rust
pub trait Database<T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    // 核心数据访问
    fn data(&self) -> &BTreeMap<u64, T>;
    fn data_mut(&mut self) -> &mut BTreeMap<u64, T>;
    
    // 配置信息
    fn default_path(&self) -> &'static str;
    fn type_name(&self) -> &'static str;
    fn static_type_name() -> &'static str;
    
    // 统一的CRUD操作
    fn new() -> Self;
    fn insert(&mut self, item: T) where T: HasUid;
    fn insert_batch(&mut self, items: Vec<T>) -> usize where T: HasUid;
    fn update_batch<F>(&mut self, uids: &[u64], update_fn: F) -> usize;
    fn get(&self, uid: &u64) -> Option<&T>;
    fn remove(&mut self, uid: &u64) -> Option<T>;
    fn remove_batch(&mut self, uids: &[u64]) -> usize;
    
    // 查询和序列化
    fn iter(&self) -> impl Iterator<Item = (&u64, &T)> + '_;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn json(&self) -> String where Self: Serialize;
    fn from_json(json: &str) -> Result<Self> where Self: DeserializeOwned;
    
    // 持久化操作
    fn save(&self) -> Result<()> where Self: Serialize;
    fn save_to(&self, path: &str) -> Result<()> where Self: Serialize;
    fn read_from(path: &str) -> Result<Self> where Self: DeserializeOwned;
}
```

### HasUid Trait

```rust
pub trait HasUid {
    fn uid(&self) -> u64;
}
```

---

## 4. 数据库容器模块 (database.rs)

### Database 结构体

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Database {
    pub student: StudentDatabase,
    pub cash: CashDatabase,
}
```

### Database API 方法

```rust
pub fn new(student: StudentDatabase, cash: CashDatabase) -> Self
pub fn save(&self) -> Result<()>                        // ✨改进✨ 统一错误处理
```

### 模块级函数
```rust
pub fn init() -> Result<Database>                       // ✨改进✨ 统一错误处理
pub fn save(db: &Database) -> Result<()>
```

---

## 5. 统计分析模块 (stats.rs)

### DashboardStats 结构体

```rust
#[derive(Serialize, Debug)]
pub struct DashboardStats {
    pub total_students: usize,
    pub total_revenue: i64,
    pub total_expense: i64,
    pub average_score: f64,
    pub max_score: f64,
    pub active_courses: usize,
}
```

### 统计函数
```rust
pub fn get_dashboard_stats(
    student_db: &StudentDatabase, 
    cash_db: &CashDatabase
) -> Result<DashboardStats>
```

---

## 6. 系统初始化模块 (init.rs)

### 初始化函数 ✨改进✨
```rust
pub fn init() -> Result<Database>                       // 现在返回 anyhow::Result
```
- 初始化学生和现金模块的UID计数器
- 加载或创建数据库文件
- 返回完整的运行时数据库实例

---

## 7. 数据持久化模块 (save.rs)

### 保存函数 ✨改进✨
```rust
pub fn save(database: Database) -> Result<()>           // 现在返回 anyhow::Result
```
- 保存所有UID计数器到文件
- 原子性保存数据库到文件系统
- 确保数据一致性

---

## 8. 新公共API入口 (manager.rs) ✨新增✨

### QmxManager 概览
```rust
pub struct QmxManager
impl QmxManager {
    pub fn new(auto_save: bool) -> Result<Self>
    pub fn from_path(student_path: &str, cash_path: &str, auto_save: bool) -> Result<Self>
    pub fn save(&self) -> Result<()>
}
```

### 学生管理
```rust
impl QmxManager {
    pub fn create_student(&self, builder: StudentBuilder) -> Result<u64>
    pub fn get_student(&self, uid: u64) -> Result<Option<Student>>
    pub fn update_student(&self, uid: u64, updater: StudentUpdater) -> Result<()>
    pub fn delete_student(&self, uid: u64) -> Result<bool>
    pub fn search_students(&self, query: StudentQuery) -> Result<Vec<Student>>
    pub fn list_students(&self) -> Result<Vec<Student>>
}
```

#### StudentBuilder
```rust
pub struct StudentBuilder
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

#### StudentUpdater
```rust
pub struct StudentUpdater
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
    pub fn membership(self, start: Option<DateTime<Utc>>, end: Option<DateTime<Utc>>) -> Self
}
```

#### StudentQuery
```rust
pub struct StudentQuery
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

### 现金管理
```rust
impl QmxManager {
    pub fn record_cash(&self, builder: CashBuilder) -> Result<u64>
    pub fn get_cash(&self, uid: u64) -> Result<Option<Cash>>
    pub fn update_cash(&self, uid: u64, updater: CashUpdater) -> Result<()>
    pub fn delete_cash(&self, uid: u64) -> Result<bool>
    pub fn search_cash(&self, query: CashQuery) -> Result<Vec<Cash>>
    pub fn get_student_cash(&self, student_id: u64) -> Result<Vec<Cash>>
}
```

#### CashBuilder
```rust
pub struct CashBuilder
impl CashBuilder {
    pub fn new(amount: i64) -> Self
    pub fn student_id(self, student_id: u64) -> Self
    pub fn note(self, note: impl Into<String>) -> Self
    pub fn installment(self, installment: Installment) -> Self
}
```

#### CashUpdater
```rust
pub struct CashUpdater
impl CashUpdater {
    pub fn new() -> Self
    pub fn student_id(self, student_id: Option<u64>) -> Self
    pub fn amount(self, amount: i64) -> Self
    pub fn note(self, note: Option<String>) -> Self
    pub fn installment(self, installment: Option<Installment>) -> Self
}
```

#### CashQuery
```rust
pub struct CashQuery
impl CashQuery {
    pub fn new() -> Self
    pub fn student_id(self, student_id: u64) -> Self
    pub fn amount_range(self, min: i64, max: i64) -> Self
    pub fn has_installment(self, has: bool) -> Self
}
```

### 统计分析
```rust
impl QmxManager {
    pub fn get_dashboard_stats(&self) -> Result<DashboardStats>
    pub fn get_student_stats(&self, uid: u64) -> Result<StudentStats>
    pub fn get_financial_stats(&self, period: TimePeriod) -> Result<FinancialStats>
}
```

### 示例（节选）
```rust
let manager = QmxManager::new(true)?;
let uid = manager.create_student(
    StudentBuilder::new("张三", 16).class(Class::TenTry)
)?;
let cash_id = manager.record_cash(CashBuilder::new(5000).student_id(uid))?;
let students = manager.search_students(StudentQuery::new().age_range(15, 18))?;
```

---

## 错误处理 ✨改进✨

**统一错误处理：** 所有模块现在都使用 `anyhow::Result<T>` 进行错误处理，提供丰富的错误上下文信息。

---

## 使用示例

### 快速开始

```rust
use qmx_backend_lib::*;
use chrono::{Utc, Duration};

fn main() -> anyhow::Result<()> {
    // 1. 初始化系统
    let mut db = init::init()?;

    // 2. 创建学生并设置会员信息
    let mut student = Student::new();
    student
        .set_name("张三".to_string())
        .set_age(18)
        .set_class(Class::TenTry)
        .set_subject(Subject::Shooting)
        .add_ring(9.5)
        .set_membership_dates(
            Some(Utc::now()),
            Some(Utc::now() + Duration::days(365))
        );

    let student_id = student.uid();
    db.student.insert(student);

    // 3. 添加现金记录
    let mut cash = Cash::new(Some(student_id));
    cash.set_cash(1000);
    cash.set_note(Some("学费收入".to_string()));
    db.cash.insert(cash);

    // 4. 创建分期付款
    let installment = Cash::new_installment(
        Some(student_id),
        3000,
        3,
        PaymentFrequency::Monthly,
        Utc::now() + Duration::days(30),
        1,
        None
    );
    db.cash.insert(installment);

    // 5. 生成统计报告
    let stats = get_dashboard_stats(&db.student, &db.cash)?;
    println!("统计数据: {:?}", stats);

    // 6. 保存数据
    save::save(db)?;
    
    Ok(())
}
```

### 会员管理示例 ✨新增✨

```rust
use qmx_backend_lib::*;
use chrono::{Utc, Duration};

fn membership_example() -> anyhow::Result<()> {
    let mut db = init::init()?;
    
    // 创建会员学生
    let mut student = Student::new();
    student
        .set_name("李四".to_string())
        .set_membership_start_date(Utc::now())
        .set_membership_end_date(Utc::now() + Duration::days(30));
    
    // 检查会员状态
    if student.is_membership_active() {
        println!("会员有效");
        if let Some(days) = student.membership_days_remaining() {
            println!("剩余 {} 天", days);
        }
    }
    
    // 延长会员期限
    student.set_membership_end_date(Utc::now() + Duration::days(60));
    
    db.student.insert(student);
    save::save(db)?;
    
    Ok(())
}
```

### 分期付款管理示例

```rust
use qmx_backend_lib::*;
use chrono::{Utc, Duration};

fn installment_example() -> anyhow::Result<()> {
    let mut db = init::init()?;
    
    // 查询逾期分期
    let overdue = db.cash.get_overdue_installments();
    for cash in overdue {
        println!("逾期分期: UID={}, 金额={}", cash.uid, cash.cash);
        
        // 更新状态为逾期
        let uid = cash.uid;
        db.cash.update_batch(&[uid], |c| {
            c.set_installment_status(InstallmentStatus::Overdue);
            true
        });
    }

    // 生成下期付款
    let plan_id = 1001;
    let next_due = Utc::now() + Duration::days(30);
    match db.cash.generate_next_installment(plan_id, next_due) {
        Ok(next_uid) => println!("生成下期付款，UID: {}", next_uid),
        Err(e) => println!("生成失败: {}", e),
    }

    // 取消分期计划
    let cancelled_count = db.cash.cancel_installment_plan(plan_id);
    println!("取消了 {} 条分期记录", cancelled_count);
    
    save::save(db)?;
    Ok(())
}
```

### 使用通用Database Trait ✨新增✨

```rust
use qmx_backend_lib::*;
use qmx_backend_lib::common::Database;

fn generic_database_example() -> anyhow::Result<()> {
    // 使用统一的trait接口
    let mut student_db = StudentDatabase::new();
    let mut cash_db = CashDatabase::new();
    
    // 统一的操作方式
    println!("学生数据库记录数: {}", student_db.len());
    println!("现金数据库记录数: {}", cash_db.len());
    
    // 批量操作
    let student_uids = vec![1, 2, 3];
    let updated = student_db.update_batch(&student_uids, |student| {
        student.set_note("批量更新".to_string());
        true
    });
    println!("批量更新了 {} 个学生记录", updated);
    
    // 统一的序列化
    let json = student_db.json();
    let restored = StudentDatabase::from_json(&json)?;
    
    Ok(())
}
```

---

## 最佳实践

### 1. 错误处理
```rust
// ✅ 推荐：使用 ? 操作符传播错误
fn example() -> anyhow::Result<()> {
    let db = init::init()?;
    save::save(db)?;
    Ok(())
}

// ❌ 避免：忽略错误
fn bad_example() {
    let db = init::init().unwrap(); // 可能panic
}
```

### 2. 会员管理
```rust
// ✅ 推荐：检查会员状态再执行操作
if student.is_membership_active() {
    // 执行会员专属操作
    student.add_ring(score);
}

// ✅ 推荐：定期检查即将到期的会员
for (_, student) in db.student.iter() {
    if let Some(days) = student.membership_days_remaining() {
        if days <= 7 {
            println!("会员 {} 即将到期", student.name());
        }
    }
}
```

### 3. 数据持久化
```rust
// ✅ 推荐：定期保存数据
fn periodic_save(db: Database) -> anyhow::Result<()> {
    // 执行业务操作...
    
    // 保存数据
    save::save(db)?;
    Ok(())
}
```

---

## 迁移指南 (v1.1.0 → v1.2.0)

### 无需更改的代码
现有的所有API调用都保持向后兼容，无需修改。

### 可选的改进
```rust
// 旧方式（仍然有效）
let rings_vec: &Vec<f64> = student.rings();

// 新方式（推荐）
let rings_slice: &[f64] = student.rings();

// 错误处理改进（推荐）
match init::init() {
    Ok(db) => { /* 处理成功 */ },
    Err(e) => println!("初始化失败: {}", e), // 更丰富的错误信息
}
```

---

*文档版本：2.1.0*  
*对应代码版本：2.0.0*  
*最后更新：2025-09-13*