use crate::cash::CashDatabase;
use crate::student::StudentDatabase;
use anyhow::Result;
use log::info;

#[derive(serde::Serialize, Debug)]
pub struct DashboardStats {
    pub total_students: usize,
    pub total_revenue: i64,
    pub total_expense: i64,
    pub average_score: f64,
    pub max_score: f64,
    pub active_courses: usize,
}

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
