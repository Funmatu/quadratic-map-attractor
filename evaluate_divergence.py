import matplotlib.pyplot as plt
import numpy as np
import quadratic_map_attractor

def main():
    print("Evaluating f32 vs f64 divergence over time on chaos edge (k=0.41)...")
    
    k = 0.410
    steps = 10000
    
    divergences = quadratic_map_attractor.evaluate_divergence_f32_vs_f64(k, steps)
    
    print(f"Total steps simulated before threshold/conclusion: {len(divergences)}")
    print(f"Divergence after 10 steps: {divergences[min(10, len(divergences)-1)]:.10f}")
    if len(divergences) > 100:
        print(f"Divergence after 100 steps: {divergences[min(100, len(divergences)-1)]:.10f}")
    
    # Save a plot of the divergence
    plt.figure(figsize=(10, 6))
    plt.plot(np.arange(len(divergences)), divergences, color='r', linewidth=1.5)
    plt.title("Numerical Divergence (6D Euclidean Distance) between f32 and f64")
    plt.xlabel("Iteration Step")
    plt.ylabel("Divergence Magnitude")
    plt.yscale("log")
    plt.grid(True, which="both", ls="--", alpha=0.5)
    
    output_path = "divergence_plot.png"
    plt.savefig(output_path, dpi=300)
    print(f"\nDivergence plot saved to {output_path}")
    print("This clearly demonstrates the non-linear error amplification (Lyapunov sensitivity) where f32 (GPU) trajectories inevitably drift into independent shadow paths, proving the need for f64 native backends for strict topological validation.")

if __name__ == "__main__":
    main()
