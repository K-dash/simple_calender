use chrono::{NaiveDate, NaiveDateTime};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, BufWriter},
};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Schedule {
    id: u64,
    subject: String,
    start: NaiveDateTime,
    end: NaiveDateTime,
}

impl Schedule {
    fn intersects(&self, other: &Schedule) -> bool {
        self.start < other.end && other.start < self.end
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Calendar {
    schedules: Vec<Schedule>,
}

const SCHEDULE_FILE: &str = "schedules.json";

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 予定の一覧表示
    List,
    /// 予定の追加
    Add {
        /// タイトル
        subject: String,
        /// 開始日時
        start: NaiveDateTime,
        /// 終了日時
        end: NaiveDateTime,
    },
}

fn main() {
    let options = Cli::parse();
    match options.command {
        Commands::List => show_list(),
        Commands::Add {
            subject,
            start,
            end,
        } => add_schedule(subject, start, end),
    }
}

fn show_list() {
    let file: Calendar = {
        let file = File::open(SCHEDULE_FILE).unwrap();
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).unwrap()
    };
    // 予定の表示
    println!("ID\tStart\tEnd\tSubject");
    for schedule in file.schedules {
        println!(
            "{}\t{}\t{}\t{}",
            schedule.id, schedule.start, schedule.end, schedule.subject
        );
    }
}

fn add_schedule(subject: String, start: NaiveDateTime, end: NaiveDateTime) {
    let mut calendar: Calendar = {
        let file = File::open(SCHEDULE_FILE).unwrap();
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).unwrap()
    };

    // 予定の作成
    let id = calendar.schedules.len() as u64;
    let new_schedule = Schedule {
        id,
        subject,
        start,
        end,
    };

    // 予定の重複判定
    for schedule in &calendar.schedules {
        if schedule.intersects(&new_schedule) {
            println!("エラー：予定が重複しています");
            return;
        }
    }

    // 予定の追加
    calendar.schedules.push(new_schedule);

    // 予定の保存
    {
        let file = File::create(SCHEDULE_FILE).unwrap();
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &calendar).unwrap();
    }
    println!("予定を追加しました");
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn native_date_time(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    ) -> NaiveDateTime {
        chrono::NaiveDate::from_ymd_opt(year, month, day)
            .unwrap()
            .and_hms_opt(hour, minute, second)
            .unwrap()
    }

    #[rstest]
    #[case(18, 15, 18, 45, false)]
    #[case(18, 15, 19, 45, true)]
    #[case(18, 15, 20, 45, true)]
    #[case(19, 15, 19, 45, true)]
    #[case(19, 15, 20, 45, true)]
    #[case(20, 15, 20, 45, false)]
    fn test_schedule_intersects(
        #[case] h0: u32,
        #[case] m0: u32,
        #[case] h1: u32,
        #[case] m1: u32,
        #[case] should_intersect: bool,
    ) {
        let schedule = Schedule {
            id: 1,
            subject: "既存予定".to_string(),
            start: native_date_time(2024, 1, 1, h0, m0, 0),
            end: native_date_time(2024, 1, 1, h1, m1, 0),
        };
        let new_schedule = Schedule {
            id: 999,
            subject: "新規予定".to_string(),
            start: native_date_time(2024, 1, 1, 19, 0, 0),
            end: native_date_time(2024, 1, 1, 20, 0, 0),
        };
        assert_eq!(schedule.intersects(&new_schedule), should_intersect);
    }
}
