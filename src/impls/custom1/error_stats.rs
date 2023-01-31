#[derive(Debug, Default)]
pub struct ErrorStatistics {
    pub minus_avg: f64,
    pub plus_avg: f64,
    pub minus_max: f64,
    pub plus_max: f64,
    pub unstable_rate: f64,
}

impl ErrorStatistics {
    pub fn new(hit_errors: &[i32]) -> Self {
        let mut total_plus: f64 = 0.0;
        let mut total_minus: f64 = 0.0;
        let mut total_all: f64 = 0.0;
        let mut count_plus: usize = 0;
        let mut count_minus: usize = 0;
        let mut max: i32 = 0;
        let mut min: i32 = i32::MAX;

        for &hit_error in hit_errors {
            max = max.max(hit_error);
            min = min.min(hit_error);

            total_all += hit_error as f64;

            if hit_error >= 0 {
                total_plus += hit_error as f64;
                count_plus += 1;
            } else {
                total_minus += hit_error as f64;
                count_minus += 1;
            }
        }

        let avg = total_all / hit_errors.len() as f64;
        let mut variance = 0.0;

        for &curr in hit_errors {
            variance += (curr as f64 - avg) * (curr as f64 - avg);
        }

        variance /= hit_errors.len() as f64;

        let minus_avg = if count_minus == 0 {
            0.0
        } else {
            total_minus / count_minus as f64
        };

        let plus_avg = if count_plus == 0 {
            0.0
        } else {
            total_plus / count_plus as f64
        };

        Self {
            minus_avg,
            plus_avg,
            minus_max: min as f64,
            plus_max: max as f64,
            unstable_rate: variance.sqrt() * 10.0,
        }
    }
}
