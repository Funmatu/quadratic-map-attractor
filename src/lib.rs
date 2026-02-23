use wasm_bindgen::prelude::*;
use rand::Rng;

#[cfg(not(target_arch = "wasm32"))]
use pyo3::prelude::*;

#[wasm_bindgen]
pub struct AttractorConfig {
    num_particles: usize,
    states: Vec<f32>,
    constants: Vec<f32>,
}

#[wasm_bindgen]
impl AttractorConfig {
    #[wasm_bindgen(constructor)]
    pub fn new(num_particles: usize, base_scale: f32) -> Self {
        let mut rng = rand::thread_rng();
        let mut states = Vec::with_capacity(num_particles * 8);
        let mut constants = Vec::with_capacity(num_particles * 8);

        for _ in 0..num_particles {
            for _ in 0..6 { states.push(rng.gen_range(-0.01..0.01)); }
            states.push(0.0); states.push(0.0);

            let cx = rng.gen_range(-base_scale..base_scale);
            let cy = rng.gen_range(-base_scale..base_scale);
            
            constants.push(cx); constants.push(cy);
            constants.push(cx * 1.05); constants.push(cy * 0.95);
            constants.push(cx * 0.95); constants.push(cy * 1.05);
            constants.push(0.0); constants.push(0.0);
        }
        Self { num_particles, states, constants }
    }

    pub fn states_ptr(&self) -> *const f32 { self.states.as_ptr() }
    pub fn constants_ptr(&self) -> *const f32 { self.constants.as_ptr() }
    pub fn num_particles(&self) -> usize { self.num_particles }
}

#[wasm_bindgen]
pub fn get_memory() -> JsValue {
    wasm_bindgen::memory()
}

// ---------------------------------------------
// PyO3 Integration for Offline Parameter Search
// ---------------------------------------------
#[cfg(not(target_arch = "wasm32"))]
fn complex_mul(a: (f32, f32), b: (f32, f32)) -> (f32, f32) {
    (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0)
}

#[cfg(not(target_arch = "wasm32"))]
fn complex_sqr(a: (f32, f32)) -> (f32, f32) {
    (a.0 * a.0 - a.1 * a.1, 2.0 * a.0 * a.1)
}

#[cfg(not(target_arch = "wasm32"))]
fn complex_mul_f64(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0)
}

#[cfg(not(target_arch = "wasm32"))]
fn complex_sqr_f64(a: (f64, f64)) -> (f64, f64) {
    (a.0 * a.0 - a.1 * a.1, 2.0 * a.0 * a.1)
}

#[cfg(not(target_arch = "wasm32"))]
#[pyfunction]
fn evaluate_chaos_edge(k: f32, escape_radius: f32, steps: usize) -> PyResult<f32> {
    let num_particles = 10000;
    let config = AttractorConfig::new(num_particles, 2.0);
    
    let mut states = config.states;
    let constants = config.constants;
    
    let mut survived = num_particles as f32;
    
    for _ in 0..steps {
        for i in 0..num_particles {
            let idx = i * 8;
            
            if states[idx] == 0.0 && states[idx+1] == 0.0 && states[idx+2] == 0.0 {
                continue; // Skip diverged (we zero them below)
            }

            let I = (states[idx], states[idx+1]);
            let U = (states[idx+2], states[idx+3]);
            let L = (states[idx+4], states[idx+5]);
            
            let cx = constants[idx];
            let cy = constants[idx+1];
            let cu_x = constants[idx+2];
            let cu_y = constants[idx+3];
            let cl_x = constants[idx+4];
            let cl_y = constants[idx+5];

            let i_sqr = complex_sqr(I);
            let u_sqr = complex_sqr(U);
            let l_sqr = complex_sqr(L);

            let next_I = complex_mul((k, 0.0), complex_mul(I, (u_sqr.0 - l_sqr.0, u_sqr.1 - l_sqr.1)));
            let next_U = complex_mul((k, 0.0), complex_mul(U, (i_sqr.0 - l_sqr.0, i_sqr.1 - l_sqr.1)));
            let next_L = complex_mul((k, 0.0), complex_mul(L, (u_sqr.0 - i_sqr.0, u_sqr.1 - i_sqr.1)));
            
            states[idx] = next_I.0 + cx; states[idx+1] = next_I.1 + cy;
            states[idx+2] = next_U.0 + cu_x; states[idx+3] = next_U.1 + cu_y;
            states[idx+4] = next_L.0 + cl_x; states[idx+5] = next_L.1 + cl_y;

            let norm_I = (states[idx].powi(2) + states[idx+1].powi(2)).sqrt();
            let norm_U = (states[idx+2].powi(2) + states[idx+3].powi(2)).sqrt();
            let norm_L = (states[idx+4].powi(2) + states[idx+5].powi(2)).sqrt();
            
            let max_norm = norm_I.max(norm_U).max(norm_L);
            if max_norm > escape_radius {
                survived -= 1.0;
                states[idx] = 0.0; states[idx+1] = 0.0;
                states[idx+2] = 0.0; states[idx+3] = 0.0;
                states[idx+4] = 0.0; states[idx+5] = 0.0;
            }
        }
    }
    
    let survival_rate = survived / (num_particles as f32);
    // Score based on distance from 50% survival rate (highest chaotic balance is around 0.5)
    let score = -((survival_rate - 0.5).powi(2));
    Ok(score)
}

#[cfg(not(target_arch = "wasm32"))]
#[pyfunction]
fn evaluate_divergence_f32_vs_f64(k: f64, steps: usize) -> PyResult<Vec<f64>> {
    let mut rng = rand::thread_rng();
    
    // Generate identical initial conditions
    let initial_i_x: f32 = rng.gen_range(-0.01..0.01);
    let initial_i_y: f32 = rng.gen_range(-0.01..0.01);
    let initial_u_x: f32 = rng.gen_range(-0.01..0.01);
    let initial_u_y: f32 = rng.gen_range(-0.01..0.01);
    let initial_l_x: f32 = rng.gen_range(-0.01..0.01);
    let initial_l_y: f32 = rng.gen_range(-0.01..0.01);

    let cx_val: f32 = rng.gen_range(-2.0..2.0);
    let cy_val: f32 = rng.gen_range(-2.0..2.0);
    let cx = cx_val; let cy = cy_val;
    let cu_x = cx * 1.05; let cu_y = cy * 0.95;
    let cl_x = cx * 0.95; let cl_y = cy * 1.05;

    let mut i_32 = (initial_i_x, initial_i_y);
    let mut u_32 = (initial_u_x, initial_u_y);
    let mut l_32 = (initial_l_x, initial_l_y);
    
    let mut i_64 = (initial_i_x as f64, initial_i_y as f64);
    let mut u_64 = (initial_u_x as f64, initial_u_y as f64);
    let mut l_64 = (initial_l_x as f64, initial_l_y as f64);
    
    let k_32 = k as f32;

    let mut divergences = Vec::with_capacity(steps);

    for _ in 0..steps {
        // --- f32 iteration --- //
        let i_sqr_32 = complex_sqr(i_32);
        let u_sqr_32 = complex_sqr(u_32);
        let l_sqr_32 = complex_sqr(l_32);

        let next_i_32 = complex_mul((k_32, 0.0), complex_mul(i_32, (u_sqr_32.0 - l_sqr_32.0, u_sqr_32.1 - l_sqr_32.1)));
        let next_u_32 = complex_mul((k_32, 0.0), complex_mul(u_32, (i_sqr_32.0 - l_sqr_32.0, i_sqr_32.1 - l_sqr_32.1)));
        let next_l_32 = complex_mul((k_32, 0.0), complex_mul(l_32, (u_sqr_32.0 - i_sqr_32.0, u_sqr_32.1 - i_sqr_32.0)));
        
        i_32 = (next_i_32.0 + cx, next_i_32.1 + cy);
        u_32 = (next_u_32.0 + cu_x, next_u_32.1 + cu_y);
        l_32 = (next_l_32.0 + cl_x, next_l_32.1 + cl_y);

        // --- f64 iteration --- //
        let i_sqr_64 = complex_sqr_f64(i_64);
        let u_sqr_64 = complex_sqr_f64(u_64);
        let l_sqr_64 = complex_sqr_f64(l_64);

        let next_i_64 = complex_mul_f64((k, 0.0), complex_mul_f64(i_64, (u_sqr_64.0 - l_sqr_64.0, u_sqr_64.1 - l_sqr_64.1)));
        let next_u_64 = complex_mul_f64((k, 0.0), complex_mul_f64(u_64, (i_sqr_64.0 - l_sqr_64.0, i_sqr_64.1 - l_sqr_64.1)));
        let next_l_64 = complex_mul_f64((k, 0.0), complex_mul_f64(l_64, (u_sqr_64.0 - i_sqr_64.0, u_sqr_64.1 - i_sqr_64.0)));
        
        i_64 = (next_i_64.0 + cx as f64, next_i_64.1 + cy as f64);
        u_64 = (next_u_64.0 + cu_x as f64, next_u_64.1 + cu_y as f64);
        l_64 = (next_l_64.0 + cl_x as f64, next_l_64.1 + cl_y as f64);

        // Calculate full 6D Euclidean distance and record
        let div_i = (i_64.0 - i_32.0 as f64).powi(2) + (i_64.1 - i_32.1 as f64).powi(2);
        let div_u = (u_64.0 - u_32.0 as f64).powi(2) + (u_64.1 - u_32.1 as f64).powi(2);
        let div_l = (l_64.0 - l_32.0 as f64).powi(2) + (l_64.1 - l_32.1 as f64).powi(2);
        let distance: f64 = (div_i + div_u + div_l).sqrt();
        divergences.push(distance);
        
        // Stop evaluating if it already completely blew up past float capacities
        if distance > 1000.0 { break; }
    }
    
    Ok(divergences)
}

#[cfg(not(target_arch = "wasm32"))]
#[pymodule]
fn quadratic_map_attractor(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(evaluate_chaos_edge, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_divergence_f32_vs_f64, m)?)?;
    Ok(())
}
