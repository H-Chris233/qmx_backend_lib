# QMX Backend Library API 文档

## 概述

QMX Backend Library 是一个用 Rust 编写的后端数据库库，专门用于管理学生和现金记录。该库提供了完整的 CRUD 操作、数据持久化、UID 管理等功能。

### 主要特性

- 🔐 **原子 UID 生成**: 使用原子计数器确保唯一性
- 💾 **持久化存储**: JSON 格式文件存储
- 📊 **完整 CRUD**: 创建、读取、更新、删除操作
- 🔄 **批量操作**: 支持批量删除
- 🛡️ **错误处理**: 使用 anyhow 提供详细的错误上下文
- 📝 **日志记录**: 完整的操作日志
- 🧪 **测试覆盖**: 全面的单元测试

## 模块结构

```
qmx_backend_lib/
├── lib.rs          # 库入口点
├── student.rs      # 学生管理模块
├── cash.rs         # 现金管理模块
├── database.rs     # 数据库容器
├── init.rs         # 初始化协调器
└── save.rs         # 持久化协调器
```

## 学生管理模块 (student.rs)

### Person 结构体

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Person {
    uid: u64,                    // 唯一标识符
    age: u8,                     // 年龄
    name: String,                // 姓名
    lesson_left: Option<u32>,    // 剩余课程数
    class: Class,                // 班级类型
    rings: Vec<f64>,             // 测量数据
    note: String,                // 备注
    cash: (i32, i32),           // 现金余额
}
```

#### 构造函数

```rust
// 创建新的 Person 实例，自动生成 UID
pub fn new() -> Self
```

**示例:**
```rust
let student = Person::new();
```

#### Student Trait 方法

##### 设置方法

```rust
// 设置年龄
fn set_age(&mut self, age: u8) -> &mut Self

// 设置姓名
fn set_name(&mut self, name: String) -> &mut Self

// 设置班级类型
fn set_class(&mut self, class: Class) -> &mut Self

// 设置剩余课程数
fn set_lesson_left(&mut self, lesson: u32) -> &mut Self

// 添加测量数据
fn add_ring(&mut self, ring: f64) -> &mut Self

// 设置备注
fn set_note(&mut self, note: String) -> &mut Self

// 设置现金余额
fn set_cash(&mut self, cash: (i32, i32)) -> &mut Self

// 强制设置 UID (不安全操作)
unsafe fn set_id(&mut self, id: u64) -> &mut Self
```

**示例:**
```rust
let mut student = Person::new();
student.set_age(25)
    .set_name("张三".to_string())
    .set_class(Class::TenTry)
    .set_note("优秀学生".to_string());
```

##### 获取方法

```rust
fn uid(&self) -> u64
fn age(&self) -> u8
fn name(&self) -> &str
fn lesson_left(&self) -> Option<u32>
fn class(&self) -> &Class
fn rings(&self) -> &Vec<f64>
fn note(&self) -> &str
fn cash(&self) -> (i32, i32)
```

**示例:**
```rust
println!("学生姓名: {}", student.name());
println!("学生年龄: {}", student.age());
println!("剩余课程: {:?}", student.lesson_left());
```

### Class 枚举

```rust
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Class {
    TenTry,    // 十次体验课
    Month,     // 月卡
    Year,      // 年卡
    Others,    // 其他
}
```

### StudentDatabase 结构体

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StudentDatabase {
    pub student_data: BTreeMap<u64, Person>,
}
```

#### 构造函数

```rust
// 创建空的数据库
pub fn new() -> Self

// 从 JSON 字符串创建数据库
pub fn from_json(json: &str) -> Result<Self>
```

**示例:**
```rust
let mut db = StudentDatabase::new();
let db_from_json = StudentDatabase::from_json(json_str)?;
```

#### CRUD 操作

```rust
// 插入学生记录
pub fn insert(&mut self, person: Person)

// 根据 UID 获取学生记录
pub fn get(&self, index: &u64) -> Option<&Person>

// 删除指定 UID 的学生记录
pub fn remove(&mut self, uid: &u64) -> Option<Person>

// 批量删除学生记录
pub fn remove_batch(&mut self, uids: &[u64]) -> usize
```

**示例:**
```rust
// 插入
let student = Person::new();
db.insert(student);

// 查询
if let Some(student) = db.get(&uid) {
    println!("找到学生: {}", student.name());
}

// 删除
if let Some(removed) = db.remove(&uid) {
    println!("已删除学生: {}", removed.name());
}

// 批量删除
let count = db.remove_batch(&[uid1, uid2, uid3]);
println!("删除了 {} 个学生记录", count);
```

#### 查询和统计

```rust
// 获取迭代器
pub fn iter(&self) -> impl Iterator<Item = (&u64, &Person)> + '_

// 获取记录数量
pub fn len(&self) -> usize

// 检查是否为空
pub fn is_empty(&self) -> bool
```

**示例:**
```rust
// 遍历所有学生
for (uid, student) in db.iter() {
    println!("UID: {}, 姓名: {}", uid, student.name());
}

// 统计信息
println!("学生总数: {}", db.len());
println!("数据库是否为空: {}", db.is_empty());
```

#### 持久化操作

```rust
// 保存到默认路径
pub fn save(&self) -> Result<()>

// 保存到指定路径
pub fn save_to(&self, path: &str) -> Result<()>

// 从指定路径读取
pub fn read_from(path: &str) -> Result<Self>

// 序列化为 JSON
pub fn json(&self) -> String
```

**示例:**
```rust
// 保存数据库
db.save()?;  // 保存到 ./data/student_database.json
db.save_to("/custom/path/students.json")?;

// 读取数据库
let db = StudentDatabase::read_from("./data/student_database.json")?;

// JSON 序列化
let json_string = db.json();
```

### 全局函数

```rust
// 加载已保存的 UID 计数器
pub fn load_saved_uid() -> Result<u64>

// 保存 UID 计数器
pub fn save_uid() -> Result<()>

// 初始化学生模块
pub fn init() -> Result<()>
```

## 现金管理模块 (cash.rs)

### Cash 结构体

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Cash {
    pub uid: u64,           // 现金记录的唯一标识符
    pub student_id: Option<u64>,  // 关联的学生 UID
    pub cash: i32,          // 金额
}
```

#### 构造函数

```rust
// 创建新的现金记录
pub fn new(student_id: Option<u64>) -> Self
```

**示例:**
```rust
// 创建不关联学生的现金记录
let cash_record = Cash::new(None);

// 创建关联学生的现金记录
let student_cash = Cash::new(Some(123));
```

#### 操作方法

```rust
// 添加金额
pub fn add(&mut self, num: i32)

// 设置金额
pub fn set_cash(&mut self, num: i32)

// 设置关联的学生 ID
pub fn set_id(&mut self, id: u64)
```

**示例:**
```rust
let mut cash = Cash::new(Some(123));
cash.add(100);      // 增加 100
cash.set_cash(200); // 设置为 200
cash.set_id(456);   // 关联到另一个学生
```

### CashDatabase 结构体

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CashDatabase {
    pub cash_data: BTreeMap<u64, Cash>,
}
```

#### 构造函数

```rust
// 创建空的现金数据库
pub fn new() -> Self
```

#### CRUD 操作

```rust
// 插入现金记录
pub fn insert(&mut self, cash: Cash)

// 根据 UID 获取现金记录
pub fn get(&self, index: &u64) -> Option<&Cash>

// 删除指定 UID 的现金记录
pub fn remove(&mut self, uid: &u64) -> Option<Cash>

// 批量删除现金记录
pub fn remove_batch(&mut self, uids: &[u64]) -> usize
```

**示例:**
```rust
let mut db = CashDatabase::new();

// 插入
let cash = Cash::new(Some(123));
db.insert(cash);

// 查询
if let Some(record) = db.get(&uid) {
    println!("金额: {}", record.cash);
}

// 删除
db.remove(&uid);

// 批量删除
let count = db.remove_batch(&[uid1, uid2]);
```

#### 查询和统计

```rust
// 获取迭代器
pub fn iter(&self) -> impl Iterator<Item = (&u64, &Cash)> + '_

// 获取记录数量
pub fn len(&self) -> usize

// 检查是否为空
pub fn is_empty(&self) -> bool
```

#### 持久化操作

```rust
// 保存到默认路径
pub fn save(&self) -> Result<()>

// 保存到指定路径
pub fn save_to(&self, path: &str) -> Result<()>

// 从指定路径读取
pub fn read_from(path: &str) -> Result<Self>
```

### 全局函数

```rust
// 加载已保存的现金 UID 计数器
pub fn load_saved_cash_uid() -> Result<u64>

// 保存现金 UID 计数器
pub fn save_uid() -> Result<()>

// 初始化现金模块
pub fn init() -> Result<()>
```

## 数据库容器模块 (database.rs)

### Database 结构体

```rust
pub struct Database {
    pub student_db: StudentDatabase,
    pub cash_db: CashDatabase,
}
```

#### 主要方法

```rust
// 创建新的数据库容器
pub fn new() -> Self

// 保存所有数据
pub fn save_all(&self) -> Result<()>

// 加载所有数据
pub fn load_all() -> Result<Self>
```

## 初始化模块 (init.rs)

### 系统初始化

```rust
// 初始化整个系统
pub fn init_all() -> Result<()>
```

**初始化顺序:**
1. 初始化学生模块 (加载 UID 计数器)
2. 初始化现金模块 (加载 UID 计数器)
3. 初始化数据库容器 (加载数据)

## 持久化模块 (save.rs)

### 数据保存

```rust
// 保存所有数据和计数器
pub fn save_all() -> Result<()>
```

## 使用示例

### 完整的学生管理示例

```rust
use qmx_backend_lib::{Person, StudentDatabase, Class};

fn main() -> anyhow::Result<()> {
    // 初始化系统
    qmx_backend_lib::init_all()?;
    
    // 创建数据库
    let mut db = StudentDatabase::new();
    
    // 创建学生
    let mut student1 = Person::new();
    student1.set_name("张三".to_string())
            .set_age(20)
            .set_class(Class::TenTry);
    
    let mut student2 = Person::new();
    student2.set_name("李四".to_string())
            .set_age(22)
            .set_class(Class::Month);
    
    // 插入数据库
    db.insert(student1);
    db.insert(student2);
    
    // 查询学生
    for (uid, student) in db.iter() {
        println!("UID: {}, 姓名: {}, 年龄: {}", 
                 uid, student.name(), student.age());
    }
    
    // 保存数据
    db.save()?;
    
    Ok(())
}
```

### 完整的现金管理示例

```rust
use qmx_backend_lib::{Cash, CashDatabase};

fn main() -> anyhow::Result<()> {
    // 初始化系统
    qmx_backend_lib::init_all()?;
    
    // 创建数据库
    let mut db = CashDatabase::new();
    
    // 创建现金记录
    let mut cash1 = Cash::new(Some(123));  // 关联学生 UID 123
    cash1.add(1000);
    
    let mut cash2 = Cash::new(None);       // 不关联学生
    cash2.add(500);
    
    // 插入数据库
    db.insert(cash1);
    db.insert(cash2);
    
    // 查询记录
    for (uid, cash) in db.iter() {
        match cash.student_id {
            Some(student_id) => {
                println!("UID: {}, 学生ID: {}, 金额: {}", 
                         uid, student_id, cash.cash);
            }
            None => {
                println!("UID: {}, 独立记录, 金额: {}", 
                         uid, cash.cash);
            }
        }
    }
    
    // 保存数据
    db.save()?;
    
    Ok(())
}
```

### 批量操作示例

```rust
use qmx_backend_lib::{Person, StudentDatabase};

fn main() -> anyhow::Result<()> {
    let mut db = StudentDatabase::new();
    
    // 批量插入学生
    let mut students = Vec::new();
    for i in 0..10 {
        let mut student = Person::new();
        student.set_name(format!("学生{}", i))
                .set_age(20 + i as u8);
        students.push(student);
    }
    
    for student in students {
        db.insert(student);
    }
    
    // 批量删除
    let uids_to_remove: Vec<u64> = db.iter()
        .take(5)
        .map(|(&uid, _)| uid)
        .collect();
    
    let removed_count = db.remove_batch(&uids_to_remove);
    println!("删除了 {} 个学生记录", removed_count);
    
    Ok(())
}
```

## 错误处理

库使用 `anyhow` crate 提供详细的错误信息。所有可能失败的操作都返回 `Result<T>`。

### 错误处理示例

```rust
use anyhow::Result;

fn safe_student_operations() -> Result<()> {
    let mut db = StudentDatabase::new();
    
    // 尝试从文件读取数据库
    match StudentDatabase::read_from("./data/student_database.json") {
        Ok(loaded_db) => {
            println!("成功加载数据库，包含 {} 个学生", loaded_db.len());
            db = loaded_db;
        }
        Err(e) => {
            println!("加载数据库失败: {}", e);
            println!("创建新的数据库");
        }
    }
    
    // 尝试保存数据库
    if let Err(e) = db.save() {
        eprintln!("保存数据库失败: {}", e);
        return Err(e);
    }
    
    Ok(())
}
```

## 文件存储结构

### 数据文件位置

```
./data/
├── student_database.json    # 学生数据库
├── cash_database.json       # 现金数据库
├── uid_counter              # 学生 UID 计数器
└── cash_uid_counter         # 现金 UID 计数器
```

### 文件格式

#### 学生数据库 JSON 格式
```json
{
  "student_data": {
    "1": {
      "uid": 1,
      "age": 20,
      "name": "张三",
      "lesson_left": 10,
      "class": "TenTry",
      "rings": [85.5, 87.0, 86.5],
      "note": "优秀学生",
      "cash": [1000, 500]
    }
  }
}
```

#### 现金数据库 JSON 格式
```json
{
  "cash_data": {
    "1": {
      "uid": 1,
      "student_id": 123,
      "cash": 1000
    }
  }
}
```

#### UID 计数器格式
```
1001
```

## 性能考虑

### 时间复杂度

- **插入**: O(log n)
- **查询**: O(log n)
- **删除**: O(log n)
- **遍历**: O(n)

### 内存使用

- 使用 BTreeMap 存储数据，内存占用与记录数量成线性关系
- 每个记录包含完整的序列化/反序列化开销

### 建议

- 对于小型到中型数据集（< 10,000 条记录）性能良好
- 对于大型数据集，建议考虑分页或索引优化
- 定期清理和备份数据文件

## 最佳实践

### 1. 初始化和清理

```rust
// 应用启动时初始化
qmx_backend_lib::init_all()?;

// 应用关闭时保存
qmx_backend_lib::save_all()?;
```

### 2. 错误处理

```rust
// 始终处理可能的错误
if let Err(e) = db.save() {
    log::error!("保存数据库失败: {}", e);
    // 实施恢复策略
}
```

### 3. 并发安全

```rust
// 在多线程环境中，建议使用互斥锁保护数据库操作
use std::sync::Mutex;

let db = Mutex::new(StudentDatabase::new());

// 在多线程中安全访问
{
    let mut db_guard = db.lock().unwrap();
    db_guard.insert(student);
}
```

### 4. 数据验证

```rust
// 在插入前验证数据
fn validate_student(student: &Person) -> Result<()> {
    if student.age() > 120 {
        return Err(anyhow::anyhow!("年龄不能超过120岁"));
    }
    if student.name().is_empty() {
        return Err(anyhow::anyhow!("姓名不能为空"));
    }
    Ok(())
}
```

## 测试

库包含完整的单元测试，覆盖所有主要功能：

```bash
# 运行所有测试
cargo test

# 运行特定模块的测试
cargo test student::tests

# 运行现金模块测试
cargo test cash::tests
```

### 测试覆盖

- ✅ 创建和修改操作
- ✅ CRUD 操作
- ✅ JSON 序列化/反序列化
- ✅ 批量操作
- ✅ 错误处理

## 版本信息

- **当前版本**: 0.1.0
- **最低 Rust 版本**: 1.56.0
- **依赖版本**: 查看 Cargo.toml 文件

## 贡献指南

1. Fork 项目
2. 创建功能分支
3. 添加测试
4. 确保所有测试通过
5. 提交 Pull Request

## 许可证

查看项目根目录的 LICENSE 文件。

## 联系信息

如有问题或建议，请通过以下方式联系：

- 邮箱: [项目邮箱]
- Issues: [GitHub Issues 链接]
- 文档: [项目文档链接]

---

*最后更新: 2025-08-07*