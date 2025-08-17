use anyhow::Result;
use crate::student::{StudentDatabase, Student};
use crate::cash::CashDatabase;

/// 仪表盘统计数据结构
#[derive(serde::Serialize, Debug)]
pub struct DashboardStats {
    pub total_students: usize,
    pub total_revenue: i32,
    pub total_expense: i32,
    pub average_score: f64,
    pub max_score: f64,
    pub active_courses: usize,
}

/// 获取仪表盘统计数据
pub fn get_dashboard_stats(
    student_db: &StudentDatabase, 
    cash_db: &CashDatabase
) -> Result<DashboardStats> {
    let mut total_revenue = 0;
    let mut total_expense = 0;
    let mut all_scores = Vec::new();
    let mut max_score = 0.0;
    
    // 统计学生数量和课程类型
    let total_students = student_db.len();
    let mut class_types = std::collections::HashSet::new();
    
    // 遍历所有学生数据
    for (_, student) in student_db.iter() {
        // 统计课程类型
        class_types.insert(format!("{:?}", student.class()));
        
        // 收集所有成绩
        for &score in student.rings() {
            all_scores.push(score);
            if score > max_score {
                max_score = score;
            }
        }
    }
    
    // 统计活跃课程类型数量（排除 Others）
    let active_courses = class_types.iter().filter(|class| class.as_str() != "Others").count();
    
    // 统计财务数据
    for (_, transaction) in cash_db.iter() {
        if transaction.cash >= 0 {
            total_revenue += transaction.cash;
        } else {
            total_expense += transaction.cash.abs();
        }
    }
    
    // 计算平均成绩
    let average_score = if all_scores.is_empty() {
        0.0
    } else {
        let sum: f64 = all_scores.iter().sum();
        sum / all_scores.len() as f64
    };
    
    Ok(DashboardStats {
        total_students,
        total_revenue,
        total_expense,
        average_score,
        max_score,
        active_courses,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::student::{Person, Class};
    use crate::cash::Cash;
    
    #[test]
    fn test_get_dashboard_stats_empty() {
        let student_db = StudentDatabase::new();
        let cash_db = CashDatabase::new();
        
        let stats = get_dashboard_stats(&student_db, &cash_db).unwrap();
        
        assert_eq!(stats.total_students, 0);
        assert_eq!(stats.total_revenue, 0);
        assert_eq!(stats.total_expense, 0);
        assert_eq!(stats.average_score, 0.0);
        assert_eq!(stats.max_score, 0.0);
        assert_eq!(stats.active_courses, 0);
    }
    
    #[test]
    fn test_get_dashboard_stats_with_data() {
        let mut student_db = StudentDatabase::new();
        let mut cash_db = CashDatabase::new();
        
        // 添加测试学生
        let mut student1 = Person::new();
        student1.set_name("张三".to_string()).set_class(Class::TenTry);
        student1.add_ring(8.5).add_ring(9.0);
        
        let mut student2 = Person::new();
        student2.set_name("李四".to_string()).set_class(Class::Month);
        student2.add_ring(7.5).add_ring(8.0).add_ring(9.5);
        
        student_db.insert(student1);
        student_db.insert(student2);
        
        // 添加测试财务记录
        let mut cash1 = Cash::new(Some(1));
        cash1.add(100);
        
        let mut cash2 = Cash::new(None);
        cash2.add(-50);
        
        cash_db.insert(cash1);
        cash_db.insert(cash2);
        
        let stats = get_dashboard_stats(&student_db, &cash_db).unwrap();
        
        assert_eq!(stats.total_students, 2);
        assert_eq!(stats.total_revenue, 100);
        assert_eq!(stats.total_expense, 50);
        assert_eq!(stats.average_score, 8.5); // (8.5+9.0+7.5+8.0+9.5)/5
        assert_eq!(stats.max_score, 9.5);
        assert_eq!(stats.active_courses, 2); // TenTry 和 Month
    }
}