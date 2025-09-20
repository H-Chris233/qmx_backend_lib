# QMX Backend Library

[![Rust](https://img.shields.io/badge/rust-1.87+-orange.svg)](https://www.rust-lang.org)
[![Version](https://img.shields.io/badge/version-2.5.0-blue.svg)](https://github.com/H-Chris233/qmx_backend_lib)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

一个用 Rust 编写的高性能学生管理和财务管理库，专为教育培训机构设计。

## 特性

- 🎓 **学生管理** - 完整的学生档案管理，包含课程、成绩、会员信息
- 💰 **财务管理** - 现金流跟踪、收支记录、财务统计
- 📅 **分期付款** - 灵活的分期付款计划管理，支持多种付款频率
- 👥 **会员系统** - 会员期限管理、状态检查、到期提醒
- 📊 **统计分析** - 仪表板统计、收入分析、学员表现统计

## 快速开始

### 安装

在你的 `Cargo.toml` 中添加：

```toml
[dependencies]
qmx_backend_lib = "2.5.0"
```

### 基本使用

```rust
use qmx_backend_lib::*;
use chrono::{Utc, Duration};

fn main() -> anyhow::Result<()> {
    // 初始化系统
    let mut db = init::init()?;

    // 创建学生
    let mut student = Student::new();
    student
        .set_name("张三".to_string())
        .set_age(18)
        .set_class(Class::TenTry)
        .set_subject(Subject::Shooting)
        .add_ring(9.5);

    // 设置会员期限
    student.set_membership_dates(
        Some(Utc::now()),
        Some(Utc::now() + Duration::days(365))
    );

    let student_id = student.uid();
    db.student.insert(student);

    // 添加收入记录
    let mut cash = Cash::new(Some(student_id));
    cash.set_cash(1000);
    cash.set_note(Some("学费收入".to_string()));
    db.cash.insert(cash);

    // 生成统计报告
    let stats = get_dashboard_stats(&db.student, &db.cash)?;
    println!("学生总数: {}", stats.total_students);
    println!("总收入: {}", stats.total_revenue);

    // 保存数据
    save::save(db)?;
    
    Ok(())
}
```

## 测试

```bash
# 运行所有测试
cargo test -- --test-threads=1

# 运行特定模块测试
cargo test student_tests
cargo test cash_tests

# 显示测试输出
cargo test -- --nocapture --test-threads=1
```

## 构建

```bash
# 开发构建
cargo build

# 发布构建
cargo build --release

# 代码格式化
cargo fmt

# 代码检查
cargo clippy

# 生成文档
cargo doc --open
```

## 许可证

本项目采用 MIT 许可证。详见 [LICENSE](LICENSE) 文件。