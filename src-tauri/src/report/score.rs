#[derive(Debug, Clone, serde::Serialize)]
pub struct QualityScore {
    pub total: u32,
    pub capacity_score: u32,
    pub speed_score: u32,
    pub stability_score: u32,
    pub badblock_score: u32,
    pub grade: Grade,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum Grade {
    Excellent,
    Good,
    Fair,
    Poor,
}

pub fn calculate_score(
    claimed_capacity: u64,
    real_capacity: Option<u64>,
    seq_read_speed: Option<f64>,
    seq_write_speed: Option<f64>,
    stability: Option<f64>,
    speed_drops: Option<u32>,
    bad_block_count: Option<u64>,
    total_blocks: Option<u64>,
) -> QualityScore {
    // Capacity score (35 points)
    let capacity_score = match real_capacity {
        Some(real) => {
            let ratio = real as f64 / claimed_capacity.max(1) as f64;
            if ratio >= 0.95 {
                35
            } else if ratio < 0.50 {
                0
            } else {
                ((ratio - 0.50) / 0.45 * 35.0) as u32
            }
        }
        None => 0,
    };

    // Speed score (25 points)
    let read_score = match seq_read_speed {
        Some(speed) => ((speed / 100.0).min(1.0) * 12.0) as u32,
        None => 0,
    };
    let write_score = match seq_write_speed {
        Some(speed) => ((speed / 50.0).min(1.0) * 13.0) as u32,
        None => 0,
    };
    let speed_score = read_score + write_score;

    // Stability score (15 points)
    let stability_score = match (stability, speed_drops) {
        (Some(stab), Some(drops)) => {
            let stab_pts = (stab * 10.0) as u32;
            let drop_pts = if drops == 0 {
                5
            } else if drops <= 2 {
                3
            } else {
                0
            };
            (stab_pts + drop_pts).min(15)
        }
        (Some(stab), None) => ((stab * 15.0) as u32).min(15),
        _ => 0,
    };

    // Bad block score (25 points)
    let badblock_score = match (bad_block_count, total_blocks) {
        (Some(bad), Some(total)) if total > 0 => {
            let ratio = bad as f64 / total as f64;
            if ratio == 0.0 {
                25
            } else if ratio > 0.01 {
                0
            } else {
                ((1.0 - ratio / 0.01) * 25.0) as u32
            }
        }
        (Some(0), _) => 25,
        _ => 0,
    };

    let total = capacity_score + speed_score + stability_score + badblock_score;
    let grade = match total {
        90..=100 => Grade::Excellent,
        70..=89 => Grade::Good,
        50..=69 => Grade::Fair,
        _ => Grade::Poor,
    };

    QualityScore {
        total,
        capacity_score,
        speed_score,
        stability_score,
        badblock_score,
        grade,
    }
}
