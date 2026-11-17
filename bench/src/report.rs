// Timestamped result files, and a summary that takes the median of every column across them so
// noise from single runs averages out.
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn median(v: &mut [f64]) -> Option<f64> {
    if v.is_empty() {
        return None;
    }
    v.sort_by(f64::total_cmp);
    let m = v.len() / 2;
    Some(if v.len() % 2 == 1 { v[m] } else { (v[m - 1] + v[m]) / 2.0 })
}

/// (year, month, day) of a day count since 1970-01-01 (Howard Hinnant's civil_from_days).
fn civil(days: i64) -> (i64, i64, i64) {
    let z = days + 719468;
    let (era, doe) = (z.div_euclid(146097), z.rem_euclid(146097));
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let (d, m) = (doy - (153 * mp + 2) / 5 + 1, if mp < 10 { mp + 3 } else { mp - 9 });
    (yoe + era * 400 + (m <= 2) as i64, m, d)
}

/// UTC time as YYYYMMDD-HHMMSS, for file names that sort by time.
pub fn stamp() -> String {
    let s = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
    let ((y, m, d), t) = (civil(s / 86400), s % 86400);
    format!("{y:04}{m:02}{d:02}-{:02}{:02}{:02}", t / 3600, t % 3600 / 60, t % 60)
}

/// The number a cell starts with: "4/4" -> 4, "12.5" -> 12.5, "skipped: ..." -> None.
fn number(cell: &str) -> Option<f64> {
    let end = cell.find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-')).unwrap_or(cell.len());
    cell[..end].parse().ok()
}

fn cells(line: &str) -> Vec<String> {
    let inner = line.trim().trim_start_matches('|').trim_end_matches('|');
    inner.split('|').map(|c| c.trim().to_string()).collect()
}

/// Median of every column, per variant, over every `<prefix><stamp>.md` file in `dir`.
pub fn summary(dir: &Path, prefix: &str) -> String {
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|f| f.starts_with(prefix) && f.ends_with(".md") && !f.contains("summary"))
        .collect();
    files.sort();
    let mut header = vec![];
    let mut variants: Vec<(String, Vec<Vec<String>>)> = vec![];
    for f in &files {
        let text = std::fs::read_to_string(dir.join(f)).unwrap_or_default();
        for line in text.lines().filter(|l| l.starts_with('|')) {
            let c = cells(line);
            if c[0] == "variant" {
                header = c; // the latest file's columns win
            } else if !c[0].starts_with("---") {
                match variants.iter_mut().find(|v| v.0 == c[0]) {
                    Some(v) => v.1.push(c[1..].to_vec()),
                    None => variants.push((c[0].clone(), vec![c[1..].to_vec()])),
                }
            }
        }
    }
    let mut out = format!(
        "# Summary of {} runs ({prefix}*)\n\nMedian of each column across runs. `runs` counts the runs that measured the variant.\n\n",
        files.len()
    );
    if header.is_empty() {
        return out + "No result files found.\n";
    }
    out += &format!("| {} | runs | {} |\n|{}\n", header[0], header[1..].join(" | "), "---|".repeat(header.len() + 1));
    for (name, runs) in &variants {
        let measured = runs.iter().filter(|r| r.first().and_then(|c| number(c)).is_some()).count();
        let cols: Vec<String> = (0..header.len() - 1)
            .map(|i| {
                let mut v: Vec<f64> = runs.iter().filter_map(|r| r.get(i).and_then(|c| number(c))).collect();
                let text = runs.iter().rev().find_map(|r| r.get(i).filter(|c| !c.is_empty()).cloned());
                median(&mut v).map_or(text.unwrap_or_default(), |m| format!("{m:.1}"))
            })
            .collect();
        out += &format!("| {name} | {measured} | {} |\n", cols.join(" | "));
    }
    out
}

#[test]
fn civil_dates() {
    assert_eq!(civil(0), (1970, 1, 1));
    assert_eq!(civil(11016), (2000, 2, 29));
    assert_eq!(civil(20774), (2026, 11, 17));
}

#[test]
fn medians_and_numbers() {
    assert_eq!(median(&mut [3.0, 1.0, 2.0]), Some(2.0));
    assert_eq!(median(&mut [4.0, 1.0]), Some(2.5));
    assert_eq!(number("4/4"), Some(4.0));
    assert_eq!(number("skipped: linux only"), None);
}
