#[derive(Debug, Default)]
pub struct ErrorStatistics {
    pub minus_avg: f64,
    pub plus_avg: f64,
    pub minus_max: f64,
    pub plus_max: f64,
    pub unstable_rate: f64,
}

impl ErrorStatistics {
    pub fn new(list: &[i32]) -> Self {
        let mut total = 0.0;
        let mut total_ = 0.0;
        let mut total_all = 0.0;
        let mut count = 0;
        let mut count_ = 0;
        let mut max = 0;
        let mut min = i32::MAX;

        for &curr in list {
            if curr > max {
                max = curr;
            }

            if curr < min {
                min = curr;
            }

            total_all += curr as f64;

            if curr >= 0 {
                total += curr as f64;
                count += 1;
            } else {
                total_ += curr as f64;
                count_ += 1;
            }
        }

        let avg = total_all / list.len() as f64;
        let mut variance = 0.0;

        for curr in list.iter().map(|&n| n as f64) {
            variance += (curr - avg) * (curr - avg);
        }

        variance /= list.len() as f64;

        Self {
            minus_avg: if count_ == 0 {
                0.0
            } else {
                total_ / count_ as f64
            },
            plus_avg: if count == 0 {
                0.0
            } else {
                total / count as f64
            },
            minus_max: min as f64,
            plus_max: max as f64,
            unstable_rate: variance.sqrt() * 10.0,
        }
    }
}
