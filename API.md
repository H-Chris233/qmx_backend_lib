# QMX Backend Library API 文档索引

## 概述

QMX Backend Library 是一个用 Rust 编写的学生管理和财务管理库，提供完整的学生信息管理、现金流记录、分期付款处理和统计分析功能。

**当前版本：** 2.4.0  
**最后更新：** 2025-09-15

## 🔔 重要更新 (v2.2.0)

**⚠️ 注意：** v2.2.0包含重要的API行为变更，请查看 **[📋 变更说明文档](API_CHANGES_v2.2.0.md)** 了解详情。

**主要变更：**
- `Student::set_class` 方法行为变更（拆分副作用）
- `CashBuilder::build()` 现在返回 `Result<Cash>`
- 新增环境变量 `QMX_DATA_DIR` 支持
- 增强的错误处理和数据一致性

---

## 📚 API 版本选择

### [📖 v1 API 文档](API_v1.md)
**传统直接操作API** - 适合需要精细控制的场景

- ✅ **稳定可靠** - 经过充分测试的成熟API
- ✅ **直接操作** - 直接操作数据结构，性能最优
- ✅ **完全控制** - 对数据库操作有完全控制权
- ✅ **批量操作** - 内置高效的批量处理方法
- 🎯 **适用场景**：数据迁移、批量处理、性能敏感应用

**核心组件：**
- `Student` / `StudentDatabase` - 学生管理
- `Cash` / `CashDatabase` - 现金流管理
- `Database` - 数据库容器
- `DashboardStats` - 统计分析
- 通用 `Database<T>` trait

### [🚀 v2 API 文档](API_v2.md)
**现代化统一API** - 推荐用于新项目开发

- 🚀 **现代设计** - Builder、Updater、Query Builder模式
- 🎯 **易于使用** - 统一的`QmxManager`入口
- 🔒 **线程安全** - Arc<RwLock<T>>架构
- 📊 **增强分析** - 丰富的统计和查询功能
- 🎯 **适用场景**：新项目开发、Web应用、多线程环境

**核心组件：**
- `QmxManager` - 统一管理入口
- `StudentBuilder` / `StudentUpdater` / `StudentQuery` - 学生操作
- `CashBuilder` / `CashUpdater` / `CashQuery` - 现金操作
- `StudentStats` / `FinancialStats` - 增强统计

---

## 🔄 版本兼容性

### 完全向后兼容
v2.2.0 **完全兼容** v1 API，现有代码无需修改：

```rust
// v1 API 继续可用
use qmx_backend_lib::*;

let mut db = init::init()?;
let mut student = Student::new();
student.set_name("张三".to_string());
db.student.insert(student);

// v2 API 可同时使用
let manager = QmxManager::new(true)?;
let uid = manager.create_student(
    StudentBuilder::new("李四", 18)
)?;
```

### 渐进式迁移
可以在同一项目中混用两个版本的API，逐步迁移：

1. **第一阶段**：继续使用v1 API，引入v2 API用于新功能
2. **第二阶段**：将部分模块迁移到v2 API
3. **第三阶段**：完全迁移到v2 API（可选）

---

## 🚀 快速开始

### 选择v1 API（传统方式）
```rust
use qmx_backend_lib::*;

fn main() -> anyhow::Result<()> {
    // 初始化数据库
    let mut db = init::init()?;
    
    // 创建学生
    let mut student = Student::new();
    student.set_name("张三".to_string())
           .set_age(18);
    db.student.insert(student);
    
    // 保存数据
    save::save(db)?;
    Ok(())
}
```

### 选择v2 API（推荐）
```rust
use qmx_backend_lib::{QmxManager, StudentBuilder};

fn main() -> anyhow::Result<()> {
    // 初始化管理器
    let manager = QmxManager::new(true)?;
    
    // 创建学生（自动保存）
    let uid = manager.create_student(
        StudentBuilder::new("张三", 18)
    )?;
    
    Ok(())
}
```

---

## 📊 功能对比

| 功能 | v1 API | v2 API | 说明 |
|------|--------|--------|------|
| 学生管理 | ✅ 直接操作 | ✅ Builder模式 | v2更易用 |
| 现金管理 | ✅ 直接操作 | ✅ Builder模式 | v2更易用 |
| 批量操作 | ✅ 高性能 | ⚠️ 通过v1实现 | v1性能更优 |
| 查询功能 | ⚠️ 手动遍历 | ✅ Query Builder | v2更强大 |
| 统计分析 | ✅ 基础统计 | ✅ 增强统计 | v2功能更丰富 |
| 线程安全 | ❌ 需手动处理 | ✅ 内置支持 | v2更安全 |
| 错误处理 | ✅ anyhow::Result | ✅ anyhow::Result | 两者相同 |
| 学习曲线 | 📈 中等 | 📉 简单 | v2更容易上手 |

---

## 🎯 使用建议

### 推荐使用 v2 API 的场景：
- 🆕 **新项目开发**
- 🌐 **Web应用开发**
- 🔄 **多线程环境**
- 👥 **团队协作项目**（API更直观）
- 📊 **需要复杂查询和统计**

### 推荐使用 v1 API 的场景：
- 🔧 **现有项目维护**
- ⚡ **性能敏感应用**
- 📦 **大批量数据处理**
- 🎮 **需要精细控制**
- 🔄 **数据迁移工具**

### 混合使用策略：
```rust
use qmx_backend_lib::*;

fn hybrid_usage_example() -> anyhow::Result<()> {
    // 使用v2进行日常操作
    let manager = QmxManager::new(true)?;
    let uid = manager.create_student(
        StudentBuilder::new("新学员", 18)
    )?;
    
    // 使用v1进行批量操作
    let mut db = init::init()?;
    let batch_students = vec![/* 大量学生数据 */];
    db.student.insert_batch(batch_students);
    
    Ok(())
}
```

---

## 📖 详细文档链接

- **[📘 v1 API 完整文档](API_v1.md)** - 传统直接操作API
- **[📗 v2 API 完整文档](API_v2.md)** - 现代化统一API
- **[📋 v2.2.0 变更说明](API_CHANGES_v2.2.0.md)** - 重要变更和迁移指南

---

## 🔧 开发工具

### 依赖配置
```toml
[dependencies]
qmx_backend_lib = "2.4.0"
anyhow = "1.0"
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
```

### 导入建议
```rust
// 使用v1 API
use qmx_backend_lib::{*, student::*, cash::*};

// 使用v2 API  
use qmx_backend_lib::{QmxManager, StudentBuilder, CashBuilder};

// 混合使用
use qmx_backend_lib::*;
```



*文档索引版本：2.4.0*  
*最后更新：2025-09-15*