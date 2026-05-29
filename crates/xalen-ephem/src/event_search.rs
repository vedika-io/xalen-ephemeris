/// Result of a numerical event search (crossing, ingress, or station).
pub struct EventSearchResult {
    /// Julian Date at which the event occurs.
    pub jd: f64,
    /// Function value at the event epoch.
    pub value: f64,
    /// Number of bisection iterations used to refine the result.
    pub iterations: u32,
}

/// Find all epochs where `f(jd)` crosses `target` within the given range.
pub fn find_crossing<F>(
    f: F,
    target: f64,
    jd_start: f64,
    jd_end: f64,
    step_days: f64,
) -> Vec<EventSearchResult>
where
    F: Fn(f64) -> f64,
{
    if step_days <= 0.0 || jd_start >= jd_end {
        return Vec::new();
    }
    let mut results = Vec::new();
    let mut jd = jd_start;
    let mut prev_val = f(jd) - target;

    while jd < jd_end {
        jd += step_days;
        let curr_val = f(jd) - target;

        // Sign change or exact hit = crossing detected
        if prev_val * curr_val < 0.0 {
            if let Some(result) = bisect_crossing(&f, target, jd - step_days, jd, 50) {
                results.push(result);
            }
        } else if curr_val == 0.0 {
            results.push(EventSearchResult {
                jd,
                value: f(jd),
                iterations: 0,
            });
        }
        prev_val = curr_val;
    }
    results
}

fn bisect_crossing<F>(
    f: &F,
    target: f64,
    mut lo: f64,
    mut hi: f64,
    max_iter: u32,
) -> Option<EventSearchResult>
where
    F: Fn(f64) -> f64,
{
    let mut iterations = 0;
    let f_lo = f(lo) - target;

    for _ in 0..max_iter {
        iterations += 1;
        let mid = (lo + hi) / 2.0;
        let f_mid = f(mid) - target;

        if f_mid.abs() < 1e-10 || (hi - lo) < 1e-10 {
            return Some(EventSearchResult {
                jd: mid,
                value: f(mid),
                iterations,
            });
        }

        if f_lo * f_mid < 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
    }

    Some(EventSearchResult {
        jd: (lo + hi) / 2.0,
        value: f((lo + hi) / 2.0),
        iterations,
    })
}

/// Find zodiac sign ingresses (30-degree boundary crossings).
pub fn find_sign_ingress<F>(
    longitude_fn: F,
    jd_start: f64,
    jd_end: f64,
    step_days: f64,
) -> Vec<EventSearchResult>
where
    F: Fn(f64) -> f64,
{
    if step_days <= 0.0 || jd_start >= jd_end {
        return Vec::new();
    }
    let mut results = Vec::new();
    let mut jd = jd_start;
    let mut prev_sign = (longitude_fn(jd) / 30.0) as u32;

    while jd < jd_end {
        jd += step_days;
        let curr_lon = longitude_fn(jd);
        let curr_sign = (curr_lon / 30.0) as u32;

        if curr_sign != prev_sign {
            let target_lon = (curr_sign * 30) as f64;
            // Refine: find exact moment longitude crosses sign boundary
            let refined = bisect_sign_boundary(&longitude_fn, jd - step_days, jd, target_lon, 50);
            if let Some(r) = refined {
                results.push(r);
            }
        }
        prev_sign = curr_sign;
    }
    results
}

fn bisect_sign_boundary<F>(
    f: &F,
    mut lo: f64,
    mut hi: f64,
    boundary: f64,
    max_iter: u32,
) -> Option<EventSearchResult>
where
    F: Fn(f64) -> f64,
{
    // Detect the 360/0 wrap case: if the longitude difference across the
    // interval exceeds 180 degrees, the function is discontinuous here and
    // we need to use wrap-aware comparison instead of sign-based bisection.
    let lo_val = f(lo);
    let hi_val = f(hi);
    let is_wrap = (hi_val - lo_val).abs() > 180.0;

    if is_wrap {
        // Wrap case: longitude crosses 360/0. We shift longitudes so the
        // discontinuity is far from our search region, then bisect on the
        // shifted value crossing the shifted boundary.
        let offset = 180.0;
        let shifted_boundary = (boundary + offset).rem_euclid(360.0);

        let shifted_lo = (f(lo) + offset).rem_euclid(360.0);

        for i in 0..max_iter {
            let mid = (lo + hi) / 2.0;
            let val = f(mid);
            let shifted_mid = (val + offset).rem_euclid(360.0);

            if (hi - lo) < 1e-8 {
                return Some(EventSearchResult {
                    jd: mid,
                    value: val,
                    iterations: i,
                });
            }

            // In the shifted domain, the function is continuous. Check which
            // side of the boundary the midpoint falls on relative to lo.
            let lo_side = shifted_lo < shifted_boundary;
            let mid_side = shifted_mid < shifted_boundary;

            if lo_side != mid_side {
                hi = mid;
            } else {
                lo = mid;
            }
        }
    } else {
        // Normal (non-wrap) case: sign-based bisection works fine.
        let _sign_lo_initial = (f(lo) / 30.0) as u32;

        for i in 0..max_iter {
            let mid = (lo + hi) / 2.0;
            let val = f(mid);
            let sign_mid = (val / 30.0) as u32;
            let sign_lo = (f(lo) / 30.0) as u32;

            if (hi - lo) < 1e-8 {
                return Some(EventSearchResult {
                    jd: mid,
                    value: val,
                    iterations: i,
                });
            }

            if sign_mid != sign_lo {
                hi = mid;
            } else {
                lo = mid;
            }
        }
    }

    Some(EventSearchResult {
        jd: (lo + hi) / 2.0,
        value: f((lo + hi) / 2.0),
        iterations: max_iter,
    })
}

/// Find planetary stations (zero-crossings of a speed function).
pub fn find_station<F>(
    speed_fn: F,
    jd_start: f64,
    jd_end: f64,
    step_days: f64,
) -> Vec<EventSearchResult>
where
    F: Fn(f64) -> f64,
{
    find_crossing(speed_fn, 0.0, jd_start, jd_end, step_days)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_linear_crossing() {
        // f(x) = x, target = 5.0 → crossing at x = 5. Use non-aligned step.
        let results = find_crossing(|x| x, 5.0, 0.0, 10.0, 0.7);
        assert_eq!(
            results.len(),
            1,
            "Expected 1 crossing, got {}",
            results.len()
        );
        assert!(
            (results[0].jd - 5.0).abs() < 1e-6,
            "Crossing should be at 5.0, got {}",
            results[0].jd
        );
    }

    #[test]
    fn find_sine_crossings() {
        // sin(x) crosses 0 at multiples of pi
        let results = find_crossing(|x| x.sin(), 0.0, 0.1, 10.0, 0.1);
        assert!(
            results.len() >= 2,
            "Should find at least 2 zero crossings of sin(x) in [0.1, 10]"
        );
        // First crossing near pi
        assert!(
            (results[0].jd - std::f64::consts::PI).abs() < 0.01,
            "First crossing should be near pi, got {}",
            results[0].jd
        );
    }

    #[test]
    fn find_sign_ingress_linear() {
        // Longitude increases linearly: 0.9856 deg/day (Sun-like)
        let results = find_sign_ingress(|jd| (jd * 0.9856).rem_euclid(360.0), 0.0, 400.0, 1.0);
        assert!(!results.is_empty(), "Should find sign ingresses");
        // First ingress: when 0.9856 * jd = 30 → jd ≈ 30.44
        assert!(
            (results[0].jd - 30.44).abs() < 1.0,
            "First ingress should be ~30.44, got {}",
            results[0].jd
        );
    }

    #[test]
    fn find_station_cosine() {
        // speed = cos(x), stations at pi/2, 3pi/2
        let results = find_station(|x| x.cos(), 0.1, 10.0, 0.1);
        assert!(!results.is_empty());
        assert!(
            (results[0].jd - std::f64::consts::FRAC_PI_2).abs() < 0.01,
            "First station should be near pi/2, got {}",
            results[0].jd
        );
    }

    #[test]
    fn find_sign_ingress_wrap_360_to_0() {
        // Longitude wraps from ~359 to ~0 (Pisces -> Aries boundary at 0/360).
        // 1 deg/day starting at 350, so wraps around day 10.
        let results = find_sign_ingress(|jd| (350.0 + jd).rem_euclid(360.0), 0.0, 30.0, 0.5);
        // Should detect the Pisces->Aries ingress at jd=10 (when lon crosses 0/360)
        let wrap_ingress = results.iter().find(|r| {
            let lon = (350.0 + r.jd).rem_euclid(360.0);
            lon < 1.0 || lon > 359.0 // near the 0/360 boundary
        });
        assert!(
            wrap_ingress.is_some(),
            "Should detect the 360/0 wrap ingress; found {:?}",
            results.iter().map(|r| (r.jd, r.value)).collect::<Vec<_>>()
        );
        let w = wrap_ingress.unwrap();
        assert!(
            (w.jd - 10.0).abs() < 0.5,
            "Wrap ingress should be near jd=10, got {}",
            w.jd
        );
    }

    #[test]
    fn crossing_exactly_on_step_boundary() {
        let results = find_crossing(|x| x, 5.0, 0.0, 10.0, 1.0);
        assert!(
            !results.is_empty(),
            "Should find crossing at step boundary jd=5.0"
        );
        let closest = results
            .iter()
            .min_by(|a, b| (a.jd - 5.0).abs().partial_cmp(&(b.jd - 5.0).abs()).unwrap())
            .unwrap();
        assert!(
            (closest.jd - 5.0).abs() < 1e-6,
            "Crossing should be at 5.0, got {}",
            closest.jd
        );
    }

    #[test]
    fn no_crossing_when_none_exists() {
        let results = find_crossing(|x| x * x + 1.0, 0.0, -10.0, 10.0, 0.5);
        assert!(results.is_empty(), "x^2+1 never crosses 0");
    }

    #[test]
    fn precision_of_bisection() {
        let results = find_crossing(|x| x, 100.0, 0.0, 200.0, 1.3);
        assert_eq!(
            results.len(),
            1,
            "Expected 1 crossing, got {}",
            results.len()
        );
        assert!(
            (results[0].jd - 100.0).abs() < 1e-8,
            "Should converge to high precision"
        );
    }
}
