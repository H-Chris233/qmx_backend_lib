use crate::cash::CashDatabase;
use crate::student::StudentDatabase;
use anyhow::Result;
use log::info;

/// 仪表板统计数据结构
///
/// 包含学生管理系统的核心统计指标，用于生成管理报告和仪表板展示。
///
/// # 字段说明
///
/// - `total_students`: 系统中的学生总数
/// - `total_revenue`: 总收入金额（单位：分）
/// - `total_expense`: 总支出金额（单位：分）
/// - `average_score`: 所有学生的平均成绩
/// - `max_score`: 系统中的最高成绩
/// - `active_courses`: 活跃课程类型数量
///
/// # 示例
///
/// ```rust
/// use qmx_backend_lib::*;
///
/// # fn main() -> anyhow::Result<()> {
/// let db = database::init()?;
/// let stats = get_dashboard_stats(&db.student, &db.cash)?;
///
/// println!("学生总数: {}", stats.total_students);
/// println!("总收入: {:.2} 元", stats.total_revenue as f64 / 100.0);
/// println!("平均成绩: {:.2}", stats.average_score);
/// # Ok(())
/// # }
/// ```
#[derive(serde::Serialize, Debug)]
pub struct DashboardStats {
    pub total_students: usize,
    pub total_revenue: i64,
    pub total_expense: i64,
    pub average_score: f64,
    pub max_score: f64,
    pub active_courses: usize,
}

/// 计算仪表板统计数据
///
/// 从学生数据库和现金数据库中提取并计算关键统计指标。
///
/// # 参数
///
/// - `student_db`: 学生数据库引用
/// - `cash_db`: 现金数据库引用
///
/// # 返回值
///
/// 返回包含所有统计指标的 `DashboardStats` 结构体。
///
/// # 错误
///
/// 当数据库访问出现问题时返回错误。
///
/// # 示例
///
/// ```rust
/// use qmx_backend_lib::*;
///
/// # fn main() -> anyhow::Result<()> {
/// let mut db = database::init()?;
///
/// // 添加一些测试数据
/// let mut student = student::Student::new();
/// student.set_name("测试学生".to_string()).add_ring(9.5);
/// db.student.insert(student);
///
/// let mut cash = cash::Cash::new(None);
/// cash.set_cash(1000);
/// db.cash.insert(cash);
///
/// // 计算统计数据
/// let stats = get_dashboard_stats(&db.student, &db.cash)?;
/// assert_eq!(stats.total_students, 1);
/// assert_eq!(stats.total_revenue, 1000);
/// # Ok(())
/// # }
/// ```
///
/// # 性能说明
///
/// - 时间复杂度: O(n + m)，其中 n 是学生数量，m 是现金记录数量
/// - 空间复杂度: O(k)，其中 k 是不同课程类型的数量
pub fn get_dashboard_stats(
    student_db: &StudentDatabase,
    cash_db: &CashDatabase,
) -> Result<DashboardStats> {
    info!("开始计算仪表盘统计数据");
    let mut total_revenue = 0;
    let mut total_expense = 0;
    let mut max_score = 0.0;
    let mut total_score_sum = 0.0;
    let mut total_score_count = 0;

    let total_students = student_db.len();
    let mut class_types = std::collections::HashSet::new();

    for (_, student) in student_db.iter() {
        class_types.insert(format!("{:?}", student.class()));
        for &score in student.rings() {
            total_score_sum += score;
            total_score_count += 1;
            if score > max_score {
                max_score = score;
            }
        }
    }

    let active_courses = class_types
        .iter()
        .filter(|class| class.as_str() != "Others")
        .count();

    for (_, transaction) in cash_db.iter() {
        if transaction.cash >= 0 {
            total_revenue += transaction.cash;
        } else {
            total_expense += transaction.cash.abs();
        }
    }

    let average_score = if total_score_count == 0 {
        0.0
    } else {
        total_score_sum / total_score_count as f64
    };

    let stats = DashboardStats {
        total_students,
        total_revenue,
        total_expense,
        average_score,
        max_score,
        active_courses,
    };
    info!(
        "仪表盘统计计算完成: students={}, revenue={}, expense={}, avg={}, max={}, active_courses={}",
        total_students, total_revenue, total_expense, average_score, max_score, active_courses
    );
    Ok(stats)
}
