import sys
try:
    import numpy as np
except ImportError:
    print("Please install numpy: pip install numpy")
    sys.exit(1)
import quadratic_map_attractor

def main():
    print("Starting parameter exploration...")
    best_k = 0.0
    best_score = -float('inf')
    
    escape_radius = 5.0
    steps = 100
    
    # 探索の範囲
    for k in np.linspace(0.4, 0.8, 41):
        score = quadratic_map_attractor.evaluate_chaos_edge(k, escape_radius, steps)
        print(f"k={k:.3f}, Score={score:.4f}")
        if score > best_score:
            best_score = score
            best_k = k
            
    print(f"\nOptimal Chaos Edge coupling strength found: k = {best_k:.3f}")

if __name__ == "__main__":
    main()
