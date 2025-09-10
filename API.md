# QMX Backend Library API 完整文档

## 概述

QMX Backend Library 是一个用 Rust 编写的学生管理和财务管理库，提供完整的学生信息管理、现金流记录、分期付款处理和统计分析功能。

## 模块结构

- **student.rs** - 学生信息管理
- **cash.rs** - 现金流和分期付款管理  
- **database.rs** - 运行时数据库容器
- **stats.rs** - 统计分析
- **init.rs** - 系统初始化
- **save.rs** - 数据持久化

---

## 1. 学生管理模块 (student.rs)

### Student 结构体

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Student {
    uid: u64,                 // 唯一标识符（自动生成）
    age: u8,                  // 年龄（0-255）
    name: String,             // 姓名（UTF-8字符串）
    phone: String,            // 电话号码
    lesson_left: Option<u32>, // 剩余课时（仅TenTry班级有效）
    class: Class,             // 班级类型枚举
    subject: Subject,         // 科目类型枚举
    rings: Vec<f64>,         // 成绩记录列表
    note: String,             // 备注文本
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
- 默认值：`age=0`, `name/phone="未填写"`, `class/subject=Others`, 空列表/字符串

#### 字段设置方法（链式调用）
```rust
pub fn set_age(&mut self, age: u8) -> &mut Self
pub fn set_name(&mut self, name: String) -> &mut Self
pub fn set_phone(&mut self, phone: String) -> &mut Self
pub fn set_class(&mut self, class: Class) -> &mut Self
pub fn set_lesson_left(&mut self, lesson: u32) -> &mut Self
pub fn set_subject(&mut self, subject: Subject) -> &mut Self
pub fn set_note(&mut self, note: String) -> &mut Self
pub unsafe fn set_id(&mut self, id: u64) -> &mut Self  // 强制覆盖UID
pub fn add_ring(&mut self, ring: f64) -> &mut Self     // 追加成绩记录
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
pub fn rings(&self) -> &Vec<f64>
pub fn note(&self) -> &str
```

### StudentDatabase 结构体

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StudentDatabase {
    pub student_data: BTreeMap<u64, Student>,
}
```

#### StudentDatabase API 方法

```rust
// 构造和序列化
pub fn new() -> Self
pub fn from_json(json: &str) -> Result<Self>
pub fn json(&self) -> String

// 持久化
pub fn save(&self) -> Result<()>
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
pub fn load_saved_uid() -> Result<u64>  // 加载UID计数器
pub fn save_uid() -> Result<()>         // 保存UID计数器
pub fn init() -> Result<()>             // 初始化学生模块
```

### 全局常量
```rust
pub static STUDENT_UID_COUNTER: AtomicU64  // 全局UID计数器
```

---

## 2. 现金流管理模块 (cash.rs)

### Cash 结构体

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Cash {
    uid: u64,                           // 现金记录唯一ID
    student_id: Option<u64>,            // 关联学生ID（可选）
    cash: i64,                          // 现金金额（正数收入，负数支出）
    note: Option<String>,               // 备注信息
    installment: Option<Installment>,   // 分期付款信息（可选）
}
```

### Installment 结构体

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Installment {
    plan_id: u64,                    // 分期计划ID
    total_amount: i64,               // 总金额
    total_installments: u32,         // 总期数
    current_installment: u32,        // 当前期数
    frequency: PaymentFrequency,     // 付款频率
    due_date: DateTime<Utc>,         // 到期日期
    status: InstallmentStatus,       // 付款状态
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
    plan_id: u64,
    total_amount: i64,
    total_installments: u32,
    current_installment: u32,
    frequency: PaymentFrequency,
    due_date: DateTime<Utc>
) -> Self
```

#### 基础操作方法
```rust
pub fn add(&mut self, num: i64)                              // 增加金额
pub fn set_cash(&mut self, num: i64)                         // 设置金额
pub fn set_id(&mut self, id: u64)                           // 设置学生ID
pub fn set_note(&mut self, note: Option<String>)            // 设置备注
pub fn note(&self) -> Option<&str>                          // 获取备注
pub fn is_installment(&self) -> bool                        // 是否为分期付款
pub fn installment_plan_id(&self) -> Option<u64>            // 获取分期计划ID
pub fn set_installment_status(&mut self, status: InstallmentStatus)  // 设置分期状态
```

### CashDatabase 结构体

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CashDatabase {
    pub cash_data: BTreeMap<u64, Cash>,
}
```

#### CashDatabase API 方法

```rust
// 构造和序列化
pub fn new() -> Self
pub fn from_json(json_str: &str) -> Result<Self>
pub fn json(&self) -> String

// 持久化
pub fn save(&self) -> Result<()>
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
pub fn get_installments(&self) -> Vec<&Cash>                           // 获取所有分期记录
pub fn get_installments_by_plan(&self, plan_id: u64) -> Vec<&Cash>     // 按计划ID获取分期
pub fn get_overdue_installments(&self) -> Vec<&Cash>                   // 获取逾期分期
pub fn get_student_installments(&self, student_id: u64) -> Vec<&Cash>  // 按学生获取分期
pub fn generate_next_installment(&mut self, plan_id: u64, due_date: DateTime<Utc>) -> Result<u64>  // 生成下期付款
pub fn cancel_installment_plan(&mut self, plan_id: u64) -> usize       // 取消分期计划
```

### 模块级函数
```rust
pub fn load_saved_cash_uid() -> Result<u64>  // 加载现金UID计数器
pub fn save_uid() -> Result<()>              // 保存现金UID计数器
pub fn init() -> Result<()>                  // 初始化现金模块
```

### 全局常量
```rust
pub static CASH_UID_COUNTER: AtomicU64  // 全局现金UID计数器
```

---

## 3. 数据库容器模块 (database.rs)

### Database 结构体

```rust
#[derive(Debug)]
pub struct Database {
    pub student: StudentDatabase,  // 学生数据库
    pub cash: CashDatabase,        // 现金数据库
}
```

### Database API 方法

```rust
pub fn new(student: StudentDatabase, cash: CashDatabase) -> Self  // 构造数据库
pub fn save(&self) -> Result<()>                                 // 保存所有子数据库
```

### 模块级函数
```rust
pub fn init() -> Result<Database>        // 初始化运行时数据库
pub fn save(db: &Database) -> Result<()> // 保存数据库
```

---

## 4. 统计分析模块 (stats.rs)

### DashboardStats 结构体

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DashboardStats {
    pub total_students: usize,    // 学生总数
    pub total_revenue: i64,       // 总收入
    pub total_expense: i64,       // 总支出
    pub average_score: f64,       // 平均成绩
    pub max_score: f64,           // 最高成绩
    pub active_courses: usize,    // 活跃课程数
}
```

### 统计函数
```rust
pub fn get_dashboard_stats(
    student_db: &StudentDatabase, 
    cash_db: &CashDatabase
) -> Result<DashboardStats>
```
- 计算仪表板统计数据
- 分析学生成绩和财务状况
- 返回完整的统计报告

---

## 5. 系统初始化模块 (init.rs)

### 初始化函数
```rust
pub fn init() -> Result<Database, String>
```
- 初始化整个系统
- 加载所有数据库和计数器
- 返回完整的运行时数据库实例
- 错误时返回字符串描述

---

## 6. 数据持久化模块 (save.rs)

### 保存函数
```rust
pub fn save(database: Database) -> Result<(), String>
```
- 保存完整数据库到文件系统
- 同步保存所有UID计数器
- 确保数据一致性
- 错误时返回字符串描述

---

## 错误处理

大部分模块使用 `anyhow::Result<T>` 进行错误处理，提供丰富的错误上下文信息。

特殊情况：
- `init.rs` 和 `save.rs` 返回 `Result<T, String>` 以提供简化的错误消息

---

## 使用示例

### 完整工作流程示例

```rust
use qmx_backend_lib::*;

// 1. 初始化系统
let mut db = init::init()?;

// 2. 创建学生
let student = Student::new()
    .set_name("张三".into())
    .set_age(18)
    .set_class(Class::TenTry)
    .set_subject(Subject::Shooting)
    .add_ring(9.5)
    .clone();

let student_id = student.uid();
db.student.insert(student);

// 3. 添加现金记录
let cash = Cash::new(Some(student_id))
    .set_cash(1000)
    .set_note(Some("学费收入".into()));
db.cash.insert(cash);

// 4. 创建分期付款
let installment = Cash::new_installment(
    Some(student_id),
    1001,
    3000,
    3,
    1,
    PaymentFrequency::Monthly,
    Utc::now() + Duration::days(30)
);
db.cash.insert(installment);

// 5. 生成统计报告
let stats = get_dashboard_stats(&db.student, &db.cash)?;
println!("统计数据: {:?}", stats);

// 6. 保存数据
save::save(db)?;
```

### 分期付款管理示例

```rust
// 查询逾期分期
let overdue = db.cash.get_overdue_installments();
for cash in overdue {
    println!("逾期分期: UID={}, 金额={}", cash.uid(), cash.cash);
}

// 生成下期付款
let next_due = Utc::now() + Duration::days(30);
let next_uid = db.cash.generate_next_installment(1001, next_due)?;
println!("生成下期付款，UID: {}", next_uid);

// 取消分期计划
let cancelled_count = db.cash.cancel_installment_plan(1001);
println!("取消了 {} 条分期记录", cancelled_count);
```

---

*文档版本：1.0.0*  
*对应代码版本：1.0.0*  
*最后更新：2025-09-10*