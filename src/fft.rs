use rustfft::{FftPlanner, num_complex::Complex};

pub fn calculate_fft(data: &[f32]) -> Vec<f32> {
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(data.len());

    let mut fft_data = data.iter().map(|f| Complex{ re: *f, im: 0.0f32 }).collect::<Vec<_>>();
    fft.process(&mut fft_data);

    fft_data.iter().map(|x|x.norm()).collect()
}

pub fn find_maxima(data: &[f32], threshold: f32) -> (f32, usize) {
    let max_index = data.len() / 2 - 1;

    let mut maxima: usize = 0;
    let mut max: f32 = 0.0;

    for (i, &x) in data.iter().enumerate() {
        if i > max_index {
            break;
        }

        if x > threshold && x > max {
            maxima = i;
            max = x;
        }
    
    }

    (max, maxima)
}

pub fn recalculate_to_freq(bucket: usize, buffer_size: usize, sample_rate: f32) -> f32 {
    (bucket as f32) * (sample_rate / (buffer_size as f32))
}