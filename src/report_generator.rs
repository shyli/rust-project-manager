use crate::models::{TimeRecord, WeeklyReport};
use crate::time_calculator::TimeCalculator;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

pub struct ReportGenerator;

impl ReportGenerator {
    /// 生成每周报表
    pub fn generate_weekly_report(
        time_records: &[&TimeRecord],
        project_names: &HashMap<Uuid, String>,
        report_date: DateTime<Utc>,
    ) -> WeeklyReport {
        let week_start = TimeCalculator::get_week_start(report_date);
        let week_end = TimeCalculator::get_week_end(report_date);

        let total_project_time =
            TimeCalculator::calculate_project_time(time_records, week_start, week_end);
        let total_non_project_time =
            TimeCalculator::calculate_non_project_time(time_records, week_start, week_end);

        let project_breakdown = TimeCalculator::generate_project_breakdown(
            time_records,
            project_names,
            week_start,
            week_end,
        );

        let mut report = WeeklyReport::new(week_start, week_end);
        report.total_project_time_minutes = total_project_time;
        report.total_non_project_time_minutes = total_non_project_time;
        report.project_breakdown = project_breakdown;

        report
    }

    /// 生成报表文本摘要
    pub fn generate_report_summary(report: &WeeklyReport) -> String {
        let mut summary = String::new();

        summary.push_str(&format!("=== 每周报表 ===\n"));
        summary.push_str(&format!(
            "时间范围: {} 至 {}\n\n",
            report.week_start.format("%Y-%m-%d"),
            report.week_end.format("%Y-%m-%d")
        ));

        summary.push_str(&format!(
            "项目内时间: {}\n",
            TimeCalculator::format_duration(report.total_project_time_minutes)
        ));
        summary.push_str(&format!(
            "项目外时间: {}\n",
            TimeCalculator::format_duration(report.total_non_project_time_minutes)
        ));

        let total_time = report.total_project_time_minutes + report.total_non_project_time_minutes;
        let efficiency = if total_time > 0 {
            (report.total_project_time_minutes as f64 / total_time as f64) * 100.0
        } else {
            0.0
        };

        summary.push_str(&format!("工作效率: {:.2}%\n\n", efficiency));

        if !report.project_breakdown.is_empty() {
            summary.push_str("项目时间分解:\n");
            for breakdown in &report.project_breakdown {
                summary.push_str(&format!(
                    "  - {}: {} ({}个事件)\n",
                    breakdown.project_name,
                    TimeCalculator::format_duration(breakdown.total_time_minutes),
                    breakdown.event_count
                ));
            }
        } else {
            summary.push_str("本周没有项目相关事件\n");
        }

        summary.push_str(&format!(
            "\n报表生成时间: {}\n",
            report.generated_at.format("%Y-%m-%d %H:%M:%S")
        ));

        summary
    }

    /// 生成详细报表（包含每日统计）
    pub fn generate_detailed_weekly_report(
        time_records: &[&TimeRecord],
        project_names: &HashMap<Uuid, String>,
        report_date: DateTime<Utc>,
    ) -> String {
        let mut detailed_report = String::new();

        let week_start = TimeCalculator::get_week_start(report_date);
        let week_end = TimeCalculator::get_week_end(report_date);

        detailed_report.push_str(&format!("=== 详细每周报表 ===\n"));
        detailed_report.push_str(&format!(
            "时间范围: {} 至 {}\n\n",
            week_start.format("%Y-%m-%d"),
            week_end.format("%Y-%m-%d")
        ));

        // 每日统计
        detailed_report.push_str("每日统计:\n");
        let mut current_day = week_start;

        while current_day <= week_end {
            let daily_records: Vec<&TimeRecord> = time_records
                .iter()
                .filter(|record| {
                    let record_date = record.start_time.date_naive();
                    let current_date = current_day.date_naive();
                    record_date == current_date
                })
                .copied()
                .collect();

            let (project_time, non_project_time) =
                TimeCalculator::calculate_daily_stats(&daily_records, current_day);

            detailed_report.push_str(&format!(
                "  {}: 项目内={}, 项目外={}\n",
                current_day.format("%Y-%m-%d (%a)"),
                TimeCalculator::format_duration(project_time),
                TimeCalculator::format_duration(non_project_time)
            ));

            current_day = current_day + chrono::Duration::days(1);
        }

        // 总体统计
        let total_project_time =
            TimeCalculator::calculate_project_time(time_records, week_start, week_end);
        let total_non_project_time =
            TimeCalculator::calculate_non_project_time(time_records, week_start, week_end);

        detailed_report.push_str("\n总体统计:\n");
        detailed_report.push_str(&format!(
            "  项目内总时间: {}\n",
            TimeCalculator::format_duration(total_project_time)
        ));
        detailed_report.push_str(&format!(
            "  项目外总时间: {}\n",
            TimeCalculator::format_duration(total_non_project_time)
        ));

        let total_time = total_project_time + total_non_project_time;
        let efficiency = if total_time > 0 {
            (total_project_time as f64 / total_time as f64) * 100.0
        } else {
            0.0
        };
        detailed_report.push_str(&format!("  工作效率: {:.2}%\n", efficiency));

        // 项目分解
        let project_breakdown = TimeCalculator::generate_project_breakdown(
            time_records,
            project_names,
            week_start,
            week_end,
        );

        if !project_breakdown.is_empty() {
            detailed_report.push_str("\n项目时间分解:\n");
            for breakdown in project_breakdown {
                detailed_report.push_str(&format!(
                    "  - {}: {} ({}个事件)\n",
                    breakdown.project_name,
                    TimeCalculator::format_duration(breakdown.total_time_minutes),
                    breakdown.event_count
                ));
            }
        }

        // 项目排名
        let project_ranking =
            TimeCalculator::get_project_ranking(time_records, project_names, week_start, week_end);

        if !project_ranking.is_empty() {
            detailed_report.push_str("\n项目时间排名:\n");
            for (index, (project_name, time)) in project_ranking.iter().enumerate() {
                detailed_report.push_str(&format!(
                    "  {}. {}: {}\n",
                    index + 1,
                    project_name,
                    TimeCalculator::format_duration(*time)
                ));
            }
        }

        detailed_report.push_str(&format!(
            "\n报表生成时间: {}\n",
            Utc::now().format("%Y-%m-%d %H:%M:%S")
        ));

        detailed_report
    }

    /// 生成月度报表摘要
    pub fn generate_monthly_summary(
        time_records: &[&TimeRecord],
        project_names: &HashMap<Uuid, String>,
        year: i32,
        month: u32,
    ) -> String {
        let mut summary = String::new();

        let (project_time, non_project_time) =
            TimeCalculator::calculate_monthly_stats(time_records, year, month);

        summary.push_str(&format!("=== 月度报表 ===\n"));
        summary.push_str(&format!("时间范围: {}年{}月\n\n", year, month));

        summary.push_str(&format!(
            "项目内时间: {}\n",
            TimeCalculator::format_duration(project_time)
        ));
        summary.push_str(&format!(
            "项目外时间: {}\n",
            TimeCalculator::format_duration(non_project_time)
        ));

        let total_time = project_time + non_project_time;
        let efficiency = if total_time > 0 {
            (project_time as f64 / total_time as f64) * 100.0
        } else {
            0.0
        };

        summary.push_str(&format!("工作效率: {:.2}%\n", efficiency));

        // 计算月度开始和结束时间
        let month_start = chrono::NaiveDate::from_ymd_opt(year, month, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();

        let next_month = if month == 12 {
            (year + 1, 1)
        } else {
            (year, month + 1)
        };

        let month_end = chrono::NaiveDate::from_ymd_opt(next_month.0, next_month.1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            - chrono::Duration::seconds(1);

        let project_breakdown = TimeCalculator::generate_project_breakdown(
            time_records,
            project_names,
            month_start,
            month_end,
        );

        if !project_breakdown.is_empty() {
            summary.push_str("\n项目时间分解:\n");
            for breakdown in project_breakdown {
                summary.push_str(&format!(
                    "  - {}: {} ({}个事件)\n",
                    breakdown.project_name,
                    TimeCalculator::format_duration(breakdown.total_time_minutes),
                    breakdown.event_count
                ));
            }
        }

        summary
    }

    /// 导出报表为JSON格式
    pub fn export_report_to_json(report: &WeeklyReport) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(report)
    }

    /// 从JSON导入报表
    pub fn import_report_from_json(json_str: &str) -> Result<WeeklyReport, serde_json::Error> {
        serde_json::from_str(json_str)
    }

    /// 生成效率分析报告
    pub fn generate_efficiency_analysis(
        time_records: &[&TimeRecord],
        project_names: &HashMap<Uuid, String>,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> String {
        let mut analysis = String::new();

        analysis.push_str(&format!("=== 效率分析报告 ===\n"));
        analysis.push_str(&format!(
            "分析期间: {} 至 {}\n\n",
            start_date.format("%Y-%m-%d"),
            end_date.format("%Y-%m-%d")
        ));

        let project_time =
            TimeCalculator::calculate_project_time(time_records, start_date, end_date);
        let non_project_time =
            TimeCalculator::calculate_non_project_time(time_records, start_date, end_date);
        let total_time = project_time + non_project_time;

        analysis.push_str("时间分配:\n");
        analysis.push_str(&format!(
            "  项目内时间: {} ({:.1}%)\n",
            TimeCalculator::format_duration(project_time),
            if total_time > 0 {
                (project_time as f64 / total_time as f64) * 100.0
            } else {
                0.0
            }
        ));
        analysis.push_str(&format!(
            "  项目外时间: {} ({:.1}%)\n",
            TimeCalculator::format_duration(non_project_time),
            if total_time > 0 {
                (non_project_time as f64 / total_time as f64) * 100.0
            } else {
                0.0
            }
        ));

        // 项目效率分析
        let project_breakdown = TimeCalculator::generate_project_breakdown(
            time_records,
            project_names,
            start_date,
            end_date,
        );

        if !project_breakdown.is_empty() {
            analysis.push_str("\n项目效率分析:\n");
            for breakdown in project_breakdown {
                let avg_event_duration = if breakdown.event_count > 0 {
                    breakdown.total_time_minutes / breakdown.event_count as i64
                } else {
                    0
                };
                analysis.push_str(&format!(
                    "  - {}: 总时间={}, 平均事件时长={}\n",
                    breakdown.project_name,
                    TimeCalculator::format_duration(breakdown.total_time_minutes),
                    TimeCalculator::format_duration(avg_event_duration)
                ));
            }
        }

        // 建议
        analysis.push_str("\n改进建议:\n");
        let efficiency = if total_time > 0 {
            (project_time as f64 / total_time as f64) * 100.0
        } else {
            0.0
        };

        if efficiency < 50.0 {
            analysis.push_str("  - 建议减少项目外活动，增加项目内工作时间\n");
        } else if efficiency > 90.0 {
            analysis.push_str("  - 工作效率很高，注意保持工作生活平衡\n");
        } else {
            analysis.push_str("  - 工作效率良好，继续保持\n");
        }

        if non_project_time > project_time {
            analysis.push_str("  - 项目外时间过多，建议优化时间分配\n");
        }

        analysis
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn create_test_time_record(
        project_id: Option<Uuid>,
        start_time: DateTime<Utc>,
        duration_minutes: i64,
    ) -> TimeRecord {
        let end_time = start_time + Duration::minutes(duration_minutes);
        TimeRecord::new(Uuid::new_v4(), project_id, start_time, end_time)
    }

    #[test]
    fn test_generate_weekly_report() {
        let project_id = Uuid::new_v4();
        let base_time = Utc::now();

        let record1 = create_test_time_record(Some(project_id), base_time, 120);
        let record2 = create_test_time_record(Some(project_id), base_time + Duration::hours(3), 90);
        let record3 = create_test_time_record(None, base_time + Duration::hours(6), 60);
        let records = vec![&record1, &record2, &record3];

        let mut project_names = HashMap::new();
        project_names.insert(project_id, "测试项目".to_string());

        let report = ReportGenerator::generate_weekly_report(&records, &project_names, base_time);

        assert_eq!(report.total_project_time_minutes, 210); // 120 + 90
        assert_eq!(report.total_non_project_time_minutes, 60);
        assert_eq!(report.project_breakdown.len(), 1);
        assert_eq!(report.project_breakdown[0].project_name, "测试项目");
        assert_eq!(report.project_breakdown[0].total_time_minutes, 210);
        assert_eq!(report.project_breakdown[0].event_count, 2);
    }

    #[test]
    fn test_generate_report_summary() {
        let project_id = Uuid::new_v4();
        let base_time = Utc::now();

        let record = create_test_time_record(Some(project_id), base_time, 120);
        let records = vec![&record];

        let mut project_names = HashMap::new();
        project_names.insert(project_id, "测试项目".to_string());

        let report = ReportGenerator::generate_weekly_report(&records, &project_names, base_time);
        let summary = ReportGenerator::generate_report_summary(&report);

        assert!(summary.contains("每周报表"));
        assert!(summary.contains("项目内时间: 2小时"));
        assert!(summary.contains("项目外时间: 0分钟"));
        assert!(summary.contains("工作效率: 100.00%"));
        assert!(summary.contains("测试项目"));
    }

    #[test]
    fn test_export_import_json() {
        let project_id = Uuid::new_v4();
        let base_time = Utc::now();

        let record = create_test_time_record(Some(project_id), base_time, 120);
        let records = vec![&record];

        let mut project_names = HashMap::new();
        project_names.insert(project_id, "测试项目".to_string());

        let report = ReportGenerator::generate_weekly_report(&records, &project_names, base_time);

        // 导出为JSON
        let json_str = ReportGenerator::export_report_to_json(&report).unwrap();

        // 从JSON导入
        let imported_report = ReportGenerator::import_report_from_json(&json_str).unwrap();

        assert_eq!(report.id, imported_report.id);
        assert_eq!(
            report.total_project_time_minutes,
            imported_report.total_project_time_minutes
        );
        assert_eq!(
            report.total_non_project_time_minutes,
            imported_report.total_non_project_time_minutes
        );
        assert_eq!(
            report.project_breakdown.len(),
            imported_report.project_breakdown.len()
        );
    }
}
